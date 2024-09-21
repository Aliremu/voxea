use std::ffi::{c_char, c_void};

use egui::ahash::RandomState;
use log::info;
use rustc_hash::FxHashMap;
use voxea_vst::{base::funknown::{FUnknown_HostImpl, IPluginFactory, IPluginFactory_Impl, Interface, TResult, FUID}, vst::host_application::{IAttributeList_HostImpl, IComponentHandler2_HostImpl, IComponentHandler_HostImpl, IHostApplication_HostImpl, IMessage_HostImpl, String128}, Module, VSTPtr};
use anyhow::Result;

pub struct VSTHostContext {
    pub module: Module,
    pub factory: VSTPtr<IPluginFactory>
}

impl VSTHostContext {
    pub fn new(path: &str) -> Result<Self> {
        unsafe {
            let mut module = Module::new(path)?;
            let mut factory = module.get_factory()?;

            info!("Plugin has {:?} classes", factory.count_classes());
        }
    }
}

#[repr(C)]
pub struct VSTHostApplication {
    vtable: &'static [*const (); 13]
}

impl VSTHostApplication {
    pub fn new() -> Self {
        Self {
            vtable: &[
                <Self as FUnknown_HostImpl>::query_interface as *const _,
                <Self as FUnknown_HostImpl>::query_interface as *const _,
                <Self as FUnknown_HostImpl>::query_interface as *const _,
                
                <Self as IHostApplication_HostImpl>::get_name as *const _,
                <Self as IHostApplication_HostImpl>::create_instance as *const _,
                
                <Self as IComponentHandler_HostImpl>::begin_edit as *const _,
                <Self as IComponentHandler_HostImpl>::perform_edit as *const _,
                <Self as IComponentHandler_HostImpl>::end_edit as *const _,
                <Self as IComponentHandler_HostImpl>::restart_component as *const _,
                
                <Self as IComponentHandler2_HostImpl>::set_dirty as *const _,
                <Self as IComponentHandler2_HostImpl>::request_open_editor as *const _,
                <Self as IComponentHandler2_HostImpl>::start_group_edit as *const _,
                <Self as IComponentHandler2_HostImpl>::finish_group_edit as *const _,
            ]
        }
    }
}

impl Interface for VSTHost {
    type VTable = [*const (); 13];

    const iid: FUID = [0; 16];

    fn vtable(&self) -> &'static Self::VTable {
        self.vtable
    }
}

impl FUnknown_HostImpl for VSTHostApplication {}

impl IHostApplication_HostImpl for VSTHostApplication {
    unsafe fn get_name(&mut self, name: String128) -> TResult {
        
    }

    unsafe fn create_instance(&mut self, cid: FUID, iid: FUID, obj: *mut *mut c_void) -> TResult {
        
    }
}

impl IComponentHandler_HostImpl for VSTHostApplication {
    unsafe fn begin_edit(&mut self,id: *const c_char) -> TResult {
        
    }

    unsafe fn perform_edit(&mut self,id: *const c_char,value:u32) -> TResult {
        
    }

    unsafe fn end_edit(&mut self,id: *const c_char) -> TResult {
        
    }

    unsafe fn restart_component(&mut self,flags:i32) -> TResult {
        
    }
}

impl IComponentHandler2_HostImpl for VSTHostApplication {
    unsafe fn set_dirty(&mut self,state:bool) -> TResult {
    
    }

    unsafe fn request_open_editor(&mut self,name: *const c_char) -> TResult {
        
    }

    unsafe fn start_group_edit(&mut self,) -> TResult {
        
    }

    unsafe fn finish_group_edit(&mut self,) -> TResult {
                            
    }
}

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
    list: FxHashMap<String, ParamValue, RandomState>,
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
