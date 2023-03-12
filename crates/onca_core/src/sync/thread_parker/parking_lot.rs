// Portions of the project have been copied from parking_lot and is copyrighted by Amanieu d'Antra under the MIT license (located in: '3rd-party-licenses/parking_lot')
use core::{
    cell::{Cell, UnsafeCell},
    ptr,
    sync::atomic::{AtomicPtr, AtomicUsize, Ordering},
};
use std::{hash::Hash, mem::ManuallyDrop};

use super::{imp::ThreadParker, word_lock::WordLock};
use crate::{
    collections::{SmallDynArray, DynArray},
    time::{Duration, Instant},
    mem::HeapPtr,
    alloc::{UseAlloc, ScopedAlloc},
};

// NOTE(jel): parking_lot mentions that time::Instant doesn't work on wasm32-unknown-unknown

static NUM_TREADS : AtomicUsize = AtomicUsize::new(0);

/// Holds the pointerto the currently active `HashTable`.
/// 
/// # Safety
/// 
/// Except for the initial value of null, it must always point to a valid `HashTable` instance.
/// Any `HashTable` this global static has ever pointed to must never be freed.
static HASHTABLE : AtomicPtr<HashTable> = AtomicPtr::new(ptr::null_mut());

// Even with 3x mre buckets than threads, the memory overhead per thread is still only a few hundred bytes per thread.
const LOAD_FACTOR : usize = 3;

struct HashTable {
    /// Hash buckets for the table.
    entries   : HeapPtr<[Bucket]>,

    /// Number of bits used for the hash function.
    hash_bits : u32,

    // Previous table. This is only kept to keep leak detectors happy.
    _prev     : *const HashTable,
}

impl HashTable {
    #[inline]
    fn new(num_threads: usize, prev: *const HashTable) -> HeapPtr<HashTable> {
        let _scope_alloc = ScopedAlloc::new(UseAlloc::Malloc);

        let new_size = (num_threads * LOAD_FACTOR).next_power_of_two();
        let hash_bits = 0usize.leading_zeros() - new_size.leading_zeros() - 1;

        let now = Instant::now();
        let mut entries = DynArray::with_capacity(new_size);
        for i in 0..new_size {
            // We must ensure the seed is not zero
            entries.push(Bucket::new(now, i as u32 + 1));
        }

        HeapPtr::new(HashTable {
            entries: entries.into_heap_slice(),
            hash_bits,
            _prev: prev
        })
    }
}

struct Bucket {
    // Lock protecting the queue.
    mutex       : WordLock,

    // Linked lists of threads waiting on this bucket.
    queue_head  : Cell<*const ThreadData>,
    queue_tail  : Cell<*const ThreadData>,

    // Next time at which point be_fair should be said.
    fair_timout : UnsafeCell<FairTimout>
}

impl Bucket {
    #[inline]
    pub fn new(timeout: Instant, seed: u32) -> Self {
        Self { 
            mutex: WordLock::new(),
            queue_head: Cell::new(ptr::null()),
            queue_tail: Cell::new(ptr::null()),
            fair_timout: UnsafeCell::new(FairTimout::new(timeout, seed))
        }
    }
}

struct FairTimout {
    /// Next time at which point be_fair should be set.
    timeout: Instant,

    /// The PRNG state for calculating the next timeout.
    seed: u32
}

impl FairTimout {
    #[inline]
    fn new(timeout: Instant, seed: u32) -> Self {
        Self { timeout, seed }
    }

    /// Determine whether we should force a fair unlock, and update the timeout.
    #[inline]
    fn should_timout(&mut self) -> bool {
        let now = Instant::now();
        if now > self.timeout {
            // Time between 0 and 1ms.
            let nanos = self.gen_u32() % 1_000_000;
            self.timeout = now + Duration::new(0, nanos);
            true
        } else {
            false
        }
    }

    /// Pseudo-random number generatror from the "Xorshift RNGs" paper by George Marsaglia.
    fn gen_u32(&mut self) -> u32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 17;
        self.seed ^= self.seed << 5;
        self.seed
    }
}

struct ThreadData {
    parker             : ThreadParker,

    /// Key that this thread is sleeping on. This may change if the thread is requeued to a different key.
    key                : AtomicUsize,

    /// Linked list of parked threads in a bucket.
    next_in_queue      : Cell<*const ThreadData>,

    /// UnparkToken passed to this thread when it is unparked.
    unpark_token       : Cell<UnparkToken>,

    // ParkToken value set by the thread when it was parked.
    park_token         : Cell<ParkToken>,

    // Is the thread pakred with a timeout?
    parked_with_timout : Cell<bool>

    // NOTE(jel): parking lot has some additional data here for deadlock detection
}

impl ThreadData {
    fn new() -> ThreadData {
        // Keep track of the total number of live `ThreadData` objects and resize the hash table accordingly.
        let num_threads = NUM_TREADS.fetch_add(1, Ordering::Relaxed) + 1;
        grow_hashtable(num_threads);

        ThreadData { 
            parker: ThreadParker::new(),
            key: AtomicUsize::new(0), 
            next_in_queue: Cell::new(ptr::null()), 
            unpark_token: Cell::new(DEFAULT_UNPARK_TOKEN), 
            park_token: Cell::new(DEFAULT_PARK_TOKEN), 
            parked_with_timout: Cell::new(false)
        }
    } 
}

/// Invokes the given closure with a reference to the current thread `ThreadData`.
#[inline(always)]
fn with_thread_data<T>(f: impl FnOnce(&ThreadData) -> T) -> T {
    // Unlike `word_lock::ThreadData`, `parking_lot::ThreadData`is always expensive to construct.
    // Try to use a thread-local version if possible.
    // Otherwise just create a ThreadData on the stack
    let mut thread_data_storage = None;
    thread_local!(static THREAD_DATA: ThreadData = ThreadData::new());
    let thread_data_ptr = THREAD_DATA
        .try_with(|x| x as *const ThreadData)
        .unwrap_or_else(|_| thread_data_storage.get_or_insert_with(ThreadData::new));

        f(unsafe { &*thread_data_ptr })
}

impl Drop for ThreadData {
    fn drop(&mut self) {
        NUM_TREADS.fetch_sub(1, Ordering::Relaxed);
    }
}

///Returns a reference to the last hash table, creating one if it doesn't exist yet.
/// The reference is valid forever.
/// However, teh `HashTable` it references might become stale at any point.
/// Meaning it still exists, but it is not the instance in active use
#[inline]
fn get_hashtable() -> &'static HashTable {
    let table = HASHTABLE.load(Ordering::Acquire);

    // If there is no table, create one
    if table.is_null() {
        create_hashtable()
    } else {
        // SAFETY: whe not null, 'HASHTABLE' always points to a 'HashTable' that is never freed.
        unsafe { &*table }
    }    
}

/// Returns a reference to the latest hash table, creating on if it doesn't exists yet.
/// The reference is valid forever.
/// However, the `HashTable` it references might become stale at any point.
/// Meaning it still exists, but it is not the instance in active use.
#[cold]
fn create_hashtable() -> &'static HashTable {
    let mut new_table = ManuallyDrop::new(HashTable::new(LOAD_FACTOR, ptr::null()));

    // If this fails then it means some other thread created the hash table first
    let table = match HASHTABLE.compare_exchange(
        ptr::null_mut(), 
        new_table.ptr_mut(), 
        Ordering::AcqRel, 
        Ordering::Acquire
    ) {
        Ok(_) => new_table.ptr_mut(),
        Err(old_table) => {
            // Free the table we created
            // SAFETY: `new_table` is created above and only freed here
            let _ = unsafe { ManuallyDrop::take(&mut new_table) };
            old_table
        }
    };

    // SAFETY: The `HashTable` behind `table` is never freed.
    // It is either the table pointer we created, or it is one loaded from `HASHTABLE`
    unsafe { &*table }
}

/// Grow the hash table so that it is big enough for the given number of threads.
/// This isn't performance critical sinze it is only done when a ThreadData is created, which only happens once per thread.
fn grow_hashtable(num_threads: usize) {
    // Lock all buckets in the existing table and get a reference to it
    let old_table = loop {
        let table = get_hashtable();

        // Check if we need to resize the existing table
        if table.entries.len() >= LOAD_FACTOR * num_threads {
            return;
        }

        // Lock all buckets in the old state
        for bucket in &table.entries[..] {
            bucket.mutex.lock();
        }

        // Now check if our table is still the last one.
        // Another thread could have grown the hash table between us reading HASHTABLE and locking the buckets.
        if HASHTABLE.load(Ordering::Relaxed) == table as *const _ as *mut _ {
            break table;
        }

        // Unlock buckets and try again
        for bucket in &table.entries[..] {
            unsafe { bucket.mutex.unlock() };
        }
    };

    // Create the new table
    let mut new_table = ManuallyDrop::new(HashTable::new(num_threads, old_table));

    // Move the entries from the old table to the new one
    for bucket in &old_table.entries[..] {
        // SAFETY: The park, unpark* and check_wait_graph_fast function create only correct linked lists.
        // All `ThreadData` instances in these lists will remain valid as long as they are present in the lists, meaning as long as their threads are parked.
        unsafe { rehash_bucket_into(bucket, &mut new_table) }; 
    }

    // Publish the new table. No races are possible at this point because any other thread trying to growthe hash table is blocked on the bucket locks in the old table
    HASHTABLE.store(new_table.ptr_mut(), Ordering::Release);

    // Unlock all buckets in the old table
    for bucket in &old_table.entries[..] {
        // SAFETY: We hold the lock here, as required
        unsafe { bucket.mutex.unlock() };
    }
}

/// Iterate through all `ThreadData` objects in the bucket and insert them into the given table in the bucket their key corresponds to for this table
/// 
/// # Safety
/// 
/// The fiven `bucket` must have a correctly constructed linked list under `queue_head`, containing `ThreadData` instances that must stay valid for at least as long as teh given `table` is in use
/// 
/// The given `table` must only contain buckets with correctly constructed linked lists
unsafe fn rehash_bucket_into(bucket: &'static Bucket, table: &mut HashTable) {
    let mut current: *const ThreadData = bucket.queue_head.get();
    while !current.is_null() {
        let next = (*current).next_in_queue.get();
        let hash = hash((*current).key.load(Ordering::Relaxed), table.hash_bits);
        if table.entries[hash].queue_tail.get().is_null() {
            table.entries[hash].queue_head.set(current);
        } else {
            (*table.entries[hash].queue_tail.get())
                .next_in_queue
                .set(current);
        }
        table.entries[hash].queue_tail.set(current);
        (*current).next_in_queue.set(ptr::null());
        current = next;
    }
}

/// Hash function for adresses
// the `parking_lot` crate also supports 32-bit, but we don't
#[inline]
fn hash(key: usize, bits: u32) -> usize {
    key.wrapping_mul(0x9E3779B97F4A7C15) >> (64 - bits)
}

/// Locks the bucket for the given key and returns a reference to it.
/// The returned bucket must be unlocked again in order to not cause deadlocks
#[inline]
fn lock_bucket(key: usize) -> &'static Bucket {
    loop {
        let hashtable = get_hashtable();

        let hash = hash(key, hashtable.hash_bits);
        let bucket = &hashtable.entries[hash];

        // Lock the bucket
        bucket.mutex.lock();

        // If no other thread has rehashed the table before we grabbed the lock, then we are god to go!
        // The lock we grabbed prevents any rehashes
        if HASHTABLE.load(Ordering::Relaxed) == hashtable as *const _ as *mut _ {
            return bucket;
        }

        // Unlock the bucket and try again
        // SAFETY: We hold the lock here, as required
        unsafe { bucket.mutex.unlock() }
    }
}

/// Lock the bucket for the given key and returns a reference to it.
/// But checks that the key hasn't been changed in the meantime dueue to a requeue.
/// The returned bucket must be unlocked again in order to not cause deadlocks
#[inline]
fn lock_bucket_checked(key: &AtomicUsize) -> (usize, &'static Bucket) {
    loop {
        let hashtable = get_hashtable();
        let current_key = key.load(Ordering::Relaxed);

        let hash = hash(current_key, hashtable.hash_bits);
        let bucket = &hashtable.entries[hash];

        // Lock the bucket
        bucket.mutex.lock();

        // Check that both the hash table and key are correct while the bucket is locked.
        // Note that the key can't change once we locked the proper bucket for it, so we just keep trying until we have the correct key
        if HASHTABLE.load(Ordering::Relaxed) == hashtable as *const _ as *mut _ &&
            key.load(Ordering::Relaxed) == current_key
        {
            return (current_key, bucket);
        }

        // Unlock the bucket and try again
        // SAFETY: We hold the lock here, as required
        unsafe { bucket.mutex.unlock() }
    }
}

/// Locks the two buckets for the given pair of keys and returns references to them.
/// The returned buckets must be unlocked again in order to not cause deadlocks.
/// 
/// If both keys hash to the same value, both returned referecnes will be to the same bucket.
/// Be careful to only unlock itonce in this case, always use `unlock_bucket_pair`
#[inline]
fn lock_bucket_pair(key1: usize, key2: usize) -> (&'static Bucket, &'static Bucket) {
    loop {
        let hashtable = get_hashtable();

        let hash1 = hash(key1, hashtable.hash_bits);
        let hash2 = hash(key2, hashtable.hash_bits);

        // Get the bucket at the lowest hash/index first
        let bucket1 = if hash1 <= hash2 {
            &hashtable.entries[hash1]
        } else {
            &hashtable.entries[hash2]
        };

        // Lock the first bucket
        bucket1.mutex.lock();

        // If no other thread has rehashed the table before we grabbed the lock, then we are good to go!
        // The lock we grabben prevents any rehashes.
        if HASHTABLE.load(Ordering::Relaxed) == hashtable as *const _ as *mut _ {
            // No lock the second bucket and return the two buckets
            if hash1 == hash2 {
                return (bucket1, bucket1);
            } else if hash1 < hash2 {
                let bucket2 = &hashtable.entries[hash2];
                bucket2.mutex.lock();
                return (bucket1, bucket2);
            } else {
                let bucket2 = &hashtable.entries[hash1];
                bucket2.mutex.lock();
                return (bucket1, bucket2);
            }
        }
    }
}

/// Unlock a pair of buckets
/// 
/// # Safety
/// 
/// Both buckets must be locked
#[inline]
unsafe fn unlock_bucket_pair(bucket1: &'static Bucket, bucket2: &'static Bucket) {
    bucket1.mutex.unlock();
    if !ptr::eq(bucket1, bucket2) {
        bucket2.mutex.unlock();
    }
}

/// Result of a park operation
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ParkResult {
    /// We were unparked by another thread with the given token.
    Unparked(UnparkToken),

    /// The validation callback returned false.
    Invalid,

    /// The timeout expired.
    Timeout,
}

impl ParkResult {
    /// Returns `true` if we were unparked by another thread.
    #[inline]
    pub fn is_unparked(self) -> bool {
        matches!(self, ParkResult::Unparked(..))
    }
}

/// Result of an unpark operation.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct UnparkResult {
    /// The number of threads that were unparked.
    pub unparked_threads  : usize,

    /// The number of threads that were requeued.
    pub requeued_threads  : usize,

    /// Whether there are any threads remaining in the queue.
    /// This only return true if a thread was unparked.
    pub have_more_threads : bool,

    /// this is set to `true` on average every 0.5ms for any given key.
    /// It should be used to switch to a fair unlocking mechanism for a particular unlock.
    pub be_fair           : bool,

    /// Private field so new fields can be added without breakage.
    _sealed               : ()
}

/// Operation that `unpark_requeue` should perform.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RequeueOp {
    /// Abort the operation without doing anything.
    Abort,

    /// Unpark one thread and requeue the rest onto the target queue
    UnparkOnRequestRest,

    /// Requeue all threads onto the target queue.
    RequeueAll,

    /// Unpark one thread and leave the rest parked. No equeueing is done.
    UnparkOne,

    /// Requeue one htread and leave the rest parked on the original queue
    RequeueOne,
}

/// Operation that `unpark_filter` should perform for each thread
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FilterOp {
    /// Unpark the htread and continue scanning the list of parked threads
    Unpark,

    /// Don't unpark the thread and continue the list of parked threads
    Skip,

    /// Don't unpark the thread and stop scanning the list of parked threads
    Stop
}

/// A value which is passed from an unparker to a parked thread
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct UnparkToken(pub usize);

/// A value associated with a parked thread which can be used by `unpark_filter`
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ParkToken(pub usize);

/// A default unpark token to use
pub const DEFAULT_UNPARK_TOKEN : UnparkToken = UnparkToken(0);

/// A default park token to use
pub const DEFAULT_PARK_TOKEN : ParkToken = ParkToken(0);

/// Parks the current thread in the queue associated with the given key.
/// 
/// The 'validate' function is called while the queue is locked and can abort the operation by returning `false`.
/// If `validate` returns `true`, then the current thread is appended to the queue is unlocked.
/// 
/// The 'before_sleep' function is called after the queue is unlocked, but before the thread is put to sleep.
/// The thread will then sleep until it is unparked or the given timeout is reached.
/// 
/// The `timed_out` function is also called while the queue is locked, but only if the timeout was reached.
/// It is passed the key of the queue it was in when it timed out, which may be different from the original key if `unpark_queue` was called.
/// It is also passed a bool which indicates whether it was the last thread in the queue.
/// 
/// # Safety
/// 
/// You should only call this function with an address that you control, since you could otherwise interfere with the operation of other synchronization primitives.
/// 
/// The `validate` and `timed_out` functions are called while the queue is locked and must not panic or call into any fuction in `parking_lot`.
/// 
/// The `before_sleep` function is called outside the queue look and is allowed to call `unpark_one`, `unpark_all`, `unpark_requeue` of `unpark_filter`, but it is not allowed to call `park` or panic.
#[inline]
pub unsafe fn park(
    key: usize,
    validate: impl FnOnce() -> bool,
    before_sleep: impl FnOnce(),
    timed_out: impl FnOnce(usize, bool),
    park_token: ParkToken,
    timeout: Option<Instant>
) -> ParkResult {
    // Grab out thread data, this also ensures that the hash table exists
    with_thread_data(|thread_data| {
        // Lock th bucket for the given key
        let bucket = lock_bucket(key);

        // If the validation functin falls, just return
        if !validate() {
            // SADETY: We hold the lock here, as required
            bucket.mutex.unlock();
            return ParkResult::Invalid;
        }

        // Append out thread data to the queue and unlock the bucket
        thread_data.parked_with_timout.set(timeout.is_some());
        thread_data.next_in_queue.set(ptr::null());
        thread_data.key.store(key, Ordering::Relaxed);
        thread_data.park_token.set(park_token);
        thread_data.parker.prepare_park();
        if !bucket.queue_head.get().is_null() {
            (*bucket.queue_tail.get()).next_in_queue.set(thread_data);
        } else {
            bucket.queue_head.set(thread_data);
        }
        // SAFETY: We hold the lock here, as required
        bucket.mutex.unlock();

        // Invode the pre-sleep callback
        before_sleep();

        // Park our thread and determine whether we were woken up by an unpark or by our timeout.
        // Note that this isn't precise: we can still be unparked since we are still in the queue.
        let unparked = match timeout {
            Some(timeout) => thread_data.parker.park_until(timeout),
            None => {
                thread_data.parker.park();
                // NOTE(jel): `parking_lot` has a call to the deadlock detector here
                true
            }
        };

        // If we were unparked, return now
        if unparked {
            return ParkResult::Unparked(thread_data.unpark_token.get());
        }

        // Lock our bucket again. Note that the hashtable may have been rehashed in the meantime.
        // Our key may also have changed if we were requeued.
        let (key, bucket) = lock_bucket_checked(&thread_data.key);

        // Now we need to check again if we were unparked or timed out.
        // Unlike the last check, this is precise because we hold the bucket lock
        if !thread_data.parker.timed_out() {
            // SAFETY: We hold the lock here, as required
            bucket.mutex.unlock();
            return ParkResult::Unparked(thread_data.unpark_token.get());
        }

        // We timed out, so we now need to remove our thread from the queue
        let mut link = &bucket.queue_head;
        let mut current = bucket.queue_head.get();
        let mut previous = ptr::null();
        let mut was_last_thread = true;
        while !current.is_null() {
            if current == thread_data {
                let next = (*current).next_in_queue.get();
                link.set(next);
                if bucket.queue_tail.get() == current {
                    bucket.queue_tail.set(previous);
                } else {
                    // Scan the rest of the queue to see if there are any other entries with the given key.
                    let mut scan = next;
                    while !scan.is_null() {
                        if (*scan).key.load(Ordering::Relaxed) == key {
                            was_last_thread = false;
                            break;
                        }
                        scan = (*scan).next_in_queue.get();
                    }
                }

                // Callback to indicate that we timed out, and whether we werer the last thread on the queue.
                timed_out(key, was_last_thread);
                break;
            } else {
                if (*current).key.load(Ordering::Relaxed) == key {
                    was_last_thread = false;
                }
                link = &(*current).next_in_queue;
                previous = current;
                current = link.get();
            }
        }

        // There should be no way for our thread to have been removed from the queue if we timed out
        debug_assert!(!current.is_null());

        // Unlock the bucket
        // SAFETY: We hold the lock here, as required
        bucket.mutex.unlock();
        ParkResult::Timeout
    })
}

/// Unparks one thread from the queue associated with the given key
/// 
/// The `callback` function is called while the queue is locked and before the target thread is woken up.
/// The `UnpartkResult` argument to the function indicated whether a thread was found in the queue and whtehr this was the last thread in the queue.
/// This value is also returned by `unpark_one`.
/// The `callback`fucntion should return an 'UparkToken` value which will be passed to the thread that is unparked.
/// If no thread is unparked, then the returned value is ignored
/// 
/// # Safety
/// 
/// You should only call this function with an address that you control, since you could otherwise interfere with the operation of other synchronization primitives.
/// 
/// The `callback` function is called while the queue is locked and must not panic or call into any function in `parking_lot`.
/// 
/// THe `parking_lot` functions are not re-entrant and calling this method from the context of an asynchronous signal handler may result in undefined behavior, including corruption of internal state and/or deadlocks.
#[inline]
pub unsafe fn unpark_one(
    key: usize,
    callback: impl FnOnce(UnparkResult) -> UnparkToken
) -> UnparkResult {
    // Lock the bucket for the given key
    let bucket = lock_bucket(key);

    // Find a thread with a matching key and remove it from the queue.
    let mut link = &bucket.queue_head;
    let mut current = bucket.queue_head.get();
    let mut previous = ptr::null();
    let mut result = UnparkResult::default();
    while !current.is_null() {
        if (*current).key.load(Ordering::Relaxed) == key {
            // Remove the thread from the queue
            let next = (*current).next_in_queue.get();
            link.set(next);
            if bucket.queue_tail.get() == current {
                bucket.queue_tail.set(previous);
            } else {
                // Scan the rest of the queue to see if there are any other entries with the given key
                let mut scan = next;
                while !scan.is_null() {
                    if (*scan).key.load(Ordering::Relaxed) == key {
                        result.have_more_threads = true;
                        break;
                    }
                    scan = (*scan).next_in_queue.get();
                }
            }

            // Invoke the callback before waking up the thread
            result.unparked_threads = 1;
            result.be_fair = (*bucket.fair_timout.get()).should_timout();
            let token = callback(result);

            // Set the token for the target thread
            (*current).unpark_token.set(token);

            // This is a bit tricky: we first lock the ThreadParker to prevent the thread from exiting and freeing its ThreadData if its wait times out.
            // Then we unlock the queue since we don't want to keep the queue locked while we perform a system call.
            // Finally we wake up the parked thread
            let handle = (*current).parker.unpark_lock();
            // SAFETY: We hold the lock here, as required
            bucket.mutex.unlock();
            handle.unpark();

            return result;
        } else {
            link = &(*current).next_in_queue;
            previous = current;
            current = link.get();
        }
    }

    // No threads with a matching key were found in the bucket
    callback(result);
    // SAFETY: We hold the lock here, as required
    bucket.mutex.unlock();
    result
}

/// Unparks all threads in the queue associated with the given key.
/// 
/// The given `UnparkToken` is passed to all unparked threads
/// 
/// This function returns the number of threads that were unparked.
/// 
/// # Safety
/// 
/// You should only call this function with an address that you control, since you could otherwise interfere with the operation of other synchronization primitives.
/// 
/// The `parking_lot` functions re not re-entrant and calling this method from the context of an asychronous signal handler may result in undefined behavior, including corruption of internal state and/or deadlocks.
#[inline]
pub unsafe fn unpark_all(key: usize, unpark_token: UnparkToken) -> usize {
    let _scope_alloc = ScopedAlloc::new(UseAlloc::Malloc);

    // Lock the bucket for the given key
    let bucket = lock_bucket(key);

    // remove all threads with the given key in the bucket
    let mut link = &bucket.queue_head;
    let mut current = bucket.queue_head.get();
    let mut previous = ptr::null();
    let mut threads = SmallDynArray::<_, 8>::new();
    while !current.is_null() {
        if (*current).key.load(Ordering::Relaxed) == key {
            // Remove the thread from the queue
            let next = (*current).next_in_queue.get();
            link.set(next);
            if bucket.queue_tail.get() == current {
                bucket.queue_tail.set(previous);
            }

            // Set the token for the target thread
            (*current).unpark_token.set(unpark_token);

            // Don't wake up treads while holding the queue lock.
            // See comment in unpark_one.
            // For now just recond which threads we need to wake up.
            threads.push((*current).parker.unpark_lock());
            current = next;
        } else {
            link = &(*current).next_in_queue;
            previous = current;
            current = link.get();
        }
    }

    // Unlock the bucket
    // SAFETY: We hold the lock here, as required
    bucket.mutex.unlock();

    // Now that we are outside the lock, wake up all the threads that we removed from the queue.
    let num_threads = threads.len();
    for handle in threads.into_iter() {
        handle.unpark();
    }

    num_threads
}

/// Removes all threads from the queue associated with `key_from`, optionally unparks the first one and requeues the rest onto the queue associated with `key_to`.
/// 
/// The `validate` function is called while both queues are locked
/// Its return value will determine which operation is performed, or whether the operation should be aborted..
/// See `RequeueOp` for datils about the different possible return values
/// 
/// The `callback` function is also called while both queues are locked.
/// It is passed the `ReueueOp` returned by `validate` and an `UnparkResult` indicating whether a thread was unparked and whether a thread was unparked and whetherthere are threads still parked in the new queue.
/// This `UnparkResult` value is also returned by `unpark_requeue`
/// 
/// The `callback` function should return an `Unpark` value which will be passed to the thread that is unparked. 
/// If no thread is unparked then the returned value is ignored.
/// 
/// # Safety
/// 
/// You should only call this function with an address that you control, since you could otherwise interfere with the operation of other synchronization primitives.
/// 
/// The `validate` and `callback` functions are called while thequque is locked and must not panic or call into any funciton in `parking_lot`
#[inline]
pub unsafe fn unpark_requeue(
    key_from: usize,
    key_to: usize,
    validate: impl FnOnce() -> RequeueOp,
    callback: impl FnOnce(RequeueOp, UnparkResult) -> UnparkToken
) -> UnparkResult {
    // Lock the two buckets for the given keys
    let (bucket_from, bucket_to) = lock_bucket_pair(key_from, key_to);

    // If the validation function fails, just return
    let mut result = UnparkResult::default();
    let op = validate();
    if op == RequeueOp::Abort {
        // SAFETY: Both are locked, as required
        unlock_bucket_pair(bucket_from, bucket_to);
        return result;
    }

    // Remove all threads with the given key in the source bucket
    let mut link = &bucket_from.queue_head;
    let mut current = bucket_from.queue_head.get();
    let mut previous = ptr::null();
    let mut requeue_threads: *const ThreadData = ptr::null();
    let mut requeue_threads_tail: *const ThreadData = ptr::null();
    let mut wakeup_thread = None;
    while !current.is_null() {
        if (*current).key.load(Ordering::Relaxed) == key_from {
            // Remove the thread from the queue
            let next = (*current).next_in_queue.get();
            link.set(next);
            if bucket_from.queue_tail.get() == current {
                bucket_from.queue_tail.set(previous);
            }

            // prepare the first thread for wakeup and requeue the rest
            if (op == RequeueOp::UnparkOnRequestRest || op == RequeueOp::UnparkOne) &&
                wakeup_thread.is_none()
            {
                wakeup_thread = Some(current);
                result.unparked_threads = 1;
            } else {
                if !requeue_threads.is_null() {
                    (*requeue_threads_tail).next_in_queue.set(current);
                } else {
                    requeue_threads = current;
                }
                requeue_threads_tail = current;
                (*current).key.store(key_to, Ordering::Relaxed);
                result.requeued_threads += 1;
            }

            if op == RequeueOp::UnparkOne || op == RequeueOp::RequeueOne {
                // Scan the rest of the queue to see if there are any other
                let mut scan = next;
                while !scan.is_null() {
                    if (*scan).key.load(Ordering::Relaxed) == key_from {
                        result.have_more_threads = true;
                        break;
                    }
                    scan = (*scan).next_in_queue.get();
                }
                break;
            }
            current = next;
        } else {
            link = &(*current).next_in_queue;
            previous = current;
            current = link.get();
        }
    }

    // Add the requeued threads to teh destination bucket
    if !requeue_threads.is_null() {
        (*requeue_threads_tail).next_in_queue.set(ptr::null());
        if !bucket_to.queue_head.get().is_null() {
            (*bucket_to.queue_tail.get())
                .next_in_queue
                .set(requeue_threads);
        } else {
            bucket_to.queue_head.set(requeue_threads);
        }
        bucket_to.queue_tail.set(requeue_threads_tail);
    }

    // Invoke the callback before waking up the thread
    if result.unparked_threads != 0 {
        result.be_fair = (*bucket_from.fair_timout.get()).should_timout();
    }
    let token = callback(op, result);

    // See comment  in unpark_one fro why we mess with the locking
    if let Some(wakeup_thread) = wakeup_thread {
        (*wakeup_thread).unpark_token.set(token);
        let handle = (*wakeup_thread).parker.unpark_lock();
        // SAFETY: We hold the lock here, as required
        unlock_bucket_pair(bucket_from, bucket_to);
        handle.unpark();
    } else {
        // SAFETY: We hold the lock here, as required
        unlock_bucket_pair(bucket_from, bucket_to);
    }

    result
}

/// Unparks a number of threads from teh front of the queue associated wit h`key` depending on the result of a filter function which inspect the `ParkToken` associated with each thread.
/// 
/// The `filter` function is called for each thread in teh queue or until `FilterOp::Stop` is returned. 
/// This function is passed the `ParkToken` associated wit ha particular thread, which is unparked if `FilterOp::Unpark` is returned
/// 
/// The `callback` function is also called while both queues are locked.
/// It is passed an `UnparkResult` indicating the number of threads that were unparked and whether ther are still parked threads in the queue.
/// This `UnparkResult` value is also returned by `unpark_filter`
/// 
/// THe `callback` function should return an `UnparkToken` value which will be passed to all threads that are unparked.
/// If no thread is unparked, then the returned value is ignored.
/// 
/// # Safety
/// 
/// You should only call this function with an address that you control, since you could otherwise interfere with the operation of other synchronization primitives.
/// 
/// The `filter` and `callback` function are called while the queue is locked and must not panic or call into any function in `parking_lot`
#[inline]
pub unsafe fn unpark_filter(
    key: usize,
    mut filter: impl FnMut(ParkToken) -> FilterOp,
    callback: impl FnOnce(UnparkResult) -> UnparkToken,
) -> UnparkResult {
    // Lock the bucket for the given key
    let bucket = lock_bucket(key);

    // Go through the queue looking for threads with a matching key
    let mut link = &bucket.queue_head;
    let mut current = bucket.queue_head.get();
    let mut previous = ptr::null();
    let mut threads = SmallDynArray::<_, 8>::new();
    let mut result = UnparkResult::default();
    while !current.is_null() {
        if (*current).key.load(Ordering::Relaxed) == key {
            // Call the filter function with teh thread's ParkToken
            let next = (*current).next_in_queue.get();
            match filter((*current).park_token.get()) {
                FilterOp::Unpark => {
                    // Remove the thread from the queue
                    link.set(next);
                    if bucket.queue_tail.get() == current {
                        bucket.queue_tail.set(previous);
                    }

                    // Add the thread to our list of threads to unpark
                    threads.push((current, None));

                    current = next;
                },
                FilterOp::Skip => {
                    result.have_more_threads = true;
                    link = &(*current).next_in_queue;
                    previous = current;
                    current = link.get();
                },
                FilterOp::Stop => {
                    result.have_more_threads = true;
                    break;
                }
            }
        } else {
            link = &(*current).next_in_queue;
            previous = current;
            current = link.get();
        }
    }

    // Invoke the callback before walking up the threads
    result.unparked_threads = threads.len();
    if result.unparked_threads != 0 {
        result.be_fair = (*bucket.fair_timout.get()).should_timout();
    }
    let token = callback(result);

    // Pass the token to all threads that are  going to be unparked and prepare them fro uhparking.
    for t in threads.iter_mut() {
        (*t.0).unpark_token.set(token);
        t.1 = Some((*t.0).parker.unpark_lock());
    }

    // SAFETY: We hold the lock here, as required
    bucket.mutex.unlock();

    // Now that we are outside the lock, wake up all the threads that we removed from the queue.
    for (_, handle) in threads.into_iter() {
        handle.unwrap_unchecked().unpark();
    }

    result
}
