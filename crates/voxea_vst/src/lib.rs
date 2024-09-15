#![feature(fn_traits)]
#![allow(warnings)]
// #[repr(C)]
// #[derive(Debug, Default, Copy, Clone)]
// pub struct FUID(u32, u32, u32, u32);
//
// impl FUID {
//     pub fn to_hex(&self) -> String {
//         let mut hex_string = String::new();
//
//         // Convert each u32 field into hexadecimal and append to the string
//         let hex_string = format!("{:08X}{:08X}{:08X}{:08X}", self.0, self.1, self.2, self.3);
//
//         hex_string
//     }
// }
// #[repr(C)]
// pub struct FUnknown {
//     vtable: *const FUnknownVTable
// }
//
// impl FUnknown {
//     pub fn iid() -> [c_char; 16] {
//         inline_uid(0x00000000, 0x00000000, 0xC0000000, 0x00000046)
//     }
// }
//
// impl Into<[c_char; 16]> for FUID {
//     fn into(self) -> [c_char; 16] {
//         unsafe { std::mem::transmute::<FUID, [c_char; 16]>(self) }
//     }
// }
//

#[allow(non_snake_case)]
#[repr(C)]
struct IHostApplicationVTable {
    pub queryInterface: unsafe extern "stdcall" fn(this: *mut IHostApplication, _iid: [c_char; 16], obj: *mut *mut c_void) -> i32,
    pub addRef: unsafe extern "stdcall" fn(this: *mut IHostApplication) -> u32,
    pub release: unsafe extern "stdcall" fn(this: *mut IHostApplication) -> u32,

    pub getName: unsafe extern "stdcall" fn(this: *mut IHostApplication, name: &mut [u16; 128]) -> i32,
    pub createInstance: unsafe extern "stdcall" fn(this: *mut IHostApplication, cid: [c_char; 16], iid: [c_char; 16], obj: *mut *mut c_void) -> i32,
}

#[repr(C)]
struct IHostApplication {
    vtable: &'static IHostApplicationVTable
}

#[no_mangle]
pub extern "stdcall" fn queryInterface(this: *mut IHostApplication, iid: [c_char; 16], obj: *mut *mut c_void) -> i32 {
    warn!("IHostApplication queryInterface: {:?}", iid);
    unsafe {
        *obj = this as *mut c_void;
    }
    0
}

#[no_mangle]
pub extern "stdcall" fn addRef(this: *mut IHostApplication) -> u32 {
    100
}

#[no_mangle]
pub extern "stdcall" fn release(this: *mut IHostApplication) -> u32 {
    100
}


#[no_mangle]
pub extern "stdcall" fn getName(this: *mut IHostApplication, name: &mut [u16; 128]) -> i32 {
    warn!("IHostApplication::getName");
    name[0] = u16::try_from('N').unwrap();
    name[1] = 0;

    0
}

#[no_mangle]
pub extern "stdcall" fn createInstance(this: *mut IHostApplication, cid: [c_char; 16], iid: [c_char; 16], obj: *mut *mut c_void) -> i32 {
    unsafe {
        warn!("cid: {:?}\niid: {:?}", cid, iid);
        if iid == IMessage::iid {
            warn!("IMessage received!");

            let message = Box::new(Message::new());
            *obj = Box::into_raw(message) as *mut c_void;
        }
    }
    0
}

impl IHostApplication {
    pub fn new() -> Self {
        Self {
            vtable: &IHostApplicationVTable {
                queryInterface,
                addRef,
                release,

                getName,
                createInstance
            }
        }
    }
}

#[allow(non_snake_case)]
#[repr(C)]
struct IComponentHandlerVTable {
    pub queryInterface: unsafe extern "stdcall" fn(this: *mut IComponentHandler, iid: [c_char; 16], obj: *mut *mut c_void) -> i32,
    pub addRef: unsafe extern "stdcall" fn(this: *mut IComponentHandler) -> u32,
    pub release: unsafe extern "stdcall" fn(this: *mut IComponentHandler) -> u32,

    // pub getName: unsafe extern "stdcall" fn(this: *mut IHostApplication, name: &mut [u16; 128]) -> i32,
    // pub createInstance: unsafe extern "stdcall" fn(this: *mut IHostApplication, cid: [c_char; 16], iid: [c_char; 16], obj: *mut *mut c_void) -> i32,

    pub beginEdit: unsafe extern "stdcall" fn(this: *mut IComponentHandler, id: u32) -> TResult,
    pub performEdit : unsafe extern "stdcall" fn(this: *mut IComponentHandler, id: u32, value: u32) -> TResult,
    pub endEdit: unsafe extern "stdcall" fn(this: *mut IComponentHandler, id: u32) -> TResult,
    pub restartComponent: unsafe extern "stdcall" fn(this: *mut IComponentHandler, flags: i32) -> TResult,

    pub setDirty: unsafe extern "stdcall" fn(this: *mut IComponentHandler, state: bool) -> TResult,
    pub requestOpenEditor : unsafe extern "stdcall" fn(this: *mut IComponentHandler, name: *const c_char) -> TResult,
    pub startGroupEdit: unsafe extern "stdcall" fn(this: *mut IComponentHandler) -> TResult,
    pub finishGroupEdit: unsafe extern "stdcall" fn(this: *mut IComponentHandler) -> TResult,

    pub test1: unsafe extern "stdcall" fn(this: *mut IComponentHandler) -> TResult,
    pub test2: unsafe extern "stdcall" fn(this: *mut IComponentHandler) -> TResult,
    pub test3: unsafe extern "stdcall" fn(this: *mut IComponentHandler) -> TResult,
    pub test4: unsafe extern "stdcall" fn(this: *mut IComponentHandler) -> TResult,
}


#[no_mangle]
pub extern "stdcall" fn queryInterface2(this: *mut IComponentHandler, iid: [c_char; 16], obj: *mut *mut c_void) -> i32 {
    warn!("IComponentHandler queryInterface: {:?} {:?}", uid_to_ascii(iid), iid);
    unsafe {
        if iid == [-52, -107, -27, 88, 45, -37, 105, 73, -117, 106, -81, -116, 54, -90, 100, -27] {
            let host = Box::new(IHostApplication::new());
            *obj = Box::into_raw(host) as *mut c_void;
        } else {
            *obj = this as *mut c_void;
        }
    }
    0
}

#[no_mangle]
pub extern "stdcall" fn addRef2(this: *mut IComponentHandler) -> u32 {
    1000
}

#[no_mangle]
pub extern "stdcall" fn release2(this: *mut IComponentHandler) -> u32 {
    1000
}

#[no_mangle]
pub extern "stdcall" fn beginEdit(this: *mut IComponentHandler, id: u32) -> TResult {
    warn!("IComponentHandler::beginEdit");
    TResult::ResultOk
}

#[no_mangle]
pub extern "stdcall" fn performEdit(this: *mut IComponentHandler, id: u32, value: u32) -> TResult {
    warn!("IComponentHandler::performEdit");
    TResult::ResultOk
}

#[no_mangle]
pub extern "stdcall" fn endEdit(this: *mut IComponentHandler, id: u32) -> TResult {
    warn!("IComponentHandler::endEdit");
    TResult::ResultOk
}

#[no_mangle]
pub extern "stdcall" fn restartComponent(this: *mut IComponentHandler, flags: i32) -> TResult {
    warn!("IComponentHandler::restartComponent");
    TResult::ResultOk
}

#[no_mangle]
pub extern "stdcall" fn setDirty(this: *mut IComponentHandler, state: bool) -> TResult {
    warn!("IComponentHandler::setDirty");
    TResult::ResultOk
}

#[no_mangle]
pub extern "stdcall" fn requestOpenEditor(this: *mut IComponentHandler, name: *const c_char) -> TResult {
    unsafe {
        warn!("requestOpenEditor: {:?}", CStr::from_ptr(name).to_str().unwrap());
    }
    TResult::ResultOk
}

#[no_mangle]
pub extern "stdcall" fn startGroupEdit(this: *mut IComponentHandler) -> TResult {
    warn!("IComponentHandler::startGroupEdit");
    TResult::ResultOk
}

#[no_mangle]
pub extern "stdcall" fn finishGroupEdit(this: *mut IComponentHandler) -> TResult {
    warn!("IComponentHandler::finishGroupEdit");
    TResult::ResultOk
}

#[no_mangle]
pub extern "stdcall" fn test(this: *mut IComponentHandler) -> TResult {
    warn!("IComponentHandler::test");
    TResult::ResultOk
}

#[repr(C)]
struct IComponentHandler {
    vtable: &'static crate::IComponentHandlerVTable
}

impl IComponentHandler {
    pub fn new() -> Self {
        Self {
            vtable: &IComponentHandlerVTable {
                queryInterface: queryInterface2,
                addRef: addRef2,
                release: release2,

                beginEdit,
                performEdit,
                endEdit,
                restartComponent,

                setDirty,
                requestOpenEditor,
                startGroupEdit,
                finishGroupEdit,

                test1: test,
                test2: test,
                test3: test,
                test4: test,
            }
        }
    }
}


fn uid_to_ascii(uid: [c_char; 16]) -> String {
    // Step 1: Convert [u8; 16] to a hex string (32 characters long)
    let hex_string = uid.iter()
        .map(|byte| format!("{:02X}", byte))  // Format each byte as 2 hex digits
        .collect::<String>();

    let formatted_uid = format!(
        "{}{}{}{}{}{}{}{}{}",
        &hex_string[0..8],
        "-",
        &hex_string[8..12],
        "-",
        &hex_string[12..16],
        "-",
        &hex_string[16..20],
        "-",
        &hex_string[20..32]
    );

    formatted_uid

    // // Step 2: Convert the hex string into [u8; 32] of ASCII values
    // let mut ascii_array = [0u8; 37];
    // for (i, c) in formatted_uid.chars().enumerate() {
    //     ascii_array[i] = c as u8;  // Convert each char to its ASCII value
    // }
    //
    // ascii_array[36] = 0;
    //
    // ascii_array
}

use std::ffi::{c_void, CStr, CString};
use std::ptr::null_mut;
use std::sync::{Arc, OnceLock};
use libc::c_char;
use libloading::{Library, Symbol};
use log::{info, warn};
use crate::base::funknown::{FUnknown, FUnknown_Impl, IAudioProcessor, IComponent, IEditController, IPluginBase, IPluginFactory, IAudioProcessor_Impl, IComponent_Impl, IEditController_Impl, IPluginBase_Impl, IPluginFactory_Impl, Interface, PClassInfo, PFactoryInfo, TResult, FUID, IPlugView, IPlugView_Impl, IMessage, IMessage_Impl, IConnectionPoint, IConnectionPoint_Impl, IPlugViewContentScaleSupport};

pub mod base;

type InitDllProc = fn() -> bool;
type ExitDllProc = fn() -> bool;
type GetPluginFactoryProc = fn() -> *mut IPluginFactory;

pub struct VSTHostContext {
    pub lib: Option<Library>,
    pub factory: *mut IPluginFactory,
    pub host: *mut IComponentHandler,
    pub comp: *mut IComponent,
    pub edit: *mut IEditController,
    pub view: *mut IPlugViewContentScaleSupport
}

impl VSTHostContext {
    pub fn attach(&self, hwnd: isize) {
        unsafe {
            let ty = CStr::from_bytes_with_nul_unchecked(b"HWND\0");

            (*(self.view)).attached(hwnd as *mut c_void, ty.as_ptr());
        }
    }
}

impl Drop for VSTHostContext {
    fn drop(&mut self) {
        unsafe {
            warn!("Closing VST Host!");
            let mut lib = self.lib.take().unwrap();
            let exit: Symbol<ExitDllProc> = lib.get(b"ExitDll").unwrap();
            exit.call(());

            lib.close().unwrap();
        }
    }
}

impl Default for VSTHostContext {
    fn default() -> Self {
        Self {
            lib: None,
            factory: std::ptr::null_mut(),
            host: std::ptr::null_mut(),
            comp: std::ptr::null_mut(),
            edit: std::ptr::null_mut(),
            view: std::ptr::null_mut(),
        }
    }
}

static mut CONTEXT: OnceLock<Arc<VSTHostContext>> = OnceLock::new();

pub fn load_vst(plug: u32) -> Arc<VSTHostContext> {
    unsafe {
        let mut ctx = VSTHostContext::default();

        let path = match plug {
            1 => "C:/Coding/RustRover/voxea/vst3/Archetype Nolly.vst3",
            2 => "C:/Coding/RustRover/voxea/vst3/LABS.vst3",
            3 => "C:/Coding/RustRover/voxea/vst3/Healing 2 Lite.vst3",

            _ => unimplemented!()
        };

        let lib = Library::new(path).unwrap();
        let init: Symbol<InitDllProc> = lib.get(b"InitDll").unwrap();
        init.call(());

        let raw_factory: Symbol<GetPluginFactoryProc> = lib.get::<GetPluginFactoryProc>(b"GetPluginFactory").unwrap();
        let raw_factory: *mut IPluginFactory = raw_factory.call(());

        ctx.factory = raw_factory;

        let factory = &mut *raw_factory;

        println!("{}", factory.count_classes());

        let mut factory_info = PFactoryInfo::default();
        factory.get_factory_info(&mut factory_info);

        println!("{}", factory_info);

        let mut host = Arc::new(IComponentHandler::new());

        for i in 0..factory.count_classes() {
            let mut class_info = PClassInfo::default();
            factory.get_class_info(i, &mut class_info);

            if class_info.category() != "Audio Module Class" {
                continue;
            }

            let mut comp: *mut c_void = std::ptr::null_mut();
            let ret = factory.create_instance(class_info.cid, IComponent::iid, &mut comp);

            ctx.comp = comp as *mut IComponent;

            let comp: &mut IComponent = &mut *(comp as *mut IComponent);
            if ret != TResult::ResultOk {
                break;
            }

            let foo_c = host.clone();
            let context = Arc::into_raw(foo_c) as *mut FUnknown;
            println!("Context: {:?}", context as *mut _);
            let res = comp.initialize(context);
            comp.set_active(true);
            let mut edit_cid = FUID::default();

            let res = comp.get_controller_class_id(&mut edit_cid);
            let mut edit: *mut c_void = std::ptr::null_mut();

            if res != TResult::ResultOk {
                let res = comp.query_interface(IEditController::iid, &mut edit);
                warn!("Initializing query_interface! {:?}", res);
            } else {
                let res = factory.create_instance(edit_cid, IEditController::iid, &mut edit);
                warn!("Initializing create_instance! {:?}", res);
            }
            ctx.edit = edit as *mut IEditController;

            let mut edit: &mut IEditController = &mut *(edit as *mut IEditController);

            let res = edit.initialize(context);

            edit.set_component_handler(context as *mut c_void);

            let mut iConnectionPointComponent: *mut c_void = std::ptr::null_mut();
            let mut iConnectionPointController: *mut c_void = std::ptr::null_mut();
            warn!("{:?}", comp.query_interface(IConnectionPoint::iid, &mut iConnectionPointComponent));
            warn!("{:?}", edit.query_interface(IConnectionPoint::iid, &mut iConnectionPointController));

            let mut iConnectionPointComponent: &mut IConnectionPoint = &mut *(iConnectionPointComponent as *mut IConnectionPoint);
            let mut iConnectionPointController: &mut IConnectionPoint = &mut *(iConnectionPointController as *mut IConnectionPoint);

            iConnectionPointComponent.connect(iConnectionPointController);
            iConnectionPointController.connect(iConnectionPointComponent);

            warn!("Parameter count: {}", edit.get_parameter_count());
            let name = CStr::from_bytes_with_nul_unchecked(b"editor\0");

            let mut view = edit.create_view(ViewType::Editor);
            // (*(view as *mut FUnknown)).query_interface(IPlugViewContentScaleSupport::iid, &mut view);

            ctx.view = view as *mut IPlugViewContentScaleSupport;
            let mut view: &mut IPlugViewContentScaleSupport = &mut *(view as *mut IPlugViewContentScaleSupport);
            let ty = CStr::from_bytes_with_nul_unchecked(b"HWND\0");
            info!("View: {:?}", view);
            // view.attached(window as *mut c_void, ty.as_ptr());


            println!("{} {:?} {:?} {:?} {:?}", class_info, comp as *mut _, edit as *mut _, context as *mut _, view);
        }

        // factory.release();

        // warn!("Closing dll!");
        //
        ctx.lib = Some(lib);
        // CONTEXT.get_or_init(|| Arc::new(ctx));

        // let exit: Symbol<ExitDllProc> = lib.get(b"ExitDll").unwrap();
        // exit.call(());

        // lib.close().unwrap();

        Arc::new(ctx)
    }
}

mod ViewType {
    use std::ffi::{c_char, CStr};
    pub const Editor: *const c_char = unsafe { CStr::from_bytes_with_nul_unchecked(b"editor\0") }.as_ptr();
}

//
// pub fn load_vst_window(window: isize, plug: u32) {
//     unsafe {
//         let mut ctx = VSTHostContext::default();
//
//         let path = match plug {
//             1 => "C:/Coding/RustRover/voxea/vst3/Archetype Nolly.vst3",
//             2 => "C:/Coding/RustRover/voxea/vst3/LABS.vst3",
//             3 => "C:/Coding/RustRover/voxea/vst3/Healing 2 Lite.vst3",
//
//             _ => unimplemented!()
//         };
//
//         let lib = Library::new(path).unwrap();
//         let init: Symbol<InitDllProc> = lib.get(b"InitDll").unwrap();
//         init.call(());
//
//         let raw_factory: Symbol<GetPluginFactoryProc> = lib.get::<GetPluginFactoryProc>(b"GetPluginFactory").unwrap();
//         let raw_factory: *mut IPluginFactory = raw_factory.call(());
//
//         ctx.factory = raw_factory;
//
//         let factory = &mut *raw_factory;
//
//         println!("{}", factory.count_classes());
//
//         let mut factory_info = PFactoryInfo::default();
//         factory.get_factory_info(&mut factory_info);
//
//         println!("{}", factory_info);
//
//         let mut host = Arc::new(IComponentHandler::new());
//
//         for i in 0..factory.count_classes() {
//             let mut class_info = PClassInfo::default();
//             factory.get_class_info(i, &mut class_info);
//
//             if class_info.category() != "Audio Module Class" {
//                 continue;
//             }
//
//             let mut comp: *mut c_void = std::ptr::null_mut();
//             let ret = factory.create_instance(class_info.cid, IComponent::iid, &mut comp);
//
//             ctx.comp = comp as *mut IComponent;
//
//             let comp: &mut IComponent = &mut *(comp as *mut IComponent);
//             if ret != TResult::ResultOk {
//                 break;
//             }
//
//             let foo_c = host.clone();
//             let context = Arc::into_raw(foo_c) as *mut FUnknown;
//             println!("Context: {:?}", context as *mut _);
//             // let res = comp.initialize(context);
//             // comp.set_active(true);
//             let mut edit_cid = FUID::default();
//
//             let res = comp.get_controller_class_id(&mut edit_cid);
//             let mut edit: *mut c_void = std::ptr::null_mut();
//
//             if res != TResult::ResultOk {
//                 let res = std::mem::transmute::<&mut IComponent, &mut FUnknown>(comp).query_interface(IEditController::iid, &mut edit);
//                 warn!("Initializing query_interface! {:?}", res);
//             } else {
//                 let res = factory.create_instance(edit_cid, IEditController::iid, &mut edit);
//                 warn!("Initializing create_instance! {:?}", res);
//             }
//             ctx.edit = edit as *mut IEditController;
//
//             let mut edit: &mut IEditController = &mut *(edit as *mut IEditController);
//
//             let res = edit.initialize(context);
//
//             edit.set_component_handler(context as *mut c_void);
//
//             let mut iConnectionPointComponent: *mut c_void = std::ptr::null_mut();
//             let mut iConnectionPointController: *mut c_void = std::ptr::null_mut();
//             warn!("{:?}", std::mem::transmute::<&mut IComponent, &mut FUnknown>(comp).query_interface(IConnectionPoint::iid, &mut iConnectionPointComponent));
//             warn!("{:?}", std::mem::transmute::<&mut IEditController, &mut FUnknown>(edit).query_interface(IConnectionPoint::iid, &mut iConnectionPointController));
//
//             let mut iConnectionPointComponent: &mut IConnectionPoint = &mut *(iConnectionPointComponent as *mut IConnectionPoint);
//             let mut iConnectionPointController: &mut IConnectionPoint = &mut *(iConnectionPointController as *mut IConnectionPoint);
//
//             iConnectionPointComponent.connect(iConnectionPointController);
//             iConnectionPointController.connect(iConnectionPointComponent);
//
//             warn!("Parameter count: {}", edit.get_parameter_count());
//             let name = CStr::from_bytes_with_nul_unchecked(b"editor\0");
//
//             let view = edit.create_view(ViewType::Editor);
//             ctx.view = view as *mut IPlugView;
//             let mut view: &mut IPlugView = &mut *(view as *mut IPlugView);
//             let ty = CStr::from_bytes_with_nul_unchecked(b"HWND\0");
//             info!("View: {:?}", view);
//             view.attached(window as *mut c_void, ty.as_ptr());
//
//
//             println!("{} {:?} {:?} {:?} {:?}", class_info, comp as *mut _, edit as *mut _, context as *mut _, view);
//         }
//
//         // factory.release();
//
//         // warn!("Closing dll!");
//         //
//         ctx.lib = Some(lib);
//         CONTEXT.get_or_init(|| Arc::new(ctx));
//
//         // let exit: Symbol<ExitDllProc> = lib.get(b"ExitDll").unwrap();
//         // exit.call(());
//
//         // lib.close().unwrap();
//     }
// }
//
//
//
//













#[allow(non_snake_case)]
#[repr(C)]
struct MessageVTable {
    pub queryInterface: unsafe extern "stdcall" fn(this: *mut Message, _iid: [c_char; 16], obj: *mut *mut c_void) -> i32,
    pub addRef: unsafe extern "stdcall" fn(this: *mut Message) -> u32,
    pub release: unsafe extern "stdcall" fn(this: *mut Message) -> u32,
    pub get_message_id: unsafe extern "stdcall" fn(this: *mut Message) -> *const c_char,
    pub set_message_id: unsafe extern "stdcall" fn(this: *mut Message, id: *const c_char) -> (),
    pub get_attributes: unsafe extern "stdcall" fn(this: *mut Message) -> *mut c_void,

    pub connect2: unsafe extern "stdcall" fn(this: *mut Message, other: *mut IConnectionPoint) -> TResult,
    pub disconnect2: unsafe extern "stdcall" fn(this: *mut Message, other: *mut IConnectionPoint) -> TResult,
    pub notify2: unsafe extern "stdcall" fn(this: *mut Message, message: *const c_char) -> TResult,
}


#[no_mangle]
pub extern "stdcall" fn queryInterface3(this: *mut Message, iid: [c_char; 16], obj: *mut *mut c_void) -> i32 {
    warn!("Message queryInterface: {:?}", iid);
    unsafe {
        *obj = this as *mut c_void;
    }
    0
}

#[no_mangle]
pub extern "stdcall" fn addRef3(this: *mut Message) -> u32 {
    1000
}

#[no_mangle]
pub extern "stdcall" fn release3(this: *mut Message) -> u32 {
    warn!("unref!");
    1000
}

#[no_mangle]
pub extern "stdcall" fn get_message_id(this: *mut Message) -> *const c_char {
    unsafe {
        let msg = CStr::from_ptr((*this).message).to_str().unwrap();

        warn!("Get Message Id: {:?}", msg);

        if msg == "ready" {
            CStr::from_bytes_with_nul_unchecked(b"ready\0").as_ptr()
        } else {
            (*this).message
        }

        // CStr::from_bytes_with_nul_unchecked(b"init\0").as_ptr()

    }
}

#[no_mangle]
pub extern "stdcall" fn set_message_id(this: *mut Message, id: *const c_char) -> () {
    unsafe {
        (*this).message = id;
        warn!("Set Message Id: {:?}", CStr::from_ptr(id).to_str().unwrap());
    }
}

#[no_mangle]
pub extern "stdcall" fn get_attributes(this: *mut Message) -> *mut c_void {
    warn!("get_attributes");

    let message = Box::new([AttributeList::new(), AttributeList::new(), AttributeList::new(), AttributeList::new(), AttributeList::new(), AttributeList::new(), AttributeList::new(), AttributeList::new(), AttributeList::new(), AttributeList::new(), AttributeList::new(), AttributeList::new()]);
    Box::into_raw(message) as *mut c_void
}

#[no_mangle]
pub extern "stdcall" fn connect2(this: *mut Message, other: *mut IConnectionPoint) -> TResult {
    warn!("connect: {:?}", other as *mut _);
    TResult::ResultOk
}

#[no_mangle]
pub extern "stdcall" fn disconnect2(this: *mut Message, other: *mut IConnectionPoint) -> TResult {
    warn!("disconnect: {:?}", other as *mut _);
    TResult::ResultOk
}

#[no_mangle]
pub extern "stdcall" fn notify2(this: *mut Message, message: *const c_char) -> TResult {
    unsafe {
        warn!("notify: {:?}", CStr::from_ptr(message).to_str().unwrap());
    }
    TResult::ResultOk
}

#[repr(C)]
struct Message {
    vtable: &'static MessageVTable,
    message: *const c_char
}

impl Message {
    pub fn new() -> Self {
        Self {
            vtable: &MessageVTable {
                queryInterface: queryInterface3,
                addRef: addRef3,
                release: release3,

                get_message_id,
                set_message_id,
                get_attributes,

                connect2,
                disconnect2,
                notify2
            },
            message: std::ptr::null_mut()
        }
    }
}

#[allow(non_snake_case)]
#[repr(C)]
struct AttributeListVTable {
    pub queryInterface: unsafe extern "stdcall" fn(this: *mut AttributeList, _iid: [c_char; 16], obj: *mut *mut c_void) -> i32,
    pub addRef: unsafe extern "stdcall" fn(this: *mut AttributeList) -> u32,
    pub release: unsafe extern "stdcall" fn(this: *mut AttributeList) -> u32,

    pub set_int: unsafe extern "stdcall" fn(this: *mut AttributeList, id: *const c_char, value: i64) -> TResult,
    pub get_int: unsafe extern "stdcall" fn(this: *mut AttributeList, id: *const c_char, value: *mut i64) -> TResult,

    pub set_float: unsafe extern "stdcall" fn(this: *mut AttributeList, id: *const c_char, value: *mut i64) -> TResult,
    pub get_float: unsafe extern "stdcall" fn(this: *mut AttributeList, id: *const c_char, value: *mut f64) -> TResult,

    pub set_string: unsafe extern "stdcall" fn(this: *mut AttributeList, id: *const c_char, value: *mut i64) -> TResult,
    pub get_string: unsafe extern "stdcall" fn(this: *mut AttributeList, id: *const c_char, value: *mut i64) -> TResult,

    pub set_binary: unsafe extern "stdcall" fn(this: *mut AttributeList, id: *const c_char, value: *mut i64) -> TResult,
    pub get_binary: unsafe extern "stdcall" fn(this: *mut AttributeList, id: *const c_char, value: *mut i64) -> TResult,

    pub bro: unsafe extern "stdcall" fn(this: *mut AttributeList, id: *const c_char, value: *mut i64) -> TResult,
}


#[no_mangle]
pub extern "stdcall" fn queryInterface4(this: *mut AttributeList, iid: [c_char; 16], obj: *mut *mut c_void) -> i32 {
    warn!("AttributeList queryInterface: {:?}", iid);
    unsafe {
        *obj = this as *mut c_void;
    }
    0
}

#[no_mangle]
pub extern "stdcall" fn addRef4(this: *mut AttributeList) -> u32 {
    1000
}

#[no_mangle]
pub extern "stdcall" fn release4(this: *mut AttributeList) -> u32 {
    1000
}

#[no_mangle]
pub extern "stdcall" fn set_int(this: *mut AttributeList, id: *const c_char, value: i64) -> TResult {
    unsafe {
        warn!("set_int: {:?} {:?}", CStr::from_ptr(id).to_str().unwrap(), value);
        if CStr::from_ptr(id).to_str().unwrap() == "__dpf_msg_target__" {
            (*this).target = value;
        } else {

        }
        TResult::ResultOk
    }
}

#[no_mangle]
pub extern "stdcall" fn get_int(this: *mut AttributeList, id: *const c_char, value: *mut i64) -> TResult {
    unsafe {
        warn!("get_int: {:?} {:?}", CStr::from_ptr(id).to_str().unwrap(), value);
        if CStr::from_ptr(id).to_str().unwrap() == "__dpf_msg_target__" {
            warn!("SETTING TO: {:?}", (*this).target);
            *value = (*this).target;
        } else {
            *value = 10;
        }

        TResult::ResultOk
    }
}

#[no_mangle]
pub extern "stdcall" fn set_float(this: *mut AttributeList, id: *const c_char, value: *mut i64) -> TResult {
    warn!("set_float: {:?} {:?}", id, value);
    unsafe {
        // *value = 1;
        TResult::ResultOk
    }
}

#[no_mangle]
pub extern "stdcall" fn get_float(this: *mut AttributeList, id: *const c_char, value: *mut f64) -> TResult {
    unsafe {
        warn!("get_float: {:?} {:?}", CStr::from_ptr(id).to_str().unwrap(), value);
        *value = 44100.0;
        TResult::ResultOk
    }
}

#[no_mangle]
pub extern "stdcall" fn set_string(this: *mut AttributeList, id: *const c_char, value: *mut i64) -> TResult {
    warn!("set_string: {:?} {:?}", id, value);
    unsafe {
        // *value = 1;
        TResult::ResultOk
    }
}

#[no_mangle]
pub extern "stdcall" fn get_string(this: *mut AttributeList, id: *const c_char, value: *mut i64) -> TResult {
    warn!("get_string: {:?} {:?}", id, value);
    unsafe {
        // *value = 1;
        TResult::ResultOk
    }
}

#[no_mangle]
pub extern "stdcall" fn set_binary(this: *mut AttributeList, id: *const c_char, value: *mut i64) -> TResult {
    warn!("set_binary: {:?} {:?}", id, value);
    unsafe {
        // *value = 1;
        TResult::ResultOk
    }
}

#[no_mangle]
pub extern "stdcall" fn get_binary(this: *mut AttributeList, id: *const c_char, value: *mut i64) -> TResult {
    warn!("get_binary: {:?} {:?}", id, value);
    unsafe {
        // *value = 1;
        TResult::ResultOk
    }
}

#[no_mangle]
pub extern "stdcall" fn bro(this: *mut AttributeList, id: *const c_char, value: *mut i64) -> TResult {
    warn!("bro: {:?} {:?}", id, value);
    unsafe {
        // *value = 1;
        TResult::ResultOk
    }
}

#[repr(C)]
struct AttributeList {
    vtable: &'static AttributeListVTable,
    target: i64
}

impl AttributeList {
    pub fn new() -> Self {
        Self {
            vtable: &AttributeListVTable {
                queryInterface: queryInterface4,
                addRef: addRef4,
                release: release4,

                set_int,
                get_int,
                set_float,
                get_float,
                set_string,
                get_string,
                set_binary,
                get_binary,
                bro
            },
            target: 1
        }
    }
}
