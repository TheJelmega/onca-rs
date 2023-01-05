use core::{
    ptr,
    mem,
    ffi::c_void,
};

use windows::{
    core::{GUID, HRESULT},
    Win32::{
        Foundation::POINTL,
        System::{
            Com::*,
            Ole::*,
            SystemServices::*,
        },
    }, 
};

use crate::com::{
    create_iunknown_vtable,
    ComInterface
};

/// Convert an interface pointer to a reference to a IDataObject
pub fn data_obj_from_ptr<'a, T>(ptr: *mut T) -> &'a mut IDataObject {
    unsafe { &mut *(ptr as *mut _) }
}

pub const fn create_idroptarget_vtable<T>(
    query_interface: unsafe extern "system" fn(&mut T, &GUID, *mut *const c_void) -> HRESULT,
    add_ref: unsafe extern "system" fn(&mut T) -> u32,
    release: unsafe extern "system" fn(&mut T) -> u32,
    drag_enter: unsafe extern "system" fn(&mut T, ComInterface<IDataObject>, MODIFIERKEYS_FLAGS, POINTL, &mut DROPEFFECT) -> HRESULT,
    drag_over: unsafe extern "system" fn(&mut T, MODIFIERKEYS_FLAGS, POINTL, &mut DROPEFFECT) -> HRESULT,
    drag_leave: unsafe extern "system" fn(&mut T) -> HRESULT,
    drop: unsafe extern "system" fn(&mut T, ComInterface<IDataObject>, MODIFIERKEYS_FLAGS, POINTL, &mut DROPEFFECT) -> HRESULT
) -> IDropTarget_Vtbl {
    unsafe {
        IDropTarget_Vtbl {
            base__: create_iunknown_vtable(query_interface, add_ref, release),
            DragEnter: mem::transmute(drag_enter),
            DragOver: mem::transmute(drag_over),
            DragLeave: mem::transmute(drag_leave),
            Drop: mem::transmute(drop),
        }
    }
}


/// Create a FORMATETC for a HDROP
pub fn create_hdrop_formatetc() -> FORMATETC {
    FORMATETC {
        cfFormat: CF_HDROP.0 as u16,
        ptd: ptr::null_mut(),
        dwAspect: DVASPECT_CONTENT.0,
        lindex: -1,
        tymed: TYMED_HGLOBAL.0 as u32
    }
}