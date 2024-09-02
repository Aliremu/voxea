use std::{sync::OnceLock, time::Instant};
use std::collections::VecDeque;
use rustc_hash::FxHashMap;

static mut REGISTRY: OnceLock<FxHashMap<&'static str, Statistics>> = OnceLock::new();
static mut REGION_STACK: OnceLock<VecDeque<&'static str>> = OnceLock::new();

#[derive(Debug, Copy, Clone)]
pub struct Timer {
    pub instant: Instant,
    pub last_elapsed: u128,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct Memory {
    pub allocated: usize,
    pub freed: usize,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct Statistics {
    pub timer: Timer,
    pub memory: Memory,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            instant: Instant::now(),
            last_elapsed: 0,
        }
    }
}

pub fn init() {
    unsafe {
        REGISTRY.get_or_init(|| FxHashMap::default());
        REGION_STACK.get_or_init(|| VecDeque::new());
    }
}

/// Beings a perf session for a region
pub fn begin(region: &'static str) {
    unsafe {
        REGISTRY
            .get_mut_or_init(|| FxHashMap::default())
            .entry(region)
            .and_modify(|v| {
                v.memory = Memory::default();
                v.timer.instant = Instant::now();
            })
            .or_default();

        REGION_STACK
            .get_mut_or_init(|| VecDeque::new())
            .push_back(region);
    }
}

/// Ends a perf session for a region and updates the internal Timer
pub fn end(region: &'static str) {
    unsafe {
        let popped = REGION_STACK
            .get_mut_or_init(|| VecDeque::new())
            .pop_back();
        assert_eq!(popped, Some(region), "Unmatched regions! Current region is: {:?}, trying to end region: {:?}", popped, region);

        REGISTRY
            .get_mut_or_init(|| FxHashMap::default())
            .entry(region)
            .and_modify(|v| {
                v.timer.last_elapsed = v.timer.instant.elapsed().as_micros();
                v.timer.instant = Instant::now();
            })
            .or_default();
    }
}

/// Gets the currently tracked region
pub fn region() -> Option<&'static str> {
    unsafe {
        REGION_STACK
            .get_or_init(|| VecDeque::new())
            .back()
            .cloned()
    }
}

#[inline]
pub fn registry() -> &'static FxHashMap<&'static str, Statistics> {
    unsafe { REGISTRY.get_or_init(|| FxHashMap::default()) }
}

#[inline]
pub fn registry_mut() -> &'static mut FxHashMap<&'static str, Statistics> {
    unsafe { REGISTRY.get_mut_or_init(|| FxHashMap::default()) }
}

#[inline]
pub fn get(region: &'static str) -> Option<&'static Statistics> {
    unsafe { REGISTRY.get_or_init(|| FxHashMap::default()).get(region) }
}

#[inline]
pub fn get_mut(region: &'static str) -> Option<&'static mut Statistics> {
    unsafe { REGISTRY.get_mut_or_init(|| FxHashMap::default()).get_mut(region) }
}

pub fn alloc(size: usize) {
    let Some(region) = region() else { return; };

    get_mut(region).unwrap().memory.allocated += size;
}

pub fn dealloc(size: usize) {
    let Some(region) = region() else { return; };

    get_mut(region).unwrap().memory.freed += size;
}
