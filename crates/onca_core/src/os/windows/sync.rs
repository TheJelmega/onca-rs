use windows::{
    core::PCWSTR, 
    Win32::{
        Foundation::*, 
        System::{WindowsProgramming::INFINITE, Threading::*}}
};
use crate::{sync::*, os::windows::name_to_wstr};
use core::ptr::null_mut;

// TODO(jel): should WaitFor*Object(s) be set as alertable?

/// Single-process mutex
/// 
/// Can be lighter than a multi-process mutex, as depending on the platform, the mutex can be less likely to transision to kernal mode
/// 
/// After initialization, the mutex should not be moved, as it relies on the location of the data in memory
/// 
/// If the object containing the mutex needs to be moved, consider putting it inside of a smart pointer, i.e. on the heap
// TODO(jel): use os alloc instead of libc::malloc/free
pub struct Mutex
{
    // SRWLOCK is used instead of CRITICAL_SECTION, as it is more perfomant and requires less data to be stored, even when just using the exclusive lock
    handle : RTL_SRWLOCK
}

impl Mutex
{
    /// Create a new mutex
    pub fn new() -> Self
    {
        let mut handle = RTL_SRWLOCK::default();
        unsafe { InitializeSRWLock(&mut handle) };
        Self { handle }
    }

    /// Get the native handle of the Mutex
    pub fn native_handle(&self) -> *const () { self.handle.Ptr as *const () }

    /// Sleep until awoken by the conditional variable
    pub fn sleep(&mut self, cond_var: &mut ConditionalVar) -> Result<(), i32>
    {
        let res = unsafe { SleepConditionVariableSRW(&mut cond_var.handle, &mut self.handle, INFINITE, 0).ok() };
        match res
        {
            Ok(_) => Ok(()),
            Err(err) => Err(err.code().0)
        }
    }

    /// Sleep until either awoken by the conditional variable or until the timeout runs out
    pub fn sleep_timeout(&mut self, cond_var: &mut ConditionalVar) -> SleepResult
    {
        let res = unsafe { SleepConditionVariableSRW(&mut cond_var.handle, &mut self.handle, INFINITE, 0).ok() };
        match res
        {
            Ok(_) => Ok(()),
            Err(err) => Err( if err.code().0 as u32 == ERROR_TIMEOUT.0 { SleepError::Timeout } else { SleepError::SystemErr(err.code().0) } )
        }
    }
}

impl Lockable for Mutex
{
    fn lock(&mut self) -> LockResult
    {
        unsafe { AcquireSRWLockExclusive(&mut self.handle) };
        Ok(())
    }

    fn unlock(&mut self)
    {
        unsafe { ReleaseSRWLockExclusive(&mut self.handle) };
    }

    fn try_lock(&mut self) -> TryLockResult
    {
        if unsafe { TryAcquireSRWLockExclusive(&mut self.handle).0 != 0 }
            { Ok(()) }
        else
            { Err(TryLockError::WouldBlock) }
    }
}

unsafe impl Sync for Mutex {}

/// Multi-process mutex
/// 
/// Is heavier than a single-process mutex, but can be used by other 
pub struct MultiProcessMutex
{
    handle : HANDLE
}

impl MultiProcessMutex
{
    // Create a multi-process mutex with an optional name
    pub fn new(name: Option<&str>) -> Result<Self, i32>
    {
        // TODO(jel): Use an actual String
        let mut name_arr = [0u16; (MAX_NAME_LENGTH + 1) * 2];
        let name_ptr : PCWSTR = match name
        {
            Some(name) => 
            {
                assert!(name.len() < MAX_NAME_LENGTH, "The name of a mutex can only be {} characters", MAX_NAME_LENGTH);
                name_to_wstr(name, &mut name_arr)
            },
            None => PCWSTR::default()
        };

        let handle = unsafe { CreateMutexW(null_mut(), false, name_ptr ) };

        match handle
        {
            Ok(handle) => Ok(Self { handle }),
            Err(err) => Err(err.code().0),
        }
    }

    /// Open an existing mutli-process mutex by it's name, this is not possible if the mutex was created without a name
    pub fn open(name: &str) -> Result<Self, i32>
    {
        let mut name_arr = [0u16; (MAX_NAME_LENGTH + 1) * 2];
        assert!(name.len() > 0, "Cannot open a mutex without a name");
        assert!(name.len() < MAX_NAME_LENGTH, "The name of a mutex can only be {} characters", MAX_NAME_LENGTH);

        let name_ptr = name_to_wstr(name, &mut name_arr);

        // https://docs.microsoft.com/en-us/windows/win32/sync/synchronization-object-security-and-access-rights
        const MUTEX_ALL_ACCESS : u32 = 0x1F0001; 
        // TODO(jel): Allow handle inheritance?
        let handle = unsafe { OpenMutexW(MUTEX_ALL_ACCESS, false, name_ptr) };
        match handle
        {
            Ok(handle) => Ok(Self { handle }),
            Err(err) => Err(err.code().0),
        }
    }

    // TODO(jel): Duplicate: https://docs.microsoft.com/en-us/windows/win32/api/handleapi/nf-handleapi-duplicatehandle
    
    // Get the native handle of the MultiPorcessMutex
    pub fn native_handle(&self) -> *const () { self.handle.0 as *const () }
}

impl Drop for MultiProcessMutex
{
    fn drop(&mut self)
    {
        unsafe { CloseHandle(self.handle) };
    }
}

// TODO(jel): Return os specific errors ???
impl Lockable for MultiProcessMutex
{
    fn lock(&mut self) -> LockResult
    {
        let res = unsafe { WaitForSingleObject(self.handle, INFINITE ) };
        if res == WAIT_OBJECT_0 { Ok(()) } else { Err(()) }
    }

    fn unlock(&mut self)
    {
        unsafe { ReleaseMutex(self.handle) };
    }

    fn try_lock(&mut self) -> TryLockResult
    {
        let res = unsafe { WaitForSingleObject(self.handle, INFINITE ) };

        const TIMEOUT : u32 = WAIT_TIMEOUT.0;
        match res
        {
            WAIT_ABANDONED => Err(TryLockError::Poisoned),
            TIMEOUT => Err(TryLockError::WouldBlock),
            _ => Ok(()),
        }
    }
}

unsafe impl Sync for MultiProcessMutex {}

/// Event
/// 
/// An event that can be waited for and needs to be manually set for the wait to end
pub struct Event
{
    handle       : HANDLE,
}

impl Event
{
    /// Create a new event
    /// 
    /// When 'manual_reset` is true, the event will not be automatically reset when any thread gains access to it after a wait, but needs to be manually reset with 'reset()'
    /// 
    /// 'initial_state' denotes whether the event should start as signalled or not
    pub fn new(name: Option<&str>, manual_reset: bool, initial_state: bool) -> Result<Self, i32>
    {
        let mut name_arr = [0u16; (MAX_NAME_LENGTH + 1) * 2];
        let name_ptr : PCWSTR = match name
        {
            Some(name) => 
            {
                assert!(name.len() < MAX_NAME_LENGTH, "The name of an event can only be {} characters", MAX_NAME_LENGTH);
                name_to_wstr(name, &mut name_arr)
            },
            None => PCWSTR::default()
        };
        let handle = unsafe { CreateEventW(null_mut(), manual_reset, initial_state, name_ptr) };
        match handle
        {
            Ok(handle) => Ok(Self { handle }),
            Err(err) => Err(err.code().0),
        }
    }

    pub fn open(name: &str) -> Result<Self, i32>
    {
        let mut name_arr = [0u16; (MAX_NAME_LENGTH + 1) * 2];
        assert!(name.len() > 0, "Cannot open a mutex without a name");
        assert!(name.len() < MAX_NAME_LENGTH, "The name of an event can only be {} characters", MAX_NAME_LENGTH);

        let name_ptr = name_to_wstr(name, &mut name_arr);

        // https://docs.microsoft.com/en-us/windows/win32/sync/synchronization-object-security-and-access-rights
        const EVENT_ALL_ACCESS : u32 = 0x1F0003; 
        // TODO(jel): Allow handle inheritance?
        let handle = unsafe { OpenEventW(EVENT_ALL_ACCESS, false, name_ptr) };
        match handle
        {
            Ok(handle) => Ok(Self { handle }),
            Err(err) => Err(err.code().0),
        }
    }

    // TODO(jel): Duplicate: https://docs.microsoft.com/en-us/windows/win32/api/handleapi/nf-handleapi-duplicatehandle

    /// Set the event
    pub fn set(&mut self)
    {
        let res = unsafe { SetEvent(self.handle) };
        assert!(res.as_bool(), "Failed to set event")
    }

    /// Manually reset the event, only affects state when created with `manual_reset` to `true`
    pub fn reset(&mut self)
    {
        { unsafe { ResetEvent(self.handle) }; }
    }

    /// Wait for the event to be signalled
    pub fn wait(&mut self) -> WaitResult
    {
        let res = unsafe { WaitForSingleObject(self.handle, INFINITE) };
        if res == WAIT_OBJECT_0 { Ok(()) } else { Err(()) }
    }

    /// Wait for the event with a `timeout` in milliseconds
    /// 
    /// Return whether the event was signalled, otherwise it hit the timout
    pub fn wait_timeout(&mut self, timeout: u32) -> WaitResult
    {
        let res = unsafe { WaitForSingleObject(self.handle, timeout) };
        if res == WAIT_OBJECT_0 { Ok(()) } else { Err(()) }
    }
}

impl Drop for Event
{
    fn drop(&mut self)
    {
        unsafe { CloseHandle(self.handle) };
    }
}

unsafe impl Sync for Event {}

/// Semaphore
/// 
/// A semaphore can be accessed by multiple threads
pub struct Semaphore
{
    handle : HANDLE,
}

impl Semaphore
{
    /// Create a semaphore
    /// 
    /// 'initial_count` denotes the initial amount of threads that can wait for it
    /// 
    /// 'max_count` denotes the maximum number of threads that can wait for it
    pub fn new(name: Option<&str>, initial_count: u32, max_count: u32) -> Result<Self, i32>
    {
        let mut name_arr = [0u16; (MAX_NAME_LENGTH + 1) * 2];
        let name_ptr : PCWSTR = match name
        {
            Some(name) => 
            {
                assert!(name.len() < MAX_NAME_LENGTH, "The name of a semaphore can only be {} characters", MAX_NAME_LENGTH);
                name_to_wstr(name, &mut name_arr)
            },
            None => PCWSTR::default()
        };

        let handle = unsafe { CreateSemaphoreW(null_mut(), initial_count as i32, max_count as i32, name_ptr) };
        match handle
        {
            Ok(handle) => Ok(Self { handle }),
            Err(err) => Err(err.code().0),
        }
    }

    pub fn open(name: &str) -> Result<Self, i32>
    {
        let mut name_arr = [0u16; (MAX_NAME_LENGTH + 1) * 2];
        assert!(name.len() > 0, "Cannot open a mutex without a name");
        assert!(name.len() < MAX_NAME_LENGTH, "The name of a semaphore can only be {} characters", MAX_NAME_LENGTH);

        let name_ptr = name_to_wstr(name, &mut name_arr);

        // https://docs.microsoft.com/en-us/windows/win32/sync/synchronization-object-security-and-access-rights
        const SEMAPHORE_ALL_ACCESS : u32 = 0x1F0003; 
        // TODO(jel): Allow handle inheritance?
        let handle = unsafe { OpenSemaphoreW(SEMAPHORE_ALL_ACCESS, false, name_ptr) };
        match handle
        {
            Ok(handle) => Ok(Self { handle }),
            Err(err) => Err(err.code().0),
        }
    }

    // TODO(jel): Duplicate: https://docs.microsoft.com/en-us/windows/win32/api/handleapi/nf-handleapi-duplicatehandle

    /// Release the current thread's access to the semaphore
    pub fn release(&mut self)
    {
        unsafe { ReleaseSemaphore(self.handle, 1, null_mut()) };
    }

    /// Release the current thread's access to the semaphore by incrementing it by the given value
    /// 
    /// Return the value of the semaphore before it was released
    pub fn release_count(&mut self, count: u32) -> u32
    {
        let mut old_count = 0;
        unsafe { ReleaseSemaphore(self.handle, count as i32, &mut old_count); }
        old_count as u32
    }

    /// Wait for the semaphore to 
    pub fn wait(&mut self) -> WaitResult
    {
        let res = unsafe { WaitForSingleObject(self.handle, INFINITE) };
        if res == WAIT_OBJECT_0 { Ok(()) } else { Err(()) }
    }

    pub fn wait_count(&mut self, count: u32) -> WaitResult
    {
        let mut handle_arr = [HANDLE(0); 16];
        assert!(count < 16, "count must be less or equal to 16");
        for i in 0..count as usize
        { handle_arr[i] = self.handle; }

        let res = unsafe { WaitForMultipleObjects(&handle_arr[0..count as usize], true, INFINITE) };
        if res >= WAIT_OBJECT_0 && res < WAIT_OBJECT_0 + count { Ok(()) } else { Err(()) }
    }
}

impl Drop for Semaphore
{
    fn drop(&mut self)
    {
        unsafe { CloseHandle(self.handle) };
    }
}

unsafe impl Sync for Semaphore {}

/// Waitable timer
/// 
/// If the timer was made with manual reset, the timer will stay in a signalled state (after first completion)
pub struct WaitableTimer
{
    handle : HANDLE
}

impl WaitableTimer
{
    /// Create a waitable timer
    pub fn new(name: Option<&str>, manual_reset: bool) -> Result<Self, i32>
    {
        let mut name_arr = [0u16; (MAX_NAME_LENGTH + 1) * 2];
        let name_ptr : PCWSTR = match name
        {
            Some(name) => 
            {
                assert!(name.len() < MAX_NAME_LENGTH, "The name of a waitable timer can only be {} characters", MAX_NAME_LENGTH);
                name_to_wstr(name, &mut name_arr)
            },
            None => PCWSTR::default()
        };

        let handle = unsafe { CreateWaitableTimerW(null_mut(), manual_reset, name_ptr) };
        match handle
        {
            Ok(handle) => Ok(Self { handle }),
            Err(err) => Err(err.code().0),
        }
    }

    pub fn open(name: &str) -> Result<Self, i32>
    {
        let mut name_arr = [0u16; (MAX_NAME_LENGTH + 1) * 2];
        assert!(name.len() > 0, "Cannot open a mutex without a name");
        assert!(name.len() < MAX_NAME_LENGTH, "The name of a waitable timer can only be {} characters", MAX_NAME_LENGTH);

        let name_ptr = name_to_wstr(name, &mut name_arr);

        // https://docs.microsoft.com/en-us/windows/win32/sync/synchronization-object-security-and-access-rights
        const TIMER_ALL_ACCESS : u32 = 0x1F0003; 
        // TODO(jel): Allow handle inheritance?
        let handle = unsafe { OpenWaitableTimerW(TIMER_ALL_ACCESS, false, name_ptr) };
        match handle
        {
            Ok(handle) => Ok(Self { handle }),
            Err(err) => Err(err.code().0),
        }
    }

    // TODO(jel): Duplicate: https://docs.microsoft.com/en-us/windows/win32/api/handleapi/nf-handleapi-duplicatehandle

    /// Set the number of milliseconds the timer needs to run before being signalled, with 0 being immediatally signalled
    pub fn set(&mut self, milliseconds: u32) -> Result<(), i32>
    {
        let mut due_time = 0i64;

        // TODO(jel): does 'true' for 'fResume' make sense? https://docs.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-setwaitabletimer
        // TODO(jel): should we have a version with a completion routine? https://docs.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-setwaitabletimer
        let res = unsafe { SetWaitableTimer(self.handle, &mut due_time, milliseconds as i32, None, null_mut(), false).as_bool() };
        if res { Ok(()) } else { Err(unsafe{ GetLastError().0 as i32 }) }
    }

    /// Cancel the timer
    /// 
    /// Returns whether it sucessfully cancelled the timer
    pub fn cancel(&mut self) -> bool
    {
        unsafe { CancelWaitableTimer(self.handle).as_bool() }
    }

    pub fn wait(&mut self) -> WaitResult
    {
        let res = unsafe { WaitForSingleObject(self.handle, INFINITE) };
        if res == WAIT_OBJECT_0 { Ok(()) } else { Err(()) }
    }
}

impl Drop for WaitableTimer
{
    fn drop(&mut self)
    {
        unsafe { CloseHandle(self.handle) };
    }
}

unsafe impl Sync for WaitableTimer {}

pub struct RwLock
{
    handle : RTL_SRWLOCK
}

impl RwLock
{
    /// Create a new mutex
    pub fn new() -> Self
    {
        let mut handle = RTL_SRWLOCK::default();
        unsafe { InitializeSRWLock(&mut handle) };
        Self { handle }
    }

    // Get the native handle of the Mutex
    pub fn native_handle(&self) -> *const () { self.handle.Ptr as *const () }

    pub fn lock_exclusive(&mut self) -> LockResult
    {
        unsafe { AcquireSRWLockExclusive(&mut self.handle) };
        Ok(())
    }

    pub fn lock_shared(&mut self) -> LockResult
    {
        unsafe { AcquireSRWLockShared(&mut self.handle) };
        Ok(())
    }

    pub fn unlock_exclusive(&mut self)
    {
        unsafe { ReleaseSRWLockExclusive(&mut self.handle) };
    }

    pub fn unlock_shared(&mut self)
    {
        unsafe { ReleaseSRWLockShared(&mut self.handle) };
    }

    pub fn try_lock_exclusive(&mut self) -> TryLockResult
    {
        if unsafe { TryAcquireSRWLockExclusive(&mut self.handle).0 != 0 }
            { Ok(()) }
        else
            { Err(TryLockError::WouldBlock) }
    }

    /// Sleep until awoken by the conditional variable
    pub fn sleep(&mut self, cond_var: &mut ConditionalVar) -> Result<(), i32>
    {
        let res = unsafe { SleepConditionVariableSRW(&mut cond_var.handle, &mut self.handle, INFINITE, 0).ok() };
        match res
        {
            Ok(_) => Ok(()),
            Err(err) => Err(err.code().0)
        }
    }

    /// Sleep until either awoken by the conditional variable or until the timeout runs out
    pub fn sleep_timeout(&mut self, cond_var: &mut ConditionalVar) -> SleepResult
    {
        let res = unsafe { SleepConditionVariableSRW(&mut cond_var.handle, &mut self.handle, INFINITE, 0).ok() };
        match res
        {
            Ok(_) => Ok(()),
            Err(err) => Err( if err.code().0 as u32 == ERROR_TIMEOUT.0 { SleepError::Timeout } else { SleepError::SystemErr(err.code().0) } )
        }
    }
}

unsafe impl Sync for RwLock {}

pub struct ConditionalVar
{
    pub(self) handle : RTL_CONDITION_VARIABLE
}

impl ConditionalVar
{
    pub fn new() -> Self
    {
        let mut handle = RTL_CONDITION_VARIABLE::default();
        unsafe { InitializeConditionVariable(&mut handle) };
        Self { handle }
    }

    /// Wake a single thread that is currently waiting for the conditional variable
    pub fn wake(&mut self)
    {
        unsafe { WakeConditionVariable(&mut self.handle) };
    }

    /// Wake all threads that are currently waiting for the conditional variable
    pub fn wake_all(&mut self)
    {
        unsafe { WakeAllConditionVariable(&mut self.handle) };
    }
}

unsafe impl Sync for ConditionalVar {}

/// Synchronization barrier
pub struct Barrier
{
    handle : RTL_BARRIER
}

impl Barrier
{
    /// Create a new synchronization barrier with the number of threads that need to enter, for the threads to continue
    pub fn new(num_threads: u32) -> Result<Self, i32>
    {
        let mut handle = RTL_BARRIER::default();
        let res = unsafe { InitializeSynchronizationBarrier(&mut handle, num_threads as i32, -1).as_bool() };
        if res { Ok(Self{ handle }) } else { Err(unsafe { GetLastError().0 } as i32) }
    }

    pub fn new_spin(num_threads: u32, spin_count: u32) -> Result<Self, i32>
    {
        let mut handle = RTL_BARRIER::default();
        let res = unsafe { InitializeSynchronizationBarrier(&mut handle, num_threads as i32, spin_count as i32).as_bool() };
        if res { Ok(Self{ handle }) } else { Err(unsafe { GetLastError().0 } as i32) }
    }

    // TODO(jel): pass actual flags
    /// Enter the barrier, `true` will be returned if the thread is the last thread to enter the barrier
    pub fn enter(&mut self, flags: BarrierEnterFlags) -> bool
    {
        //SYNCHRONIZATION_BARRIER_FLAGS_BLOCK_ONLY
        unsafe { EnterSynchronizationBarrier(&mut self.handle, flags.bits() as u32).as_bool() }
    }
}

impl Drop for Barrier
{
    fn drop(&mut self) 
    {
        let res = unsafe { DeleteSynchronizationBarrier(&mut self.handle).as_bool() };
        assert!(res, "Failed to delete synchronization barrier");
    }
}