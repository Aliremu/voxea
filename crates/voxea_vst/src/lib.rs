#![feature(fn_traits)]
#![allow(warnings)]

use crate::base::funknown::{
    FUnknown, FUnknown_Impl, IAudioProcessor, IAudioProcessor_Impl, IComponent, IComponent_Impl,
    IEditController, IEditController_Impl, IPlugView, IPlugViewContentScaleSupport, IPlugView_Impl,
    IPluginBase, IPluginBase_Impl, IPluginFactory, IPluginFactory_Impl, Interface, PClassInfo,
    PFactoryInfo, TResult, ViewType, FUID,
};
use crate::vst::host_application::{
    IConnectionPoint, IConnectionPoint_Impl, IMessage, IMessage_Impl,
};
use anyhow::Result;
use libc::c_char;
use libloading::{Library, Symbol};
use log::{info, warn};
use std::error::Error;
use std::ffi::{c_void, CStr, CString};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use vst::audio_processor::speaker_arr::SpeakerArrangement;
use vst::audio_processor::{
    AudioBusBuffers, BusDirection, BusInfo, IParameterChanges, IoMode, MediaType, ProcessData,
    ProcessMode, ProcessSetup, SymbolicSampleSize,
};
use vst::host_application::IComponentHandler;

pub mod base;
pub mod gui;
pub mod vst;

type InitDllProc = fn() -> bool;
type ExitDllProc = fn() -> bool;
type GetPluginFactoryProc = fn() -> *mut IPluginFactory;

#[derive(Debug, Clone, Copy)]
pub struct VSTPtr<T: FUnknown_Impl> {
    data: *mut T,
    _marker: PhantomData<T>,
}

impl<T: FUnknown_Impl> VSTPtr<T> {
    pub fn new(ptr: *mut T) -> Self {
        Self {
            data: ptr,
            _marker: PhantomData,
        }
    }
}

impl<T: FUnknown_Impl> Deref for VSTPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.data) }
    }
}

impl<T: FUnknown_Impl> DerefMut for VSTPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self.data) }
    }
}

// impl<T: FUnknown_Impl> Drop for VSTPtr<T> {
// fn drop(&mut self) {
// unsafe {
// self.release();
// }
// }
// }

unsafe impl<T: FUnknown_Impl> Sync for VSTPtr<T> {}
unsafe impl<T: FUnknown_Impl> Send for VSTPtr<T> {}

pub struct Module {
    lib: Option<Library>,
}

impl Module {
    pub fn new(path: &str) -> Result<Self> {
        unsafe {
            let lib = Library::new(path).unwrap();
            let init: Symbol<InitDllProc> = lib.get(b"InitDll").unwrap();
            init.call(());

            Ok(Self { lib: Some(lib) })
        }
    }

    pub fn get_factory(&mut self) -> Result<VSTPtr<IPluginFactory>> {
        unsafe {
            let raw_factory: Symbol<GetPluginFactoryProc> = self
                .lib
                .as_ref()
                .expect("Library is None!")
                .get::<GetPluginFactoryProc>(b"GetPluginFactory")?;

            Ok(VSTPtr::new(raw_factory.call(())))
        }
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        unsafe {
            let mut lib = self.lib.take().unwrap();
            let exit: Symbol<ExitDllProc> = lib.get(b"ExitDll").unwrap();
            exit.call(());

            lib.close().unwrap();
        }
    }
}

pub fn uid_to_ascii(uid: [c_char; 16]) -> String {
    // Convert [u8; 16] to a hex string (32 characters long)
    let hex_string = uid
        .iter()
        .map(|byte| format!("{:02X}", byte)) // Format each byte as 2 hex digits
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
}
