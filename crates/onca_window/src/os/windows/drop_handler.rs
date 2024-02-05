use core::ffi::c_void;
use onca_common::{
    prelude::*,
    alloc::ScopedAlloc,
};
use onca_logging::{log_warning, log_error, log_debug};
use win_utils::com::ComInterface;
use windows::{
    core::{HRESULT, GUID},
    Win32::{System::{
        Ole::*,
        Com::*,
        SystemServices::*,
    }, Foundation::{POINTL, DV_E_FORMATETC, S_OK}, UI::Shell::{HDROP, DragQueryFileA, DragFinish}}
};

use onca_windows_utils as win_utils;

use crate::{Window, WindowEvent, LOG_CAT};

#[repr(C)]
pub(crate) struct DropHandlerData {
    vtable     : &'static IDropTarget_Vtbl,
    self_ptr   : *mut DropHandlerData,
    ref_count  : win_utils::com::ComRefCount,
    window     : *mut Window,
    valid      : bool,
    effect     : DROPEFFECT,
}

pub const DROP_HANDLER_VTBL : IDropTarget_Vtbl = win_utils::ole::create_idroptarget_vtable(
    DropHandler::QueryInterface,
    DropHandler::AddRef,
    DropHandler::Release,
    DropHandler::DragEnter,
    DropHandler::DragOver,
    DropHandler::DragLeave,
    DropHandler::Drop
);

pub(crate) struct DropHandler {
    pub(crate) data : IDropTarget,
}

impl DropHandler {
    pub(crate) fn new(window: &mut Window) -> DropHandler {
        let mut handler_data = Box::new(DropHandlerData {
            vtable: &DROP_HANDLER_VTBL,
            self_ptr: std::ptr::null_mut(),
            ref_count: win_utils::com::ComRefCount::new(),
            window: window as *mut _,
            valid: false,
            effect: DROPEFFECT_NONE

        });

        let data = &mut *handler_data as *mut DropHandlerData;
        unsafe { (*data).self_ptr = Box::into_raw(handler_data) };
        //let drop_target = unsafe { core::mem::transmute::<_, IDropTarget>(&mut data) };
        let drop_target = unsafe { core::mem::transmute::<_, IDropTarget>(data) };

        DropHandler { data: drop_target }
    }

    #[allow(unused, non_snake_case)]
    unsafe extern "system" fn QueryInterface(_this: &mut DropHandlerData, _riid: &GUID, _ppv_object: *mut *const c_void) -> HRESULT {
        // Doesn't seem to be required for `IDropTarget`.
        unimplemented!()
    }

    #[allow(unused, non_snake_case)]
    unsafe extern "system" fn AddRef(this: &mut DropHandlerData) -> u32 {
        let this = Self::from_interface(this);
        this.ref_count.add_ref()
    }

    #[allow(unused, non_snake_case)]
    unsafe extern "system" fn Release(this: &mut DropHandlerData) -> u32 {
        let this = Self::from_interface(this);
        this.ref_count.release(|| {
            drop(Box::from_raw(this.self_ptr));
        })
    }

    #[allow(unused, non_snake_case)]
    // We ignore the modifiers given here, and the user is responsible to handle any special action with modifiers using `onca_input`
    unsafe extern "system" fn DragEnter(this: &mut DropHandlerData, data_obj: ComInterface<IDataObject>, _: MODIFIERKEYS_FLAGS, pt: POINTL, effect: &mut DROPEFFECT) -> HRESULT {
        let window = &mut *this.window;

        let hdrop = Self::iter_files(data_obj, |path| {
            log_debug!(LOG_CAT, "Started hovering file '{}' over window {} at location({}, {})", &path, window.id(), pt.x, pt.y);
            window.send_window_event(WindowEvent::HoverFileStarted(pt.x as u16, pt.y as u16, &path));
        });
        this.valid = hdrop.is_some();
        if this.valid {
            this.effect = DROPEFFECT_COPY
        } else {
            this.effect = DROPEFFECT_NONE
        }
        *effect = this.effect;

        S_OK
    }

    #[allow(unused, non_snake_case)]
    // We ignore the modifiers given here, and the user is responsible to handle any special action with modifiers using `onca_input`
    unsafe extern "system" fn DragOver(this: &mut DropHandlerData, mod_keys: MODIFIERKEYS_FLAGS, pt: POINTL, effect: &mut DROPEFFECT) -> HRESULT {
        if this.valid {
            let window = &mut *this.window;
            window.send_window_event(WindowEvent::HoverFileTick(pt.x as u16, pt.y as u16));
            *effect = this.effect;
        }

        S_OK
    }

    #[allow(unused, non_snake_case)]
    unsafe extern "system" fn DragLeave(this: &mut DropHandlerData) -> HRESULT {
        if this.valid {
            let window = &mut *this.window;
            log_debug!(LOG_CAT, "Stopped hovering files over window {}", window.id());
            window.send_window_event(WindowEvent::HoverFileHoverEnded);
            this.valid = false;
        }

        S_OK
    }

    // We ignore the modifiers given here, and the user is responsible to handle any special action with modifiers using `onca_input`
    #[allow(unused, non_snake_case)]
    unsafe extern "system" fn Drop(this: &mut DropHandlerData, data_obj: ComInterface<IDataObject>, mod_keys: MODIFIERKEYS_FLAGS, pt: POINTL, effect: &mut DROPEFFECT) -> HRESULT {
        let window = &mut *this.window;

        let hdrop = Self::iter_files(data_obj, |path| {
            log_debug!(LOG_CAT, "Dropped file '{}' over window {} at location({}, {})", &path, window.id(), pt.x, pt.y);
            window.send_window_event(WindowEvent::DroppedFile(pt.x as u16, pt.y as u16, &path));
        });
        if let Some(hdrop) = hdrop {
            DragFinish(hdrop);
        }
        this.valid = false;

        S_OK
    }

    unsafe fn from_interface<'a, I>(this: *mut I) -> &'a mut DropHandlerData {
        win_utils::from_interface(this)
    }

    unsafe fn iter_files<F: FnMut(String)>(data_obj: ComInterface<IDataObject>, mut callback: F) -> Option<HDROP> {
        let mut format = win_utils::ole::create_hdrop_formatetc();
        let res = data_obj.get().GetData(&mut format);
        match res {
            Ok(medium) => {
                let hdrop = HDROP(medium.u.hGlobal.0 as isize);

                let num_files = DragQueryFileA(hdrop, 0xFFFF_FFFF, None);

                for i in 0..num_files {
                    let _scope_alloc = ScopedAlloc::new(AllocId::TlsTemp);

                    let path_len = DragQueryFileA(hdrop, i, None);
                    let mut buf = Vec::<u8>::new();
                    buf.reserve(path_len as usize + 1);
                    buf.set_len(path_len as usize + 1);

                    let bytes_written = DragQueryFileA(hdrop, i, Some(&mut buf));
                    buf.set_len(bytes_written as usize);
                    callback(String::from_utf8_unchecked(buf));
                }
                Some(hdrop)
            },
            Err(err) => {
                if err.code() == DV_E_FORMATETC {
                    // If the dorpped item is not a file, this error will occur.
                    // In this case it is OK to return without taking further action
                    log_warning!(LOG_CAT, "Object dropped was not a file");
                    None
                } else {
                    log_error!(LOG_CAT, "Unexpected error occured while processing dropped/hovered item.");
                    None
                }
            },
        }
    }
}