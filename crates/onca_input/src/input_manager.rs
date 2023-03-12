use core::{
    ptr::null_mut,
    num::NonZeroU8,
};

use onca_core::{
    prelude::*,
    collections::HashMap,
    sync::{Mutex, RwLock},
    event_listener::EventListener,
    time::DeltaTime,
    sys,
};
use onca_hid as hid;
use onca_logging::{log_verbose, log_error, log_warning, log_info};
use onca_window::WindowManager;

use crate::{os::{self, OSInput}, input_devices::{Keyboard, InputDevice}, LOG_INPUT_CAT, Mouse, Gamepad, ControlScheme, User, DeviceHandle, DeviceType, AxisValue, ControlSchemeID, InputAxisId, MappingContext};

// TODO: Register device with custom API, so would ignore `InputDevice::handleInput` and manage it in `InputDevice::tick`
struct DeviceStorage {
    devices        : DynArray<Option<(HeapPtr<hid::Device>, HeapPtr<dyn InputDevice>)>>,
    device_mapping : HashMap<hid::DeviceHandle, usize>,
}

impl DeviceStorage {
    fn new() -> Self {
        Self { devices: DynArray::new(), device_mapping: HashMap::new() }
    }

    fn get_device_mut(&mut self, handle: hid::DeviceHandle) -> Option<(&mut hid::Device, &mut dyn InputDevice)> {
        let idx = self.device_mapping.get(&handle)?;
        let (hid_dev, dev) = self.devices[*idx].as_mut()?;
        Some((hid_dev.as_mut(), dev.as_mut()))
    }

    fn get_device(&self, handle: hid::DeviceHandle) -> Option<(&hid::Device, &dyn InputDevice)> {
        let idx = self.device_mapping.get(&handle)?;
        let (hid_dev, dev) = self.devices[*idx].as_ref()?;
        Some((hid_dev.as_ref(), dev.as_ref()))
    }

    fn get_device_type(&self, handle: hid::DeviceHandle) -> DeviceType {
        self.get_device(handle).map_or(DeviceType::Other("<unknown>".to_onca_string()), |(_, dev)| dev.get_device_type())
    }

    fn get_axis_value(&self, handle: hid::DeviceHandle, path: &InputAxisId) -> Option<AxisValue> {
        let (_, dev) = self.get_device(handle)?;
        dev.get_axis_value(path)
    }

    fn has_device(&self, handle: hid::DeviceHandle) -> bool {
        self.device_mapping.contains_key(&handle)
    }

    fn add_device(&mut self, hid_dev: HeapPtr<hid::Device>, dev: HeapPtr<dyn InputDevice>) {
        let handle = hid_dev.handle();
        let idx = match self.devices.iter().position(|opt| opt.is_none()) {
            Some(idx) => {
                self.devices[idx] = Some((hid_dev, dev));
                idx
            },
            None => {
                let idx = self.devices.len();
                self.devices.push(Some((hid_dev, dev)));
                idx
            },
        };
        self.device_mapping.insert(handle, idx);
    }

    fn remove_device(&mut self, handle: hid::DeviceHandle) {
        if let Some(idx) = self.device_mapping.remove(&handle) {
            self.devices[idx] = None;
        }
    }

    fn tick(&mut self, dt: f32, rebind_notify: &mut dyn FnMut(InputAxisId)) {
        for opt in &mut self.devices {
            if let Some((_, dev)) = opt {
                dev.tick(dt, rebind_notify);
            }
        }
    }

    fn handle_hid_input(&mut self, handle: hid::DeviceHandle, raw_report: &[u8]) {
        if let Some((hid_dev, input_dev)) = self.get_device_mut(handle) {
            let input_report = unsafe { hid::InputReport::from_raw_slice(raw_report, &hid_dev) };
            input_dev.handle_hid_input(&*hid_dev, input_report);
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
    rebind_callback : HeapPtr<dyn Fn(InputAxisId) -> RebindResult>,
}

type CreateDevicePtr = HeapPtr<dyn Fn() -> Option<HeapPtr<dyn InputDevice>>>;

/// Manager for all input: devices, bindings, events, etc
/// 
/// All processing for the input manager is handled by the main thread
pub struct InputManager {
    pub(crate) os_input     : os::OSInput,
    pub(crate) mouse        : Option<Mouse>,
    pub(crate) keyboard     : Option<Keyboard>,
    device_store            : RwLock<DeviceStorage>,
    raw_input_listener      : Arc<Mutex<RawInputListener>>,

    device_product_creators : Mutex<HashMap<hid::VendorProduct, CreateDevicePtr>>,
    device_custom_creators  : Mutex<DynArray<(HeapPtr<dyn Fn(&hid::Identifier) -> bool>, CreateDevicePtr)>>,
    device_usage_creators   : Mutex<HashMap<hid::Usage, CreateDevicePtr>>,

    mapping_contexts        : Mutex<DynArray<MappingContext>>,

    control_schemes         : RwLock<DynArray<ControlScheme>>,
    users                   : RwLock<DynArray<User>>,

    unused_devices          : DynArray<DeviceHandle>,

    rebind_context          : Option<RebindContext>
}

impl InputManager {
    pub fn new(window_manager: &HeapPtr<WindowManager>) -> HeapPtr<Self> {
        assert!(sys::is_on_main_thread(), "The input manager should only be created on the main thread");

        let mut ptr = HeapPtr::new(Self {
            os_input: os::OSInput::new(),
            mouse: None,
            keyboard: None,
            device_store: RwLock::new(DeviceStorage::new()),
            raw_input_listener: Arc::new(Mutex::new(RawInputListener::new())),
            device_product_creators: Mutex::new(HashMap::new()),
            device_custom_creators: Mutex::new(DynArray::new()),
            device_usage_creators: Mutex::new(HashMap::new()),
            mapping_contexts: Mutex::new(DynArray::new()),
            control_schemes: RwLock::new(DynArray::new()),
            users: RwLock::new(DynArray::new()),
            unused_devices: DynArray::new(),
            rebind_context: None,
        });
        ptr.raw_input_listener.lock().init(&ptr);

        window_manager.register_raw_input_listener(ptr.raw_input_listener.clone());

        // Try to register all devices we can
        ptr.init_devices();

        // Make sure that there is 1 user
        ptr.users.write().resize_with(1, || User::new());

        ptr
    }

    /// Register an input device creator for a specific device, including the hid usages it needs to have registered
    pub fn register_product_create_device<F>(&self, vendor_product: hid::VendorProduct, usages: &[hid::Usage], create_dev: F)
    where
        F : Fn() -> Option<HeapPtr<dyn InputDevice>> + 'static
    {
        {
            let mut dev_prod_creators = self.device_product_creators.lock();
            if !dev_prod_creators.contains_key(&vendor_product) {
                dev_prod_creators.insert(vendor_product, HeapPtr::new(create_dev));
            } else {
                log_warning!(LOG_INPUT_CAT, "Device creation has already been registered for vendor and product: {vendor_product}");
            }
        }
        for usage in usages {
            self.os_input.register_device_usage(*usage);
        }
    }

    /// Register an input device createor for custom logic, e.g. controllers that support a custom API
    pub fn register_custom_create_device<P, F>(&self, pred: P, usages: &[hid::Usage], create_dev: F)
    where
        P : Fn(&hid::Identifier) -> bool + 'static,
        F : Fn() -> Option<HeapPtr<dyn InputDevice>> + 'static
    {
        {
            let mut dev_custom_creators = self.device_custom_creators.lock();
            dev_custom_creators.push((HeapPtr::new(pred), HeapPtr::new(create_dev)));
        }
        for usage in usages {
            self.os_input.register_device_usage(*usage);
        }
    }
    
    /// Register a generic input device creator for a specific usage
    pub fn register_usage_creator_device<F>(&self, usage: hid::Usage, create_dev: F)
    where
    F : Fn() -> Option<HeapPtr<dyn InputDevice>> + 'static
    {
        {
            let mut usage_creators = self.device_usage_creators.lock();
            if !usage_creators.contains_key(&usage) {
                usage_creators.insert(usage, HeapPtr::new(create_dev));
            } else {
                log_warning!(LOG_INPUT_CAT, "Device creation has already been registered for usage: {usage}");
            }
        }
        self.os_input.register_device_usage(usage);
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
        if self.rebind_context.is_some() {
            log_warning!(LOG_INPUT_CAT, "Trying to start rebind when rebind is still in progress");
            return;
        }

        self.rebind_context = Some(RebindContext {
            binding_name: binding_name.to_onca_string(),
            context_name: mapping_context_identifier.map(|s| s.to_onca_string()),
            rebind_callback: HeapPtr::new(rebind_callback)
        })
    }
  
    pub fn tick(&mut self, dt: DeltaTime) {
        assert!(sys::is_on_main_thread(), "The input manager should only be ticked on the main thread");

        let _scope_alloc = ScopedAlloc::new(UseAlloc::TlsTemp);
        
        // Update devices
        let mut rebind_axes = DynArray::new();
        let mut callback = |input: InputAxisId| if self.rebind_context.is_none() { rebind_axes.push(input) };
        
        if let Some(mouse) = &mut self.mouse {
            mouse.tick(dt.get_dt(), &mut callback);
        }
        if let Some(kb) = &mut self.keyboard {
            kb.tick(dt.get_dt(), &mut callback);
        }
        self.device_store.write().tick(dt.get_dt(), &mut callback);
        
        for axis in rebind_axes {
            self.notify_rebind(axis);
        }

        let device_store = self.device_store.read();
        let mut users = self.users.write();
        if users.len() != 1 {
            let schemes = self.control_schemes.read();
            // If there are users that don't have a control set, try to generate them
            for (user_idx, user) in users.iter_mut().enumerate() {
                if !self.unused_devices.is_empty() && user.control_set().is_none() {
                    for scheme in &*schemes {
                        // The first control scheme that can be created will be used
                        if let Some(control_set) = scheme.create_control_set(&self.unused_devices, |handle| self.get_device_type(handle)) {
                            self.unused_devices.retain(|handle| !control_set.devices().contains(handle));
                            user.set_control_set(control_set);
                            break;
                        }
                    }
                }
                user.process_input(dt, user_idx as u8, |user, axis| self.get_input_for_user(user, axis, &device_store));
            }
        } else {
            assert!(users.len() == 1);
            users[0].process_input(dt, 0, |_, axis| self.get_input_for_any(axis, &device_store));
        }
    }

    pub fn keyboard(&self) -> Option<&Keyboard> {
        self.keyboard.as_ref()
    }

    pub fn mouse(&self) -> Option<&Mouse> {
        self.mouse.as_ref()
    }

    /// Set the maximum number of users that can be created.4
    /// 
    /// If `1` is passed, all input devices will be consumed by user 0, regardless of control scheme.
    /// If more than `1` is passed, each user will only ever have a single active control scheme, which cannot be switched without removing the user first.
    pub fn set_max_users(&mut self, max_users: NonZeroU8) {
        self.users.write().resize_with(max_users.get() as usize, || User::new());
    }

    
    /// Add a new possible control scheme to this user
    pub fn add_control_scheme(&mut self, scheme: ControlScheme) {
        self.control_schemes.write().push(scheme)
    }

    /// Remove a control scheme
    pub fn remove_control_scheme(&mut self, identifier: &ControlSchemeID) {
        let mut control_schemes = self.control_schemes.write();
        if let Some(idx) = control_schemes.iter().position(|scheme| scheme.identifier() == identifier) {
            control_schemes.remove(idx);
            for user in &mut *self.users.write() {
                user.notify_scheme_removed(identifier);
            }
        }
    }

    pub(crate) fn has_device(&mut self, hid_handle: hid::DeviceHandle) -> bool {
        self.device_store.read().has_device(hid_handle)
    }

    pub(crate) fn can_create_device_for(&self, hid_iden: hid::Identifier) -> bool {
        self.device_product_creators.lock().contains_key(&hid_iden.vendor_device) ||
            self.device_usage_creators.lock().contains_key(&hid_iden.usage)
    }

    pub(crate) fn add_device(&mut self, device: hid::Device) {
        let device = HeapPtr::new(device);
        let handle = DeviceHandle::Hid(device.handle());
        if let Some(input_dev) = self.create_input_device_for(*device.identifier()) {
            let usage = device.identifier().usage;
            let vendor_product = device.get_vendor_string().unwrap_or_default() + " " + &device.get_product_string().unwrap_or_default();

            log_info!(LOG_INPUT_CAT, "Added new input device '{vendor_product}' for usage {usage}");

            self.device_store.write().add_device(device, input_dev);

            // First check if any user had this device disconnected, if so, give it the device and try to create the control scheme
            for user in &mut *self.users.write() {
                if let None = user.control_set() && user.try_reconnect_device(handle) {
                    return;
                }
            }
            // Otherwise, add to the available devices
            self.unused_devices.push(handle)
        }
    }

    pub(crate) fn handle_hid_input(&self, handle: hid::DeviceHandle, raw_report: &[u8]) {
        self.device_store.write().handle_hid_input(handle, raw_report);
    }

    pub(crate) fn remove_device(&self, handle: hid::DeviceHandle) {
        self.device_store.write().remove_device(handle);

        let mut users = self.users.write();
        for (idx, user) in users.iter_mut().enumerate() {
            if user.notify_device_removed(DeviceHandle::Hid(handle)) {
                if user.get_currently_held_devices().is_empty() {
                    // We can safely remove the value at the index and invalidate the iterator, as we don't use it anymore
                    users.remove(idx);
                }

                // Only one user can hold a device, so we don't need to check any other ones
                break;
            }
        }
    }

    pub(crate) fn notify_rebind(&mut self, input: InputAxisId) {
        if let Some(ctx) = &mut self.rebind_context {
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
                RebindResult::Cancel => self.rebind_context = None,
            }
        }
    }

    fn init_devices(&mut self) {
        self.mouse = Mouse::new();
        self.keyboard = Keyboard::new();

        // Register built-in device creators
        let gamepad_usage = hid::Usage::from_u16(1, 5);
        self.register_usage_creator_device(gamepad_usage, || Gamepad::new().map(|x| {
            // We need to get around rust not realizing that `HeapPtr` could `CoerseUnsized` directly in a return statement
            // This could be one of those "std::boxed::Box is special" cases, as the first line clearly shows that it works
            let res : HeapPtr<dyn InputDevice> = HeapPtr::new(x);
            res
        }));
    }

    fn create_input_device_for(&self, ident: hid::Identifier) -> Option<HeapPtr<dyn InputDevice>> {
        // Find the best fitting, registered device
        //
        // 1) any device that matches the specific vendor and product
        if let Some(create) = self.device_product_creators.lock().get(&ident.vendor_device) {
            if let Some(ptr) = create() {
                return Some(ptr);
            }
        }

        // 2) any device that matches custom logic
        if let Some(create) = self.device_custom_creators.lock().iter().find(|(fun, _)| fun(&ident)) {
            if let Some(ptr) = create.1() {
                return Some(ptr);
            }
        }

        // 3) any device that matches the usage
        if let Some(create) = self.device_usage_creators.lock().get(&ident.usage) {
            if let Some(ptr) = create() {
                return Some(ptr);
            }
        }

        // If we have not match for either a product or usage, we don't know about this input device
        log_verbose!(LOG_INPUT_CAT, "Trying to create an input device for an unknown hid product/usage '{ident}', ignoring this device");
        None
    }
    
    fn get_input_for_any(&self, axis_path: &InputAxisId, device_store: &DeviceStorage) -> AxisValue {
        // try the keyboard first
        let kb_input = match &self.keyboard {
            Some(kb) => kb.get_axis_value(axis_path),
            None => None,
        };
        if let Some(kb_input) = kb_input {
            return kb_input;
        }
        // then try the mouse
        let mouse_input = match &self.mouse {
            Some(mouse) => mouse.get_axis_value(axis_path),
            None => None,
        };
        if let Some(mouse_input) = mouse_input {
            return mouse_input;
        }
        // then try the other input devices
        for opt in &device_store.devices {
            if let Some((_, dev)) = opt {
                if let Some(val) = dev.get_axis_value(axis_path) {
                    return val;
                }
            }
        }
        // If we don't have a value, return a false value
        AxisValue::Digital(false)
    }

    fn get_input_for_user(&self, user: &User, axis_path: &InputAxisId, device_store: &DeviceStorage) -> AxisValue {
        match user.control_set() {
            Some(control_set) => {
                for handle in control_set.devices() {
                    if let Some(val) = self.get_axis_value_from_handle(*handle, axis_path, device_store) {
                        return val;
                    }
                }
                AxisValue::Digital(false)
            },
            None => AxisValue::Digital(false),
        }
    }

    fn get_axis_value_from_handle(&self, handle: DeviceHandle, axis_path: &InputAxisId, device_store: &DeviceStorage) -> Option<AxisValue> {
        match handle {
            DeviceHandle::Invalid => None,
            DeviceHandle::Mouse => self.mouse.as_ref().and_then(|mouse| mouse.get_axis_value(axis_path)),
            DeviceHandle::Keyboard => self.keyboard.as_ref().and_then(|keyboard| keyboard.get_axis_value(axis_path)),
            DeviceHandle::Hid(handle) => device_store.get_axis_value(handle, axis_path),
        }
    }

    fn get_device_type(&self, handle: DeviceHandle) -> DeviceType {
        match handle {
            DeviceHandle::Invalid => DeviceType::Other("<unknown>".to_onca_string()),
            DeviceHandle::Mouse => DeviceType::Mouse,
            DeviceHandle::Keyboard => DeviceType::Keyboard,
            DeviceHandle::Hid(hid_handle) => self.device_store.read().get_device_type(hid_handle),
        }
    }

}

struct RawInputListener {
    manager : *mut InputManager
}

impl RawInputListener {
    pub(crate) fn new() -> Self {
        Self { manager: null_mut() }
    }

    pub(crate) fn init(&mut self, manager: &HeapPtr<InputManager>) {
        self.manager = manager.ptr_mut();
    }
}

impl EventListener<onca_window::RawInputEvent> for RawInputListener {
    fn notify(&mut self, event: &onca_window::RawInputEvent) {
        // SAFETY: `process_window_event`: We know the implementation to be safe
        // SAFETY: deref: Events can only be sent via the main thread (via the window manager), so our deref here is not causing mutability over multiple threads
        unsafe { OSInput::process_window_event(&mut *self.manager, event) };
    }
}