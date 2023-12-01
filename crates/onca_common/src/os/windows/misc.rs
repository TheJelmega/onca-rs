use windows::{
    core::GUID,
    Win32::System::Rpc::{UuidCreate, UuidCreateSequential},
};

pub(crate) fn create_v1_uuid() -> [u8; 16] {
    let mut uuid = GUID::default();
    unsafe { UuidCreateSequential(&mut uuid) };
    unsafe { core::mem::transmute(uuid) }
}

pub(crate) fn create_v4_uuid() -> [u8; 16] {
    let mut uuid = GUID::default();
    unsafe { UuidCreate(&mut uuid) };
    unsafe { core::mem::transmute(uuid) }
}
