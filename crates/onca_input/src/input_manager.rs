use core::{
    ptr::null_mut,
    num::NonZeroU8,
};
use std::{collections::HashMap, sync::{Arc, atomic::{AtomicBool, Ordering}}, ffi::c_void};

use onca_common::{
    prelude::*,
    sync::{Mutex, RwLock, MutexGuard},
    event_listener::EventListener,
    time::DeltaTime,
    sys::{self, is_on_main_thread},
};
use onca_hid as hid;
use onca_logging::{log_verbose, log_error, log_warning, log_info};
use onca_toml::Toml;
use onca_window::{WindowManager, WindowId};

use crate::{
    os::{self, OSInput},
    input_devices::{Keyboard, InputDevice},
    LOG_INPUT_CAT, Mouse, Gamepad, ControlScheme, User, DeviceType, AxisValue, ControlSchemeID, InputAxisId, MappingContext, NativeDeviceHandle, Handle, parse_definitions, GenericDevice
};



// TODO: Register device with custom API, so would ignore `InputDevice::handleInput` and manage it in `InputDevice::tick`
struct DeviceStorage {
    devices: Vec<(u8, Option<Box<dyn InputDevice>>)>,
}

impl DeviceStorage {
    fn new() -> Self {
        Self { devices: Vec::new() }
    }

    fn get_device_mut(&mut self, handle: Handle) -> Option<&mut dyn InputDevice> {
        let idx = handle.id as usize;
        if idx >= self.devices.len() ||
            self.devices[idx].0 != handle.lifetime
        {
            return None;
        }

        if let Some(dev) = &mut self.devices[idx].1 {
            Some(dev.as_mut())
        } else {
            None
        }
    }

    fn get_device(&self, handle: Handle) -> Option<&dyn InputDevice> {
        let idx = handle.id as usize;
        if idx >= self.devices.len() ||
            self.devices[idx].0 != handle.lifetime
        {
            return None;
        }

        if let Some(dev) = &self.devices[idx].1 {
            Some(dev.as_ref())
        } else {
            None
        }
    }

    fn get_device_types(&self, handle: Handle) -> DeviceType {
        self.get_device(handle).map_or(DeviceType::Other("<unknown>".to_string()), |dev| dev.get_device_type())
    }

    fn has_device(&self, handle: Handle) -> bool {
        self.get_device(handle).is_some()
    }

    fn add_device(&mut self, dev: Box<dyn InputDevice>) -> Handle {
        match self.devices.iter().position(|(_, opt)| opt.is_none()) {
            Some(idx) => {
                self.devices[idx].0 += 1;
                self.devices[idx].1 = Some(dev);
                Handle { id: idx as u8, lifetime: self.devices[idx].0 }
            },
            None => {
                let idx = self.devices.len();
                self.devices.push((0, Some(dev)));
                Handle { id: idx as u8, lifetime: 0 }
            },
        }
    }

    fn remove_device(&mut self, handle: Handle) -> Option<NativeDeviceHandle> {
        let dev = core::mem::take(&mut self.devices[handle.id as usize].1);
        if let Some(mut dev) = dev {
            Some(dev.take_native_handle())
        } else {
            None
        }
    }

    fn tick(&mut self, dt: f32, rebind_notify: &mut dyn FnMut(InputAxisId)) {
        for (_, opt) in &mut self.devices {
            if let Some(dev) = opt {
                dev.tick(dt, rebind_notify);
            }
        }
    }

    fn handle_hid_input(&mut self, handle: Handle, raw_report: &[u8]) {
        if let Some(dev) = self.get_device_mut(handle) {
            dev.handle_hid_input(raw_report);
        } else {
            log_error!(LOG_INPUT_CAT, DeviceStorage::handle_hid_input, "Failed to find device to process hid report")
        }
    }

    fn handle_native_input(&mut self, handle: Handle, native_data: *const c_void) {
        if let Some(dev) = self.get_device_mut(handle) {
            dev.handle_native_input(native_data);
        } else {
            log_error!(LOG_INPUT_CAT, DeviceStorage::handle_hid_input, "Failed to find device to process hid report")
        }
    }
}

/// Result returned by a rebind handler
pub enum RebindResult {
    /// Continue to try and rebind the key, e.g. invalid axis
    Continue,
    /// Accept a rebind with the given axis, e.g. confirmed by `confirm` key and return the actual key
    Accept(InputAxisId),
    /// Cancel the rebind
    Cancel,
}

struct RebindContext {
    binding_name    : String,
    context_name    : Option<String>,
    rebind_callback : Box<dyn Fn(InputAxisId) -> RebindResult>,
}

type CreateDevicePtr = Box<dyn Fn(NativeDeviceHandle) -> Result<Box<dyn InputDevice>, NativeDeviceHandle>>;

/// Manager for all input: devices, bindings, events, etc
/// 
/// All processing for the input manager is handled by the main thread
pub struct InputManager {
    pub(crate) os_input:     Mutex<os::OSInput>,
    pub(crate) mouse:        Option<Mouse>,
    pub(crate) keyboard:     Option<Keyboard>,
    device_store:            RwLock<DeviceStorage>,
    raw_input_listener:      Arc<Mutex<RawInputListener>>,

    device_custom_creators:  Mutex<Vec<(Box<dyn Fn(&hid::Identifier, &str) -> bool>, CreateDevicePtr)>>,
    device_product_creators: Mutex<HashMap<hid::VendorProduct, CreateDevicePtr>>,
    device_usage_creators:   Mutex<HashMap<hid::Usage, CreateDevicePtr>>,

    mapping_contexts:        Mutex<Vec<MappingContext>>,

    control_schemes:         RwLock<Vec<ControlScheme>>,
    users:                   RwLock<Vec<User>>,

    unused_devices:          Mutex<Vec<Handle>>,
    has_init_devices:        AtomicBool,

    rebind_context:          Mutex<Option<RebindContext>>
}

impl InputManager {
    pub fn new(window_manager: &Box<WindowManager>) -> Result<Arc<Self>, i32> {
        assert!(sys::is_on_main_thread(), "The input manager should only be created on the main thread");

        let main_window = match window_manager.get_main_window() {
            Some(window) => window,
            None => {
                log_error!(LOG_INPUT_CAT, Self::new, "Cannot create the window manager before the main window is created");
                return Err(0);
            },
        };
        
        let os_input = os::OSInput::new(main_window)?;
        let mut ptr = Arc::new(Self {
            os_input: Mutex::new(os_input),
            mouse: None,
            keyboard: None,
            device_store: RwLock::new(DeviceStorage::new()),
            raw_input_listener: Arc::new(Mutex::new(RawInputListener::new())),
            device_product_creators: Mutex::new(HashMap::new()),
            device_custom_creators: Mutex::new(Vec::new()),
            device_usage_creators: Mutex::new(HashMap::new()),
            mapping_contexts: Mutex::new(Vec::new()),
            control_schemes: RwLock::new(Vec::new()),
            // Make sure that there is 1 user
            users: RwLock::new(vec![User::new()]),
            has_init_devices: AtomicBool::new(false),
            unused_devices: Mutex::new(Vec::new()),
            rebind_context: Mutex::new(None),
        });
        ptr.raw_input_listener.lock().init(&ptr);
        window_manager.register_raw_input_listener(ptr.raw_input_listener.clone());

        os::register_input_devices(&ptr);

        Ok(ptr)
    }

    /// Register an input device createor for custom logic, e.g. controllers that support a custom API
    pub fn register_custom_create_device<P, F>(&self, pred: P, usages: &[hid::Usage], create_dev: F)
    where
        P: Fn(&hid::Identifier, &str) -> bool + 'static,
        F: Fn(NativeDeviceHandle) -> Result<Box<dyn InputDevice>, NativeDeviceHandle> + 'static
    {
        {
            let mut dev_custom_creators = self.device_custom_creators.lock();
            dev_custom_creators.push((Box::new(pred), Box::new(create_dev)));
        }
        self.os_input.lock().register_device_usages(usages);
    }
    
    /// Register an input device creator for a specific device, including the hid usages it needs to have registered
    pub fn register_product_create_device<F>(&self, vendor_product: hid::VendorProduct, usages: &[hid::Usage], create_dev: F)
    where
        F: Fn(NativeDeviceHandle) -> Result<Box<dyn InputDevice>, NativeDeviceHandle> + 'static
    {
        {
            let mut dev_prod_creators = self.device_product_creators.lock();
            if !dev_prod_creators.contains_key(&vendor_product) {
                dev_prod_creators.insert(vendor_product, Box::new(create_dev));
            } else {
                log_warning!(LOG_INPUT_CAT, "Device creation has already been registered for vendor and product: {vendor_product}");
            }
        }
        self.os_input.lock().register_device_usages(usages);
    }

    /// Register a generic input device creator for a specific usage
    pub fn register_usage_creator_device<F>(&self, usage: hid::Usage, create_dev: F)
    where
        F: Fn(NativeDeviceHandle) -> Result<Box<dyn InputDevice>, NativeDeviceHandle> + 'static
    {
        {
            let mut usage_creators = self.device_usage_creators.lock();
            if !usage_creators.contains_key(&usage) {
                usage_creators.insert(usage, Box::new(create_dev));
            } else {
                log_warning!(LOG_INPUT_CAT, "Device creation has already been registered for usage: {usage}");
            }
        }
        self.os_input.lock().register_device_usages(&[usage]);
    }

    pub fn register_generic_hid_definitions(&self, toml: &Toml) {
        let defs = parse_definitions(toml);
        for def in defs {
            self.register_product_create_device(def.vendor_product, &[], move |handle| GenericDevice::new(handle, &def).map(|x| {
                // We need to get around rust not realizing that `Box` could `CoerseUnsized` directly in a return statement
                // This could be one of those "std::boxed::Box is special" cases, as the first line clearly shows that it works
                let res: Box<dyn InputDevice> = Box::new(x);
                res
            }));
        }
    }

    /// Register a mapping context.
    /// 
    /// # Error
    /// 
    /// If a mapping contexts with
    pub fn register_mapping_context(&self, mapping_context: MappingContext) -> Result<(), MappingContext> {
        let mut contexts = self.mapping_contexts.lock();
        if contexts.iter().find(|ctx| &ctx.identifier == &mapping_context.identifier).is_none() {
            contexts.push(mapping_context);
            Ok(())
        } else {
            Err(mapping_context)
        }
    }

    /// Add a mapping context for a given user.
    /// 
    /// If other mappings with the same priority exist, the new mapping will be inserted at the end of mappings with that priority
    pub fn add_mapping_context_to_user(&self, user_idx: u8, priority: u16, mapping_context_identifier: &str) {
        let user_idx = user_idx as usize;
        let mut users = self.users.write();
        if user_idx >= users.len() {
            log_warning!(LOG_INPUT_CAT, "Trying to add mapping context to user that cannot exists");
            return;
        }

        if let Some(mapping_context) = self.mapping_contexts.lock().iter().find(|ctx| ctx.identifier == *mapping_context_identifier) {
            users[user_idx].add_mapping_context(priority, mapping_context.clone());
        } else {
            log_warning!(LOG_INPUT_CAT, "Trying to add unregistered mapping context '{mapping_context_identifier}'");
        }
    }

    /// Remove a mapping context for a given user
    pub fn remove_mapping_context_from_user(&self, user_idx: u8, mapping_context_identifier: &str) {
        let user_idx = user_idx as usize;
        let mut users = self.users.write();
        if user_idx >= users.len() {
            log_warning!(LOG_INPUT_CAT, "Trying to remove mapping context from user that cannot exists");
            return;
        }

        users[user_idx as usize].remove_mapping_context(mapping_context_identifier);
    }

    /// Remove all mappings with a priority below the given `priority` from a given user
    pub fn remove_mapping_context_below_priority_from_user(&self, user_idx: u8, priority: u16) {
        let user_idx = user_idx as usize;
        let mut users = self.users.write();
        if user_idx >= users.len() {
            log_warning!(LOG_INPUT_CAT, "Trying to remove mapping context from user that cannot exists");
            return;
        }

        users[user_idx as usize].remove_mapping_with_priority_below(priority);
    }

    /// Rebind an input with a given name and a callback to control the rebind
    /// 
    /// An identifier for the mapping context can be optionally supplied, limiting the rebind only to the given context, and not to all bindings with the given rebind name.
    pub fn rebind<F>(&mut self, binding_name: &str, mapping_context_identifier: Option<&str>, rebind_callback: F)
    where
        F : Fn(InputAxisId) -> RebindResult + 'static
    {
        let mut rebind_context = self.rebind_context.lock();
        if rebind_context.is_some() {
            log_warning!(LOG_INPUT_CAT, "Trying to start rebind when rebind is still in progress");
            return;
        }

        *rebind_context = Some(RebindContext {
            binding_name: binding_name.to_string(),
            context_name: mapping_context_identifier.map(|s| s.to_string()),
            rebind_callback: Box::new(rebind_callback)
        })
    }
  
    pub fn tick(&self, dt: DeltaTime) {
        assert!(sys::is_on_main_thread(), "The input manager should only be ticked on the main thread");

        // Initialize devices that are already connected on the initial tick
        if !self.has_init_devices.load(Ordering::SeqCst) {
            self.init_devices();
        }

        scoped_alloc!(AllocId::TlsTemp);

        // Update OS input
        self.os_input.lock().tick();
        
        // Update devices
        let mut rebind_axes = Vec::new();
        let mut callback = |input: InputAxisId| if self.rebind_context.lock().is_none() { rebind_axes.push(input) };
        
        self.device_store.write().tick(dt.get_dt(), &mut callback);
        
        for axis in rebind_axes {
            self.notify_rebind(axis);
        }

        let device_store = self.device_store.read();
        let mut users = self.users.write();
        if users.len() != 1 {
            let schemes = self.control_schemes.read();
            let mut unused_devices = self.unused_devices.lock();
            // If there are users that don't have a control set, try to generate them
            for (user_idx, user) in users.iter_mut().enumerate() {
                if !unused_devices.is_empty() && user.control_set().is_none() {
                    for scheme in &*schemes {
                        // The first control scheme that can be created will be used
                        if let Some(control_set) = scheme.create_control_set(&unused_devices, |handle| device_store.get_device_types(handle)) {
                            unused_devices.retain(|handle| !control_set.devices().contains(handle));
                            user.set_control_set(control_set);
                            break;
                        }
                    }
                }
                user.process_input(dt, user_idx as u8, |user, axis| Self::get_input_for_user(user, axis, &device_store));
            }
        } else {
            assert!(users.len() == 1);
            users[0].process_input(dt, 0, |_, axis| self.get_input_for_any(axis, &device_store));
        }
    }

    /// Set the maximum number of users that can be created.4
    /// 
    /// If `1` is passed, all input devices will be consumed by user 0, regardless of control scheme.
    /// If more than `1` is passed, each user will only ever have a single active control scheme, which cannot be switched without removing the user first.
    pub fn set_max_users(&self, max_users: NonZeroU8) {
        self.users.write().resize_with(max_users.get() as usize, || User::new());
    }

    
    /// Add a new possible control scheme to this user
    pub fn add_control_scheme(&mut self, scheme: ControlScheme) {
        self.control_schemes.write().push(scheme)
    }

    /// Remove a control scheme
    pub fn remove_control_scheme(&self, identifier: &ControlSchemeID) {
        let mut control_schemes = self.control_schemes.write();
        if let Some(idx) = control_schemes.iter().position(|scheme| scheme.identifier() == identifier) {
            control_schemes.remove(idx);
            for user in &mut *self.users.write() {
                user.notify_scheme_removed(identifier);
            }
        }
    }

    pub(crate) fn has_device(&self, handle: Handle) -> bool {
        self.device_store.read().has_device(handle)
    }

    pub(crate) fn can_create_device_for(&self, hid_iden: hid::Identifier) -> bool {
        self.device_product_creators.lock().contains_key(&hid_iden.vendor_device) ||
            self.device_usage_creators.lock().contains_key(&hid_iden.usage)
    }

    pub(crate) fn add_device(&self, iden: hid::Identifier, native_handle: NativeDeviceHandle) -> Option<Handle> {
        //log_info!(LOG_INPUT_CAT, "Trying to add device {device}");

        if let Some(input_dev) = self.create_input_device_for(iden, native_handle) {
            
            let usage = iden.usage;
            let vendor = input_dev.get_native_handle().hid_dev.as_ref().map_or_else(
                || String::new(),
                |dev| dev.get_vendor_string().unwrap_or(input_dev.get_hid_identifier().vendor_device.vendor.to_string())
            );
            let product = input_dev.get_native_handle().hid_dev.as_ref().map_or_else(
                || String::new(),
                |dev| dev.get_vendor_string().unwrap_or(input_dev.get_hid_identifier().vendor_device.device.to_string())
            );

            if vendor.is_empty() || product.is_empty() {
                log_info!(LOG_INPUT_CAT, "Added new input device '{}' for usage {usage}", input_dev.get_hid_identifier().vendor_device);
            } else {
                log_info!(LOG_INPUT_CAT, "Added new input device '{{ vendor: {vendor}, product: {product} }}' for usage {usage}");
            }
            
            let mut store = self.device_store.write();
            let handle = store.add_device(input_dev);

            let dev = store.get_device(handle).unwrap();
            self.os_input.lock().notify_device_added(handle, dev.get_native_handle());

            // First check if any user had this device disconnected, if so, give it the device and try to create the control scheme
            for user in &mut *self.users.write() {
                if let None = user.control_set() && user.try_reconnect_device(handle, dev.get_native_handle()) {
                    return Some(handle);
                }
            }
            // Otherwise, add to the available devices
            self.unused_devices.lock().push(handle);
            Some(handle)
        } else {
            None
        }
    }

    pub(crate) fn handle_hid_input(&self, handle: Handle, raw_report: &[u8]) {
        self.device_store.write().handle_hid_input(handle, raw_report);
    }

    pub (crate) fn handle_native_input(&self, handle: Handle, native_data: *const c_void) {
        self.device_store.write().handle_native_input(handle, native_data);
    }

    pub(crate) fn remove_device(&self, handle: Handle) {
        let mut store = self.device_store.write();

        if !store.has_device(handle) {
            return;
        }

        let input_dev = store.get_device(handle).unwrap();
        let iden = input_dev.get_hid_identifier();
        let usage = iden.usage;
        let vendor = input_dev.get_native_handle().hid_dev.as_ref().map_or_else(
            || String::new(),
            |dev| dev.get_vendor_string().unwrap_or(input_dev.get_hid_identifier().vendor_device.vendor.to_string())
        );
        let product = input_dev.get_native_handle().hid_dev.as_ref().map_or_else(
            || String::new(),
            |dev| dev.get_vendor_string().unwrap_or(input_dev.get_hid_identifier().vendor_device.device.to_string())
        );

        if vendor.is_empty() || product.is_empty() {
            log_info!(LOG_INPUT_CAT, "Removed input device '{}' for usage {usage}", input_dev.get_hid_identifier().vendor_device);
        } else {
            log_info!(LOG_INPUT_CAT, "Removed input device '{{ vendor: {vendor}, product: {product} }}' for usage {usage}");
        }

        let mut native_handle = match store.remove_device(handle) {
            Some(handle) => handle,
            None => return,
        };

        let mut users = self.users.write();
        for (idx, user) in users.iter_mut().enumerate() {
            native_handle = match user.notify_device_removed(handle, native_handle) {
                Ok(_) => {
                    if user.get_currently_held_devices().is_empty() {
                        // We can safely remove the value at the index and invalidate the iterator, as we don't use it anymore
                        users.remove(idx);
                    }

                    // Only one user can hold a device, so we don't need to check any other ones
                    break;
                },
                Err(handle) => handle,
            }
        }
    }

    pub(crate) fn notify_rebind(&self, input: InputAxisId) {
        let mut rebind_context = self.rebind_context.lock();

        if let Some(ctx) = &mut *rebind_context {
            match (ctx.rebind_callback)(input) {
                RebindResult::Continue => (),
                RebindResult::Accept(input) => {
                    // Update the source context
                    for mapping_context in &mut *self.mapping_contexts.lock() {
                        if let Some(ident) = &ctx.context_name && mapping_context.identifier != *ident {
                            continue;
                        }
                    }

                    // Propagate to context instances
                    let mut users = self.users.write();
                    for user in &mut *users {
                        user.rebind(&ctx.binding_name, ctx.context_name.as_ref(), input.clone());
                    }
                },
                // We can safely set it to `None` as we won't be using ctx after this
                RebindResult::Cancel => *rebind_context = None,
            }
        }
    }

    pub(crate) fn get_os_input(&self) -> MutexGuard<OSInput> {
        self.os_input.lock()
    }

    fn init_devices(&self) {
        // Register:
        let default_usages = [
            // - Pointer
            //hid::Usage::from_u16(1, 1),
            // - Mouse
            hid::Usage::from_u16(1, 2),
            // - Gamepad
            hid::Usage::from_u16(1, 4),
            // - Joystick
            //hid::Usage::from_u16(1, 5),
            // - Keyboard
            hid::Usage::from_u16(1, 6),
            // - Keypad
            //hid::Usage::from_u16(1, 7),

            // - External Pen Device
            //hid::Usage::from_u16(13, 1),
            // - Integrated Pen Device
            //hid::Usage::from_u16(13, 2),
            // - Touchscreen
            //hid::Usage::from_u16(13, 4),
            // - Precision Touchpad
            //hid::Usage::from_u16(13, 5),
        ];


        self.os_input.lock().register_device_usages(&default_usages);

        // Register built-in device creators
        self.register_usage_creator_device(hid::Usage::from_u16(1, 2), |handle| Mouse::new(handle).map(|x| {
            // We need to get around rust not realizing that `Box` could `CoerseUnsized` directly in a return statement
            // This could be one of those "std::boxed::Box is special" cases, as the first line clearly shows that it works
            let res: Box<dyn InputDevice> = Box::new(x);
            res
        }));
        self.register_usage_creator_device(hid::Usage::from_u16(1, 6), |handle| Keyboard::new(handle).map(|x| {
            // We need to get around rust not realizing that `Box` could `CoerseUnsized` directly in a return statement
            // This could be one of those "std::boxed::Box is special" cases, as the first line clearly shows that it works
            let res: Box<dyn InputDevice> = Box::new(x);
            res
        }));

        let gamepad_usage = hid::Usage::from_u16(1, 5);
        self.register_usage_creator_device(gamepad_usage, |handle| Gamepad::new(handle).map(|x| {
            // We need to get around rust not realizing that `Box` could `CoerseUnsized` directly in a return statement
            // This could be one of those "std::boxed::Box is special" cases, as the first line clearly shows that it works
            let res: Box<dyn InputDevice> = Box::new(x);
            res
        }));

        // Create devices for unloaded devices
        //let native_dev_handles = self.os_input.get_devices().unwrap();
        //for handle in native_dev_handles {
        //    self.add_device(*handle.native.get_hid_identifier(), handle);
        //}

        self.has_init_devices.store(true, Ordering::SeqCst);
    }

    fn create_input_device_for(&self, ident: hid::Identifier, mut handle: NativeDeviceHandle) -> Option<Box<dyn InputDevice>> {
        // Find the best fitting, registered device
        //
        // 1) any device that matches custom logic
        let hid_iden = handle.get_hid_identifier();
        let unique_iden = handle.get_unique_identifier();

        if let Some(create) = self.device_custom_creators.lock().iter().find(|(fun, _)| fun(hid_iden, unique_iden)) {
            handle = match create.1(handle) {
                Ok(ptr) => return Some(ptr),
                Err(handle) => handle,
            };
        }

        // 2) any device that matches the specific vendor and product
        if let Some(create) = self.device_product_creators.lock().get(&ident.vendor_device) {
            handle = match create(handle) {
                Ok(ptr) => return Some(ptr),
                Err(handle) => handle,
            };
        }

        // 3) any device that matches the usage
        if let Some(create) = self.device_usage_creators.lock().get(&ident.usage) {
            _ = match create(handle) {
                Ok(ptr) => return Some(ptr),
                Err(handle) => handle,
            };
        }

        // If we have not match for either a product or usage, we don't know about this input device
        log_verbose!(LOG_INPUT_CAT, "Trying to create an input device for an unknown hid product/usage '{ident}', ignoring this device");
        None
    }
    
    fn get_input_for_any(&self, axis_path: &InputAxisId, device_store: &DeviceStorage) -> AxisValue {
        for opt in &device_store.devices {
            if let (_, Some(dev)) = opt {
                if let Some(val) = dev.get_axis_value(axis_path) {
                    return val;
                }
            }
        }
        // If we don't have a value, return a false value
        AxisValue::Digital(false)
    }

    fn get_input_for_user(user: &User, axis_path: &InputAxisId, device_store: &DeviceStorage) -> AxisValue {
        match user.control_set() {
            Some(control_set) => {
                for handle in control_set.devices() {
                    let store = device_store;
                    if let Some(val) = store.get_device(*handle).and_then(|dev| dev.get_axis_value(axis_path)) {
                        return val;
                    }
                }
                AxisValue::Digital(false)
            },
            None => AxisValue::Digital(false),
        }
    }
}

impl Drop for InputManager {
    fn drop(&mut self) {
        self.raw_input_listener.lock().shutdown();
    }
}

struct RawInputListener {
    manager : Option<Arc<InputManager>>
}

impl RawInputListener {
    pub(crate) fn new() -> Self {
        Self { manager: None }
    }

    pub(crate) fn init(&mut self, manager: &Arc<InputManager>) {
        self.manager = Some(manager.clone());
    }

    pub(crate) fn shutdown(&mut self) {
        self.manager = None;
    }
}

impl EventListener<onca_window::RawInputEvent> for RawInputListener {
    fn notify(&mut self, event: &onca_window::RawInputEvent) {
        // SAFETY: `process_window_event`: We know the implementation to be safe
        // SAFETY: deref: Events can only be sent via the main thread (via the window manager), so our deref here is not causing mutability over multiple threads
        if let Some(manager) = &mut self.manager {
            unsafe { OSInput::process_window_event(&**manager, event) };
        }
    }
}