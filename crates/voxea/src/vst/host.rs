use std::{ffi::{c_char, c_void, CStr}, sync::Arc};

use egui::ahash::RandomState;
use log::{info, warn};
use rustc_hash::FxHashMap;
use voxea_vst::{base::funknown::{FUnknown, FUnknown_HostImpl, FUnknown_Impl, IAudioProcessor, IAudioProcessor_Impl, IComponent, IComponent_Impl, IEditController, IEditController_Impl, IPlugView, IPluginBase_Impl, IPluginFactory, IPluginFactory_Impl, Interface, PFactoryInfo, TResult, ViewType, FUID}, vst::{audio_processor::{BusDirection, BusInfo, IParameterChanges_HostImpl, IoMode, MediaType, ProcessMode, ProcessSetup, SymbolicSampleSize}, host_application::{IAttributeList, IAttributeList_HostImpl, IComponentHandler, IComponentHandler2_HostImpl, IComponentHandler_HostImpl, IConnectionPoint, IConnectionPoint_Impl, IHostApplication_HostImpl, IMessage_HostImpl, String128}}, Module, VSTPtr};
use anyhow::Result;

#[derive(Default)]
pub struct VSTHostContext {
    pub module: Option<Module>,
    pub factory: Option<VSTPtr<IPluginFactory>>,
    pub component: Option<VSTPtr<IComponent>>,
    pub editor: Option<VSTPtr<IEditController>>,
    pub view: Option<VSTPtr<IPlugView>>
}

impl VSTHostContext {
    pub fn new(path: &str) -> Result<Self> {
        unsafe {
            let mut module = Module::new(path)?;
            let mut factory = module.get_factory()?;
            
            let mut ctx = Self::default();
            
            let mut factory_info = PFactoryInfo::default();
            factory.get_factory_info(&mut factory_info);

            info!("Loaded plugin! {}", factory_info);

            let host = Arc::new(VSTHostApplication::new());

            for i in 0..factory.count_classes() {
                let class_info = factory.get_class_info(i)?;

                if class_info.category() != "Audio Module Class" {
                    continue;
                }

                let comp = factory.create_instance::<IComponent>(class_info.cid)?;
                comp.set_io_mode(IoMode::Simple);

                // let mut bus_info = Arc::new(BusInfo::default());
                // let ptr = Arc::into_raw(bus_info.clone());
                // comp.get_bus_info(MediaType::Audio, BusDirection::Input, 0, ptr as *mut BusInfo);

                // warn!("Bus Count: {:?}", bus_info);
                for i in 0..comp.get_bus_count(MediaType::Audio, BusDirection::Input) {
                    let mut bus_info = BusInfo::default();
                    comp.get_bus_info(MediaType::Audio, BusDirection::Input, i, &mut bus_info);

                    warn!("Bus Input Info: {:?}", bus_info);
                }

                for i in 0..comp.get_bus_count(MediaType::Audio, BusDirection::Output) {
                    let mut bus_info = BusInfo::default();
                    comp.get_bus_info(MediaType::Audio, BusDirection::Output, i, &mut bus_info);

                    warn!("Bus Output Info: {:?}", bus_info);
                }

                // let mut speaker: SpeakerArrangement = 0;
                // processor.get_bus_arrangements(BusDirection::Input, 0, &mut speaker as *mut u64);

                // warn!("Speakers: {:?}", speaker);

                // processor.set_bus_arrangements(inputs, num_inputs, outputs, num_outputs);

                let context = Arc::into_raw(host.clone()) as *mut FUnknown;

                warn!("Context: {:?}", context as *mut _);

                let res = comp.initialize(context);

                let edit = match comp.get_controller_class_id() {
                    Ok(edit_cid) => {
                        warn!("Initializing create_instance! {:?}", res);
                        factory.create_instance::<IEditController>(edit_cid)?
                    }

                    Err(err) => {
                        warn!("Initializing query_interface! {:?}, {:?}", res, err);
                        comp.query_interface::<IEditController>()?
                    }
                };


                let res = edit.initialize(context);

                edit.set_component_handler(context as *mut c_void);

                let component_connection = comp.query_interface::<IConnectionPoint>()?;
                let controller_connection = edit.query_interface::<IConnectionPoint>()?;

                info!("Component Connection: {:?}", component_connection);
                info!("Controller Connection: {:?}", controller_connection);

                component_connection.connect(controller_connection);
                controller_connection.connect(component_connection);

                let processor = comp.query_interface::<IAudioProcessor>()?;
                let res = processor.setup_processing(&mut ProcessSetup {
                    process_mode: ProcessMode::Realtime,
                    symbolic_sample_size: SymbolicSampleSize::Sample32,
                    max_samples_per_block: 192,
                    sample_rate: 44100.0,
                });

                warn!(
                    "AudioProcessor: {:?}. Setup Processing: {:?}",
                    processor, res
                );

                comp.activate_bus(MediaType::Audio, BusDirection::Input, 0, true);
                comp.activate_bus(MediaType::Audio, BusDirection::Output, 0, true);

                comp.set_active(true);

                warn!("Parameter count: {}", edit.get_parameter_count());

                let view = edit.create_view(ViewType::Editor);

                warn!(
                    "{} {:?} {:?} {:?} {:?}",
                    class_info, comp as *mut _, edit as *mut _, context as *mut _, view
                );

                ctx.component = Some(VSTPtr::new(comp));
                ctx.editor = Some(VSTPtr::new(edit));
                ctx.view = Some(VSTPtr::new(view));
            }

            ctx.module = Some(module);

            Ok(ctx)     
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

impl Interface for VSTHostApplication {
    type VTable = [*const (); 13];

    const iid: FUID = [0; 16];

    fn vtable(&self) -> &'static Self::VTable {
        self.vtable
    }
}

impl FUnknown_HostImpl for VSTHostApplication {}

impl IHostApplication_HostImpl for VSTHostApplication {
    unsafe fn get_name(&mut self, name: String128) -> TResult {
        TResult::ResultOk
    }

    unsafe fn create_instance(&mut self, cid: FUID, iid: FUID, obj: *mut *mut c_void) -> TResult {
        
        TResult::ResultOk
    }
}

impl IComponentHandler_HostImpl for VSTHostApplication {
    unsafe fn begin_edit(&mut self,id: *const c_char) -> TResult {
        
        TResult::ResultOk
    }

    unsafe fn perform_edit(&mut self,id: *const c_char,value:u32) -> TResult {
        
        TResult::ResultOk
    }

    unsafe fn end_edit(&mut self,id: *const c_char) -> TResult {
        
        TResult::ResultOk
    }

    unsafe fn restart_component(&mut self,flags:i32) -> TResult {
        
        TResult::ResultOk
    }
}

impl IComponentHandler2_HostImpl for VSTHostApplication {
    unsafe fn set_dirty(&mut self,state:bool) -> TResult {
    
        TResult::ResultOk
    }

    unsafe fn request_open_editor(&mut self,name: *const c_char) -> TResult {
        
        TResult::ResultOk
    }

    unsafe fn start_group_edit(&mut self,) -> TResult {
        
        TResult::ResultOk
    }

    unsafe fn finish_group_edit(&mut self,) -> TResult {
                            
        TResult::ResultOk
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
        warn!("get_message_id");
        std::ptr::null()
    }

    unsafe fn set_message_id(&mut self, id: *const c_char) -> () {
        warn!("set_message_id");
    }

    unsafe fn get_attributes(&mut self) -> *mut IAttributeList {
        warn!("get_attributes");
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
    list: FxHashMap<String, ParamValue>,
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

                list: FxHashMap::default(),
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


#[repr(C)]
pub struct HostParameterChanges {
    vtable: &'static [*const (); 6],
}

impl HostParameterChanges {
    pub fn new() -> Self {
        Self {
            vtable: &[
                <Self as FUnknown_HostImpl>::query_interface as *const (),
                <Self as FUnknown_HostImpl>::add_ref as *const (),
                <Self as FUnknown_HostImpl>::release as *const (),
                <Self as IParameterChanges_HostImpl>::get_parameter_count as *const (),
                <Self as IParameterChanges_HostImpl>::get_parameter_data as *const (),
                <Self as IParameterChanges_HostImpl>::add_parameter_data as *const (),
            ],
        }
    }
}

impl Interface for HostParameterChanges {
    type VTable = [*const (); 6];
    const iid: FUID = [0; 16];

    fn vtable(&self) -> &'static Self::VTable {
        self.vtable
    }
}

impl FUnknown_HostImpl for HostParameterChanges {}

impl IParameterChanges_HostImpl for HostParameterChanges {
    unsafe fn get_parameter_count(&mut self) -> i32 {
        warn!("get_parameter_count");
        0
    }

    unsafe fn get_parameter_data(&mut self) -> *mut c_void {
        warn!("get_parameter_data");
        std::ptr::null_mut()
    }

    unsafe fn add_parameter_data(&mut self, id: *const c_char, index: *mut i32) -> *mut c_void {
        warn!("add_parameter_data");
        std::ptr::null_mut()
    }
}
