use crate::base::funknown::{
    DefaultImplementation, FUnknown, FUnknown_HostImpl, FUnknown_Impl, FUnknown_Vtbl,
    IAudioProcessor, IComponent, Interface, Marker, TResult, FUID,
};
use crate::{addRef3, queryInterface3, release3};
use libc::c_char;
use log::{error, warn};
use std::collections::HashMap;
use std::ffi::{c_void, CStr};
use voxea_macro::{implement, interface};

pub type String128 = [u16; 128];

#[interface(0x58E595CC, 0xDB2D4969, 0x8B6AAF8C, 0x36A664E5)]
pub trait IHostApp: FUnknown {
    fn get_name(&mut self, name: String128) -> TResult;
    fn create_instance(&mut self, cid: FUID, iid: FUID, obj: *mut *mut c_void) -> TResult;
}

#[interface(0x58E595CC, 0xDB2D4969, 0x8B6AAF8C, 0x36A664E5)]
pub trait IComponentHandler: FUnknown {
    fn get_name(&mut self, name: String128) -> TResult;
    fn create_instance(&mut self, cid: FUID, iid: FUID, obj: *mut *mut c_void) -> TResult;
}

#[interface(0x70A4156F, 0x6E6E4026, 0x989148BF, 0xAA60D8D1)]
pub trait IConnectionPoint: FUnknown {
    fn connect(&mut self, other: *mut IConnectionPoint) -> TResult;
    fn disconnect(&mut self, other: *mut IConnectionPoint) -> TResult;
    fn notify(&mut self, message: *mut c_void) -> TResult;
}

#[interface(0x936F033B, 0xC6C047DB, 0xBB0882F8, 0x13C1E613)]
pub trait IMessage: FUnknown {
    fn get_message_id(&mut self) -> *const c_char;
    fn set_message_id(&mut self, id: *const c_char) -> ();
    fn get_attributes(&mut self) -> *mut IAttributeList;
}

#[interface(0x1E5F0AEB, 0xCC7F4533, 0xA2544011, 0x38AD5EE4)]
pub trait IAttributeList: FUnknown {
    // Set integer value
    fn set_int(&mut self, id: *const c_char, value: i64) -> TResult;

    // Get integer value
    fn get_int(&mut self, id: *const c_char, value: &mut i64) -> TResult;

    // Set float value
    fn set_float(&mut self, id: *const c_char, value: f64) -> TResult;

    // Get float value
    fn get_float(&mut self, id: *const c_char, value: &mut f64) -> TResult;

    // Set string value (UTF-16, must be null-terminated)
    fn set_string(&mut self, id: *const c_char, string: *const u16) -> TResult;

    // Get string value (UTF-16), size is in bytes
    fn get_string(&mut self, id: *const c_char, string: *mut u16, size_in_bytes: u32) -> TResult;

    // Set binary data
    fn set_binary(&mut self, id: *const c_char, data: *const c_void, size_in_bytes: u32)
        -> TResult;

    // Get binary data
    fn get_binary(
        &mut self,
        id: *const c_char,
        data: &mut *const c_void,
        size_in_bytes: &mut u32,
    ) -> TResult;
}

//
// #[repr(C)]
// pub struct TestHost {
//
// }
//
// impl TestHost {
//     // pub fn new() -> Box<Self> {
//     //     Box::new(Self {})
//     // }
//
//     pub fn to_ffi() -> *const c_void {
//         let host = Box::new(Self {});
//         unsafe {
//             let (pointer, vtable) = std::mem::transmute_copy::<Box<_>, (*const u8, *const usize)>(&host);
//             vtable as *const c_void
//         }
//     }
// }
//
// impl Interface for TestHost {
//     type VTable = ();
//
//     fn vtable(&self) -> &'static Self::VTable {
//         todo!()
//     }
//
//     const iid: FUID = [0; 16];
// }

impl FUnknown_HostImpl for TestMessage {}

#[repr(C)]
pub struct TestVtbl {
    q1: *const usize,
    q2: *const usize,
    q3: *const usize,

    q4: *const usize,
    q5: *const usize,
    q6: *const usize,
}

#[repr(C)]
pub struct TestMessage {
    vtable: &'static TestVtbl,
}

impl TestMessage {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            vtable: &TestVtbl {
                q1: TestMessage::query_interface as *const () as *const usize,
                q2: TestMessage::add_ref as *const () as *const usize,
                q3: TestMessage::release as *const () as *const usize,

                q4: TestMessage::get_message_id as *const () as *const usize,
                q5: TestMessage::set_message_id as *const () as *const usize,
                q6: TestMessage::get_attributes as *const () as *const usize,
            },
        })
    }
}

impl Interface for TestMessage {
    type VTable = TestVtbl;

    fn vtable(&self) -> &'static Self::VTable {
        self.vtable
    }

    const iid: FUID = [0; 16];
}

impl IMessage_HostImpl for TestMessage {
    unsafe fn get_message_id(&mut self) -> *const c_char {
        error!("get_message_id");
        std::ptr::null()
    }

    unsafe fn set_message_id(&mut self, id: *const c_char) -> () {
        error!("set_message_id");
    }

    unsafe fn get_attributes(&mut self) -> *mut IAttributeList {
        error!("get_attributes");
        Box::into_raw(TestApplicationList::new()) as *mut IAttributeList
    }
}

#[repr(C)]
#[derive(Debug)]
pub enum ParamValue {
    IntValue(i64),
    FloatValue(f64),
    CStrValue(*const c_char),
    BinaryValue(*const c_void),
}

#[repr(C)]
pub struct TestApplicationList {
    // vtable: &'static IAttributeList_Vtbl
    vtable: &'static [*const (); 11],
    list: HashMap<String, ParamValue>,
}

impl TestApplicationList {
    pub fn new() -> Box<Self> {
        unsafe {
            Box::new(Self {
                vtable: &[
                    <Self as FUnknown_HostImpl>::query_interface as *const (),
                    <Self as FUnknown_HostImpl>::add_ref as *const (),
                    <Self as FUnknown_HostImpl>::release as *const (),
                    <Self as IAttributeList_HostImpl>::set_int as *const (),
                    <Self as IAttributeList_HostImpl>::get_int as *const (),
                    <Self as IAttributeList_HostImpl>::set_float as *const (),
                    <Self as IAttributeList_HostImpl>::get_float as *const (),
                    <Self as IAttributeList_HostImpl>::set_string as *const (),
                    <Self as IAttributeList_HostImpl>::get_string as *const (),
                    <Self as IAttributeList_HostImpl>::set_binary as *const (),
                    <Self as IAttributeList_HostImpl>::get_binary as *const (),
                ],

                list: HashMap::new(),
            })
        }
    }
}

impl Interface for TestApplicationList {
    type VTable = [*const (); 11];

    fn vtable(&self) -> &'static Self::VTable {
        self.vtable
    }

    const iid: FUID = [0; 16];
}

impl FUnknown_HostImpl for TestApplicationList {}

impl IAttributeList_HostImpl for TestApplicationList {
    unsafe fn set_int(&mut self, id: *const c_char, value: i64) -> TResult {
        warn!("set_int");

        self.list.insert(
            CStr::from_ptr(id).to_str().unwrap().to_string(),
            ParamValue::IntValue(value),
        );
        TResult::ResultOk
    }

    unsafe fn get_int(&mut self, id: *const c_char, value: &mut i64) -> TResult {
        let id = CStr::from_ptr(id).to_str().unwrap();
        warn!("get_int: {:?}", id);
        warn!("Map: {:#?}", self.list);
        if let Some(ParamValue::IntValue(val)) = self.list.get(id) {
            warn!("\"{:?}\" -> {:?}", id, val);
            *value = *val;
        }
        TResult::ResultOk
    }

    unsafe fn set_float(&mut self, id: *const c_char, value: f64) -> TResult {
        warn!("set_float");
        TResult::ResultOk
    }

    unsafe fn get_float(&mut self, id: *const c_char, value: &mut f64) -> TResult {
        warn!("get_float");
        TResult::ResultOk
    }

    unsafe fn set_string(&mut self, id: *const c_char, string: *const u16) -> TResult {
        warn!("set_string");
        TResult::ResultOk
    }

    unsafe fn get_string(
        &mut self,
        id: *const c_char,
        string: *mut u16,
        size_in_bytes: u32,
    ) -> TResult {
        warn!("get_string");
        TResult::ResultOk
    }

    unsafe fn set_binary(
        &mut self,
        id: *const c_char,
        data: *const c_void,
        size_in_bytes: u32,
    ) -> TResult {
        warn!("set_binary");
        TResult::ResultOk
    }

    unsafe fn get_binary(
        &mut self,
        id: *const c_char,
        data: &mut *const c_void,
        size_in_bytes: &mut u32,
    ) -> TResult {
        warn!("get_binary");
        TResult::ResultOk
    }
}
