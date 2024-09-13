#![feature(fn_traits)]

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
    warn!("iid: {:?}", iid);
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
    pub queryInterface: unsafe extern "stdcall" fn(this: *mut IComponentHandler, _iid: [c_char; 16], obj: *mut *mut c_void) -> i32,
    pub addRef: unsafe extern "stdcall" fn(this: *mut IComponentHandler) -> u32,
    pub release: unsafe extern "stdcall" fn(this: *mut IComponentHandler) -> u32,

    pub getName: unsafe extern "stdcall" fn(this: *mut IHostApplication, name: &mut [u16; 128]) -> i32,
    pub createInstance: unsafe extern "stdcall" fn(this: *mut IHostApplication, cid: [c_char; 16], iid: [c_char; 16], obj: *mut *mut c_void) -> i32,

    pub beginEdit: unsafe extern "stdcall" fn(this: *mut IComponentHandler, id: u32) -> i32,
    pub performEdit : unsafe extern "stdcall" fn(this: *mut IComponentHandler, id: u32, value: u32) -> i32,
    pub endEdit: unsafe extern "stdcall" fn(this: *mut IComponentHandler, id: u32) -> i32,
    pub restartComponent: unsafe extern "stdcall" fn(this: *mut IComponentHandler, flags: i32) -> i32,

    pub isPlugInterfaceSupported: unsafe extern "stdcall" fn(this: *mut IComponentHandler, fuid: FUID) -> i32,

}


#[no_mangle]
pub extern "stdcall" fn queryInterface2(this: *mut IComponentHandler, _iid: [c_char; 16], obj: *mut *mut c_void) -> i32 {
    unsafe {
        *obj = this as *mut c_void;
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
pub extern "stdcall" fn beginEdit(this: *mut IComponentHandler, id: u32) -> i32 {
    0
}

#[no_mangle]
pub extern "stdcall" fn performEdit(this: *mut IComponentHandler, id: u32, value: u32) -> i32 {
    0
}

#[no_mangle]
pub extern "stdcall" fn endEdit(this: *mut IComponentHandler, id: u32) -> i32 {
    0
}

#[no_mangle]
pub extern "stdcall" fn restartComponent(this: *mut IComponentHandler, flags: i32) -> i32 {
    0
}

#[no_mangle]
pub extern "stdcall" fn isPlugInterfaceSupported(this: *mut IComponentHandler, fuid: FUID) -> i32 {
    0
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

                getName,
                createInstance,

                beginEdit,
                performEdit,
                endEdit,
                restartComponent,

                isPlugInterfaceSupported
            }
        }
    }
}

//
// fn uid_to_ascii(uid: [c_char; 16]) -> [u8; 37] {
//     // Step 1: Convert [u8; 16] to a hex string (32 characters long)
//     let hex_string = uid.iter()
//         .map(|byte| format!("{:02X}", byte))  // Format each byte as 2 hex digits
//         .collect::<String>();
//
//     let formatted_uid = format!(
//         "{}{}{}{}{}{}{}{}{}",
//         &hex_string[0..8],
//         "-",
//         &hex_string[8..12],
//         "-",
//         &hex_string[12..16],
//         "-",
//         &hex_string[16..20],
//         "-",
//         &hex_string[20..32]
//     );
//
//     // Step 2: Convert the hex string into [u8; 32] of ASCII values
//     let mut ascii_array = [0u8; 37];
//     for (i, c) in formatted_uid.chars().enumerate() {
//         ascii_array[i] = c as u8;  // Convert each char to its ASCII value
//     }
//
//     ascii_array[36] = 0;
//
//     ascii_array
// }

use std::ffi::{c_void, CStr, CString};
use std::ptr::null_mut;
use std::sync::Arc;
use libc::c_char;
use libloading::{Library, Symbol};
use winapi::shared::windef::HWND;
use log::{info, warn};
use crate::base::funknown::{FUnknown, FUnknown_Impl, IAudioProcessor, IComponent, IEditController, IPluginBase, IPluginFactory, IAudioProcessor_Impl, IComponent_Impl, IEditController_Impl, IPluginBase_Impl, IPluginFactory_Impl, Interface, PClassInfo, PFactoryInfo, TResult, FUID, IPlugView, IPlugView_Impl, IMessage, IMessage_Impl};

pub mod base;

type InitDllProc = fn() -> bool;
type ExitDllProc = fn() -> bool;
type GetPluginFactoryProc = fn() -> *mut IPluginFactory;

pub fn load_vst(window: isize) {
    unsafe {
        let lib = Library::new("../../../vst3/ZamDelay.vst3").unwrap();
        let init: Symbol<InitDllProc> = lib.get(b"InitDll").unwrap();
        init.call(());

        let raw_factory: Symbol<GetPluginFactoryProc> = lib.get::<GetPluginFactoryProc>(b"GetPluginFactory").unwrap();
        let raw_factory: *mut IPluginFactory = raw_factory.call(());
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
            let comp: &mut IComponent = &mut *(comp as *mut IComponent);
            if ret != TResult::ResultOk {
                break;
            }

            // comp.set_active(true);
            let foo_c = host.clone();
            let context = Arc::into_raw(foo_c) as *mut FUnknown;
            println!("Context: {:?}", context as *mut _);
            let res = comp.initialize(context);
            comp.set_active(true);
            let mut edit_cid = FUID::default();

            comp.get_controller_class_id(&mut edit_cid);

            let mut edit: *mut c_void = std::ptr::null_mut();
            let res = factory.create_instance(edit_cid, IEditController::iid, &mut edit);
            let mut edit: &mut IEditController = &mut *(edit as *mut IEditController);
            edit.set_component_handler(context as *mut c_void);
            println!("initializing!");
            let res = edit.initialize(context);
            let view = edit.get_parameter_count();
            let name = CStr::from_bytes_with_nul_unchecked(b"editor\0");

            let view = edit.create_view(name.as_ptr());
            let mut view: &mut IPlugView = &mut *(view as *mut IPlugView);
            let ty = CStr::from_bytes_with_nul_unchecked(b"HWND\0");
            info!("View: {:?}", view);
            view.attached(window as *mut c_void, ty.as_ptr());

            let mut point: *mut c_void = std::ptr::null_mut();


            println!("{} {:?} {:?} {:?} {:?}", class_info, comp as *mut _, edit as *mut _, context as *mut _, view);
        }

        // factory.release();

        let exit: Symbol<ExitDllProc> = lib.get(b"ExitDll").unwrap();
        exit.call(());

        lib.close().unwrap();
    }
}

















#[allow(non_snake_case)]
#[repr(C)]
struct MessageVTable {
    pub queryInterface: unsafe extern "stdcall" fn(this: *mut Message, _iid: [c_char; 16], obj: *mut *mut c_void) -> i32,
    pub addRef: unsafe extern "stdcall" fn(this: *mut Message) -> u32,
    pub release: unsafe extern "stdcall" fn(this: *mut Message) -> u32,
    pub get_message_id: unsafe extern "stdcall" fn(this: *mut Message) -> *const c_char,
    pub set_message_id: unsafe extern "stdcall" fn(this: *mut Message, id: *const c_char) -> (),
    pub get_attributes: unsafe extern "stdcall" fn(this: *mut Message) -> *mut c_void,
}


#[no_mangle]
pub extern "stdcall" fn queryInterface3(this: *mut Message, _iid: [c_char; 16], obj: *mut *mut c_void) -> i32 {
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
    1000
}

#[no_mangle]
pub extern "stdcall" fn get_message_id(this: *mut Message) -> *const c_char {
    unsafe {
        // CStr::from_bytes_with_nul_unchecked(b"init\0").as_ptr()
        (*this).message
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
    let message = Box::new(AttributeList::new());
    Box::into_raw(message) as *mut c_void
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

    pub test1: unsafe extern "stdcall" fn(this: *mut AttributeList) -> TResult,
    pub get_int: unsafe extern "stdcall" fn(this: *mut AttributeList, id: *const c_char, value: *mut i64) -> TResult,
    pub test3: unsafe extern "stdcall" fn(this: *mut AttributeList) -> TResult,
    pub test4: unsafe extern "stdcall" fn(this: *mut AttributeList) -> TResult,
    pub test5: unsafe extern "stdcall" fn(this: *mut AttributeList) -> TResult,
    pub test6: unsafe extern "stdcall" fn(this: *mut AttributeList) -> TResult,
    pub test7: unsafe extern "stdcall" fn(this: *mut AttributeList) -> TResult,
    pub test8: unsafe extern "stdcall" fn(this: *mut AttributeList) -> TResult,
}


#[no_mangle]
pub extern "stdcall" fn queryInterface4(this: *mut AttributeList, _iid: [c_char; 16], obj: *mut *mut c_void) -> i32 {
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
pub extern "stdcall" fn get_int(this: *mut AttributeList, id: *const c_char, value: *mut i64) -> TResult {
    unsafe {
        *value = 1;
        TResult::ResultOk
    }
}

#[no_mangle]
pub extern "stdcall" fn test(this: *mut AttributeList) -> TResult {
    unsafe {
        TResult::ResultOk
    }
}

#[repr(C)]
struct AttributeList {
    vtable: &'static AttributeListVTable
}

impl AttributeList {
    pub fn new() -> Self {
        Self {
            vtable: &AttributeListVTable {
                queryInterface: queryInterface4,
                addRef: addRef4,
                release: release4,

                test1: test,
                get_int,
                test3: test,
                test4: test,
                test5: test,
                test6: test,
                test7: test,
                test8: test,
            }
        }
    }
}
