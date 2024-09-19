pub mod funknown;
pub mod plugin;

pub trait GInterface {
    type VTable;
    const iid: i32;
    fn vtable(&mut self) -> &'static Self::VTable;
}

pub struct GUnknown_Vtbl {
    pub query_interface: fn() -> (),
    pub add_ref: fn() -> (),
}

struct GUnknown {
    vtable: &'static GUnknown_Vtbl,
}

impl GInterface for GUnknown {
    type VTable = GUnknown_Vtbl;
    const iid: i32 = 10;

    fn vtable(&mut self) -> &'static Self::VTable {
        self.vtable
    }
}

pub trait GUnknown_Impl: GInterface {
    fn query_interface(&mut self)
    where
        <Self as GInterface>::VTable: 'static,
    {
    }

    fn add_ref(&mut self)
    where
        <Self as GInterface>::VTable: 'static,
    {
    }
}

impl<T: GInterface> GUnknown_Impl for T {
    fn query_interface(&mut self)
    where
        <Self as GInterface>::VTable: 'static,
    {
        unsafe {
            std::mem::transmute::<&'static Self::VTable, &GUnknown_Vtbl>(self.vtable())
                .query_interface;
        }
    }

    fn add_ref(&mut self)
    where
        <Self as GInterface>::VTable: 'static,
    {
        unsafe {
            std::mem::transmute::<&'static Self::VTable, &GUnknown_Vtbl>(self.vtable()).add_ref;
        }
    }
}
//
// impl GUnknown_Impl for GUnknown {}

pub struct GComponent_Vtbl {
    pub base: GUnknown_Vtbl,
    pub set_component: fn() -> (),
}

struct GComponent {
    vtable: &'static GComponent_Vtbl,
}

impl GInterface for GComponent {
    type VTable = GComponent_Vtbl;
    const iid: i32 = 12;

    fn vtable(&mut self) -> &'static Self::VTable {
        self.vtable
    }
}

pub trait GComponent_Impl: GInterface + GUnknown_Impl {
    fn set_component(&mut self)
    where
        <Self as GInterface>::VTable: 'static,
    {
    }
}

impl Marker<GComponent> for GComponent {}

impl<T: GInterface + GUnknown_Impl + Marker<GComponent>> GComponent_Impl for T {
    fn set_component(&mut self)
    where
        <Self as GInterface>::VTable: 'static,
    {
        unsafe {
            std::mem::transmute::<&'static Self::VTable, &GComponent_Vtbl>(self.vtable())
                .set_component;
        }
    }
}

pub struct GController_Vtbl {
    pub base: GUnknown_Vtbl,
    pub set_controller: fn() -> (),
}

struct GController {
    vtable: &'static GController_Vtbl,
}

impl GInterface for GController {
    type VTable = GController_Vtbl;
    const iid: i32 = 12;

    fn vtable(&mut self) -> &'static Self::VTable {
        self.vtable
    }
}

pub trait GController_Impl: GInterface + GUnknown_Impl {
    fn set_controller(&mut self)
    where
        <Self as GInterface>::VTable: 'static,
    {
    }
}

impl Marker<GController> for GController {}

impl<T: GInterface + Marker<GController>> GController_Impl for T {
    fn set_controller(&mut self)
    where
        <Self as GInterface>::VTable: 'static,
    {
        unsafe {
            std::mem::transmute::<&'static Self::VTable, &GController_Vtbl>(self.vtable())
                .set_controller;
        }
    }
}

pub struct GComponentProcessor_Vtbl {
    pub base: GComponent_Vtbl,
    pub process: fn() -> (),
}

struct GComponentProcessor {
    vtable: &'static GComponentProcessor_Vtbl,
}

impl GInterface for GComponentProcessor {
    type VTable = GComponentProcessor_Vtbl;
    const iid: i32 = 14;

    fn vtable(&mut self) -> &'static Self::VTable {
        self.vtable
    }
}

pub trait GComponentProcessor_Impl: GInterface + Marker<GComponent> {
    fn process(&mut self) -> ()
    where
        <Self as GInterface>::VTable: 'static,
    {
    }
}

pub trait Marker<T> {}

impl Marker<GComponentProcessor> for GComponentProcessor {}
impl Marker<GComponent> for GComponentProcessor {}

impl<T: GInterface + Marker<GComponent> + Marker<GComponentProcessor>> GComponentProcessor_Impl
    for T
{
    fn process(&mut self) -> ()
    where
        <Self as GInterface>::VTable: 'static,
    {
        unsafe {
            std::mem::transmute::<&'static Self::VTable, &GComponentProcessor_Vtbl>(self.vtable())
                .process;
        }
    }
}

pub struct HostComponentProcessor {
    vtable: &'static GComponentProcessor_Vtbl,
}

// impl HostComponentProcessor {
//     pub fn new() -> Self {
//         Self {
//             vtable: GComponentProcessor_Vtbl {
//                 process: HostComponentProcessor::process
//             }
//         }
//     }
//
// }
//
// impl GInterface for HostComponentProcessor {
//     type VTable = GComponentProcessor_Vtbl;
//     const iid: i32 = 0;
//
//     fn vtable(&mut self) -> &'static Self::VTable {
//         self.vtable
//     }
// }
//
// impl GComponentProcessor_Impl for HostComponentProcessor {
//     fn process(&mut self)
//     where
//         <Self as GInterface>::VTable: 'static,
//     {}
// }

pub fn test(processor: &mut GComponentProcessor, controller: &mut GController) {
    processor.query_interface();
    processor.add_ref();
    processor.process();

    controller.set_controller();
    // processor.set_component();
}
