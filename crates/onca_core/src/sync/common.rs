use onca_core_macros::flags;

#[macro_export]
macro_rules! lock {
    ($($lockable: expr),*) => {
        $(
            $crate::sync::Lock::<_>::new(&mut $lockable);
        )*
    };
}

pub const MAX_NAME_LENGTH : usize = u8::MAX as usize;

pub type LockResult = Result<(), ()>;

/// Result of a try_lock
pub enum TryLockError
{
    /// The lock has been poisoned
    Poisoned,
    /// The lock would have been blocked (unsuccessful try_lock)
    WouldBlock,
}
pub type TryLockResult = Result<(), TryLockError>;

pub type WaitResult = Result<(), ()>;

pub enum SleepError
{
    Timeout,
    SystemErr(i32)
}
pub type SleepResult = Result<(), SleepError>;

/// Lockable type which can be used in a lock
pub trait Lockable
{
    /// Lock the lockable
    /// 
    /// Returns a result telling whether the lock was successful, if not, the lock was poisoned
    fn lock(&mut self) -> LockResult;
    /// Unlock the lockable
    fn unlock(&mut self);
    /// Try to lock the lockable and return if it was locked
    fn try_lock(&mut self) -> TryLockResult;
}

pub struct Lock<'a, A: Lockable>
{
    lockable : &'a mut A
}

impl<'a, A: Lockable> Lock<'a, A>
{
    pub fn new(lockable: &'a mut A) -> Self
    {
        if let Err(_) = lockable.lock()
            { panic!("Failed to acquire lock") };
        Lock::<_>{ lockable }
    }
}

impl<'a, A: Lockable> Drop for Lock<'a, A>
{
    fn drop(&mut self)
    {
        self.lockable.unlock();
    }
}

/// Barrier enter flags
#[flags(u8)]
pub enum BarrierEnterFlags
{
    /// No flags
    None,
    /// Keep spinning until last thread enters
    SpinOnly,
    /// Immediatally block until last thread enters
    BlockOnly,
    /// Notify that the barrier will not be deleted until all threads have exited
    /// 
    /// If any thread enters without this flag, the flag will be ignored
    /// 
    /// Using this may improve performance as it avoids additional check to handle the case where the barrier is deleted before completion
    NoDelete,
}