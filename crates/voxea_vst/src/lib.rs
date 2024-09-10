#![feature(fn_traits)]
//
// use std::ffi::{c_void, CStr};
// use std::fmt::Formatter;
// use libc::c_char;
// use libloading::{Library, Symbol};
//
// pub mod base;
//
// #[repr(C)]
// #[derive(Debug, Default, Clone, Copy)]
// pub enum FactoryFlags {
//     #[default]
//     NoFlags = 0,
//
//     ClassesDiscardable = 1 << 0,
//     LicenseCheck = 1 << 1,
//     ComponentNonDiscardable = 1 << 3,
//     Unicode = 1 << 4
// }
//
// #[repr(C)]
// #[derive(Debug, Clone, Copy)]
// struct PFactoryInfo {
//     vendor: [c_char; 64],
//     url: [c_char; 256],
//     email: [c_char; 128],
//     flags: FactoryFlags
// }
//
// #[repr(C)]
// #[derive(Debug, Clone, Copy)]
// struct PClassInfo {
//     cid: [c_char; 16],
//     cardinality: i32,
//     category: [c_char; 32],
//     name: [c_char; 64]
// }
//
// impl Default for PClassInfo {
//     fn default() -> Self {
//         Self {
//             cid: [0; 16],
//             cardinality: 0,
//             category: [0; 32],
//             name: [0; 64],
//         }
//     }
// }
//
// impl Default for PFactoryInfo {
//     fn default() -> Self {
//         Self {
//             vendor: [0; 64],
//             url: [0; 256],
//             email: [0; 128],
//             flags: FactoryFlags::NoFlags,
//         }
//     }
// }
//
// impl std::fmt::Display for PFactoryInfo {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         unsafe {
//             write!(f, "PFactoryInfo {{ vendor: {:?}, url: {:?}, email: {:?}, flags: {:?} }}",
//                    CStr::from_ptr(self.vendor.as_ptr()),
//                    CStr::from_ptr(self.url.as_ptr()),
//                    CStr::from_ptr(self.email.as_ptr()),
//                    self.flags
//             )
//         }
//     }
// }
//
// impl std::fmt::Display for PClassInfo {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         unsafe {
//             write!(f, "PClassInfo {{ cid: {:?}, cardinality: {:?}, category: {:?}, name: {:?} }}",
//                    self.cid,
//                    self.cardinality,
//                    CStr::from_ptr(self.category.as_ptr()),
//                    CStr::from_ptr(self.name.as_ptr()),
//             )
//         }
//     }
// }
//
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
//
// #[allow(non_snake_case)]
// #[repr(C)]
// struct FUnknownVTable {
//     pub queryInterface: unsafe extern "thiscall" fn(this: *mut FUnknown, _iid: [c_char; 16], obj: *mut c_void) -> i32,
//     pub addRef: unsafe extern "thiscall" fn(this: *mut FUnknown) -> u32,
//     pub release: unsafe extern "thiscall" fn(this: *mut FUnknown) -> u32,
// }
//
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
// type FIDString = *const c_char;
//
// #[allow(non_snake_case)]
// #[repr(C)]
// struct IPluginFactoryVTable {
//     pub queryInterface: unsafe extern "thiscall" fn(this: *mut IPluginFactory, _iid: [c_char; 16], obj: *mut c_void) -> i32,
//     pub addRef: unsafe extern "thiscall" fn(this: *mut IPluginFactory) -> u32,
//     pub release: unsafe extern "thiscall" fn(this: *mut IPluginFactory) -> u32,
//
//     pub getFactoryInfo: unsafe extern "thiscall" fn(this: *mut IPluginFactory, factory_info: *mut PFactoryInfo) -> i32,
//     pub countClasses: unsafe extern "thiscall" fn(this: *mut IPluginFactory) -> i32,
//     pub getClassInfo: unsafe extern "thiscall" fn(this: *mut IPluginFactory, index: i32, info: *mut PClassInfo) -> i32,
//     pub createInstance: unsafe extern "thiscall" fn(this: *mut IPluginFactory, cid: [c_char; 16], iid: [c_char; 16], obj: *mut *mut c_void) -> i32,
// }
//
// #[repr(C)]
// struct IPluginFactory {
//     vtable: *const IPluginFactoryVTable
// }
//
// impl IPluginFactory {
//     unsafe fn add_ref(&mut self) -> u32 {
//         ((*(self.vtable)).addRef)(self)
//     }
//
//     unsafe fn release(&mut self) -> u32 {
//         ((*(self.vtable)).release)(self)
//     }
//
//     unsafe fn get_factory_info(&mut self, factory_info: *mut PFactoryInfo) -> i32 {
//         ((*(self.vtable)).getFactoryInfo)(self, factory_info)
//     }
//
//     unsafe fn get_class_info(&mut self, index: i32, class_info: *mut PClassInfo) -> i32 {
//         ((*(self.vtable)).getClassInfo)(self, index, class_info)
//     }
//
//     unsafe fn count_classes(&mut self) -> i32 {
//         ((*(self.vtable)).countClasses)(self)
//     }
//
//     unsafe fn create_instance(&mut self, cid: [c_char; 16], iid: [c_char; 16], obj: *mut *mut c_void) -> i32 {
//         ((*(self.vtable)).createInstance)(self, cid, iid, obj)
//     }
// }
//
//
// #[allow(non_snake_case)]
// #[repr(C)]
// struct IPluginBaseVTable {
//     pub queryInterface: unsafe extern "thiscall" fn(this: *mut IPluginBase, _iid: [c_char; 16], obj: *mut c_void) -> i32,
//     pub addRef: unsafe extern "thiscall" fn(this: *mut IPluginBase) -> u32,
//     pub release: unsafe extern "thiscall" fn(this: *mut IPluginBase) -> u32,
//
//     pub initialize: unsafe extern "thiscall" fn(this: *mut IPluginBase, context: *mut IHostApplication) -> i32,
//     pub terminate: unsafe extern "thiscall" fn(this: *mut IPluginBase) -> i32,
// }
//
// #[repr(C)]
// struct IPluginBase {
//     vtable: *const IPluginBaseVTable
// }
//
// impl IPluginBase {
//     unsafe fn add_ref(&mut self) -> u32 {
//         ((*(self.vtable)).addRef)(self)
//     }
//
//     unsafe fn release(&mut self) -> u32 {
//         ((*(self.vtable)).release)(self)
//     }
//
//     unsafe fn initialize(&mut self, context: *mut IHostApplication) -> i32 {
//         ((*(self.vtable)).initialize)(self, context)
//     }
//
//     unsafe fn terminate(&mut self) -> i32 {
//         ((*(self.vtable)).terminate)(self)
//     }
// }
//
#[allow(non_snake_case)]
#[repr(C)]
struct IHostApplicationVTable {
    pub queryInterface: fn(this: *mut IHostApplication, _iid: [c_char; 16], obj: *mut c_void) -> i32,
    pub addRef: fn(this: *mut IHostApplication) -> u32,
    pub release: fn(this: *mut IHostApplication) -> u32,

    pub getName: fn(this: *mut IHostApplication, name: &mut [u16; 128]) -> i32,
    pub createInstance: fn(this: *mut IHostApplication, cid: [c_char; 16], iid: [c_char; 16], obj: *mut *mut c_void) -> i32,
}

#[repr(C)]
struct IHostApplication {
    vtable: &'static IHostApplicationVTable
}

impl IHostApplication {
    pub fn new() -> Self {
        Self {
            vtable: &IHostApplicationVTable {
                queryInterface: |this, iid, obj| {
                    -1
                },

                addRef: |this| {
                    1000
                },

                release: |this| {
                    1000
                },

                getName: |this, name| {
                    name[0] = u16::try_from('N').unwrap();
                    name[1] = 0;

                    0
                },

                createInstance: |this, cid, iid, obj| {
                    0
                }
            }
        }
    }
}
//
// type InitDllProc = fn() -> bool;
// type ExitDllProc = fn() -> bool;
// type GetPluginFactoryProc = fn() -> *mut IPluginFactory;
//

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
//
// pub fn load_vst() {
//     unsafe {
//         let lib = Library::new("../../../vst3/Archetype Nolly.vst3").unwrap();
//         let init: Symbol<InitDllProc> = lib.get(b"InitDll").unwrap();
//         init.call(());
//
//         let raw_factory: Symbol<GetPluginFactoryProc> = lib.get::<GetPluginFactoryProc>(b"GetPluginFactory").unwrap();
//         let raw_factory: *mut IPluginFactory = raw_factory.call(());
//         let factory = &mut *raw_factory;
//
//         factory.add_ref();
//
//         println!("{}", factory.count_classes());
//
//         let mut factory_info = PFactoryInfo::default();
//         factory.get_factory_info(&mut factory_info);
//
//         let mut host = IHostApplication::new();
//
//         for i in 0..factory.count_classes() {
//             let mut class_info = PClassInfo::default();
//             factory.get_class_info(i, &mut class_info);
//
//             let mut object: *mut c_void = std::ptr::null_mut();
//
//             let fid1 = class_info.cid;
//             let fid2 = FUnknown::iid();
//             let fid3 = inline_uid(0xE831FF31, 0xF2D54301, 0x928EBBEE, 0x25697802);
//
//             println!("{:?}", fid1);
//             println!("{:?}", fid2);
//             println!("{:?}", fid3);
//
//             let res = factory.create_instance(fid1, fid2, &mut object);
//
//             let object = &mut *(object as *mut IPluginBase);
//
//             let initres = object.initialize(&mut host);
//
//             object.terminate();
//
//             println!("{:?} {:?} {} {:p} {} {}", class_info.cid, FUnknown::iid(), class_info, object, res, initres);
//         }
//
//
//
//         factory.release();
//
//         println!("{:?}, {}", raw_factory, factory_info);
//
//         let exit: Symbol<ExitDllProc> = lib.get(b"ExitDll").unwrap();
//         exit.call(());
//
//         lib.close().unwrap();
//     }
// }

use std::ffi::c_void;
use std::ptr::null_mut;
use libc::c_char;
use libloading::{Library, Symbol};
use windows_core::IUnknown;
use crate::base::funknown::{FUnknown, FUnknown_Impl, IAudioProcessor, IComponent, IEditController, IPluginBase, IPluginFactory, IAudioProcessor_Impl, IComponent_Impl, IEditController_Impl, IPluginBase_Impl, IPluginFactory_Impl, Interface, PClassInfo, PFactoryInfo, TResult};

pub mod base;

type InitDllProc = fn() -> bool;
type ExitDllProc = fn() -> bool;
type GetPluginFactoryProc = fn() -> *mut IPluginFactory;

// pub struct Test;
//
// trait Test_Impl: Interface {
//
//     fn test(&mut self) -> i32 {
//         (*self.vtable()).
//     }
// }

pub fn load_vst() {
    unsafe {
        let lib = Library::new("../../../vst3/Archetype Nolly.vst3").unwrap();
        let init: Symbol<InitDllProc> = lib.get(b"InitDll").unwrap();
        init.call(());

        let raw_factory: Symbol<GetPluginFactoryProc> = lib.get::<GetPluginFactoryProc>(b"GetPluginFactory").unwrap();
        let raw_factory: *mut IPluginFactory = raw_factory.call(());
        let factory = &mut *raw_factory;

        println!("{}", factory.count_classes());

        let mut factory_info = PFactoryInfo::default();
        factory.get_factory_info(&mut factory_info);

        println!("{}", factory_info);

        let mut host = IHostApplication::new();

        for i in 0..factory.count_classes() {
            let mut class_info = PClassInfo::default();
            factory.get_class_info(i, &mut class_info);

            let mut object: *mut c_void = std::ptr::null_mut();
            let ret = factory.create_instance(class_info.cid, IEditController::iid, &mut object);
            let mut object: &mut IEditController = &mut *(object as *mut IEditController);
            if ret == TResult::ResultOk {
                println!("initializing!");
                let res = object.initialize(&mut host as *mut _ as *mut FUnknown);
                println!("Result: {:?}", res);
            }


            println!("{} {:?} {:?}", class_info, object as *mut _, ret);
        }



        // factory.release();

        let exit: Symbol<ExitDllProc> = lib.get(b"ExitDll").unwrap();
        exit.call(());

        lib.close().unwrap();
    }
}