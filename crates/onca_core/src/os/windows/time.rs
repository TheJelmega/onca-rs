use windows::Win32::{
    Foundation::SYSTEMTIME,
    System::SystemInformation::GetSystemTime
};

use crate::time::TimeStamp;


pub fn get_timestamp() -> TimeStamp {
    unsafe {
        let sys_time = GetSystemTime();
        TimeStamp {
            year:        sys_time.wYear,
            month:       sys_time.wMonth as u8,
            day_of_week: sys_time.wDayOfWeek as u8,
            day:         sys_time.wDay as u8,
            hour:        sys_time.wHour as u8,
            minute:      sys_time.wMinute as u8,
            second:      sys_time.wSecond as u8,
            millisecond: sys_time.wMilliseconds
        }
    }
}