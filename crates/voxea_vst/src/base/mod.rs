pub mod funknown;
pub mod plugin;

pub trait GInterface {
    type VTable;
    const iid: i32;
    fn vtable(&mut self) -> &'static Self::VTable;
}

pub struct GUnknown_Vtbl {
    pub query_interface: fn() -> (),
    pub add_ref: fn() -> ()
}

struct GUnknown {
    vtable: &'static GUnknown_Vtbl
}

impl GInterface for GUnknown {
    type VTable = GUnknown_Vtbl;
    const iid: i32 = 10;

    fn vtable(&mut self) -> &'static Self::VTable {
        self.vtable
    }
}

pub trait GUnknown_Impl: GInterface {
    fn query_interface(&mut self) where <Self as GInterface>::VTable: 'static {
        unsafe {
            std::mem::transmute::<&'static Self::VTable, &GUnknown_Vtbl>(self.vtable()).query_interface;
        }
    }

    fn add_ref(&mut self) where <Self as GInterface>::VTable: 'static {
        unsafe {
            std::mem::transmute::<&'static Self::VTable, &GUnknown_Vtbl>(self.vtable()).add_ref;
        }
    }
}

impl GUnknown_Impl for GUnknown {}

pub struct GComponent_Vtbl {
    pub base: GUnknown_Vtbl,
    pub set_component: fn() -> (),
}

struct GComponent {
    vtable: &'static GComponent_Vtbl
}

impl GInterface for GComponent {
    type VTable = GComponent_Vtbl;
    const iid: i32 = 12;

    fn vtable(&mut self) -> &'static Self::VTable {
        self.vtable
    }
}

pub trait GComponent_Impl: GInterface {
    fn set_component(&mut self) where <Self as GInterface>::VTable: 'static {
        unsafe {
            std::mem::transmute::<&'static Self::VTable, &GComponent_Vtbl>(self.vtable()).set_component;
        }
    }
}

impl GComponent_Impl for GComponent {}
impl<T: GComponent_Impl> GUnknown_Impl for T {}

pub struct GComponentProcessor_Vtbl {
    pub base: GComponent_Vtbl,
    pub process: fn() -> (),
}

struct GComponentProcessor {
    vtable: &'static GComponentProcessor_Vtbl
}

impl GInterface for GComponentProcessor {
    type VTable = GComponentProcessor_Vtbl;
    const iid: i32 = 14;

    fn vtable(&mut self) -> &'static Self::VTable {
        self.vtable
    }
}

pub trait GComponentProcessor_Impl: GInterface {
    fn process(&mut self) where <Self as GInterface>::VTable: 'static {
        unsafe {
            std::mem::transmute::<&'static Self::VTable, &GComponentProcessor_Vtbl>(self.vtable()).process;
        }
    }
}

impl GComponentProcessor_Impl for GComponentProcessor {}
impl<T: GComponentProcessor_Impl> GComponent_Impl for T {}

pub struct HostComponentProcessor {
    vtable: &'static GComponentProcessor_Vtbl
}

impl HostComponentProcessor {
    pub fn new() -> Self {
        Self {
            vtable: GComponentProcessor_Vtbl {
                process: HostComponentProcessor::process
            }
        }
    }

}

impl GInterface for HostComponentProcessor {
    type VTable = GComponentProcessor_Vtbl;
    const iid: i32 = 0;

    fn vtable(&mut self) -> &'static Self::VTable {
        self.vtable
    }
}

impl GComponentProcessor_Impl for HostComponentProcessor {
    fn process(&mut self)
    where
        <Self as GInterface>::VTable: 'static,
    {

    }
}

pub fn test(processor: &mut GComponentProcessor) {
    processor.query_interface();
    processor.add_ref();
}


