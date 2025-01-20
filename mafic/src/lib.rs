
#![feature(context_ext)]
#![feature(local_waker)]
#![feature(noop_waker)]
#![feature(downcast_unchecked)]

#![doc = include_str!("../README.md")]

pub mod wire; 
pub mod register;
pub mod engine;
pub mod module;

use std::sync::*;

pub use crate::engine::{Engine, EngineState};
pub use crate::wire::{WireId, WireMap, WireState};
pub use crate::register::{RegisterId, RegisterMap, RegisterState};
pub use crate::module::ModuleLike;

thread_local! { 
    /// The global instance of [`EngineState`] managed by the library. 
    static STATE: Arc<Mutex<EngineState>> = EngineState::new_shareable();
}


/// Helper for access to global [thread-local] state. 
pub struct Mafic;
impl Mafic { 

    /// Return a mutable reference to the global [`EngineState`]. 
    pub fn state() -> Arc<Mutex<EngineState>> {
        STATE.with(|state| state.clone())
    }

    /// Execute some closure `f` while holding the lock for [`mafic::STATE`]. 
    pub fn with_state<T>(f: impl Fn(&mut EngineState) -> T) -> T {
        STATE.with(|state| {
            (f)(&mut state.lock().unwrap())
        })
    }

    /// Create a new [`Engine`] with the global [`EngineState`]. 
    pub fn init_engine() -> Engine<'static> {
        STATE.with(|state| { 
            Engine::new(state.clone())
        })
    }

    /// Allocate a register
    pub fn reg<T: Copy + std::fmt::Debug + 'static>(init: T) -> RegisterId<T> {
        STATE.with(|state| { 
            //let state = state.clone();
            state.lock().unwrap().registers.alloc(init)
        })
    }

    /// Allocate a wire
    pub fn wire<T: Copy + std::fmt::Debug + 'static>() -> WireId<T> {
        STATE.with(|state| { 
            //let state = state.clone();
            state.lock().unwrap().wires.alloc()
        })
    }

    /// Read the value of a wire
    pub fn peek<T: Copy + std::fmt::Debug + 'static>
        (wire: WireId<T>) -> Option<T>
    {
        STATE.with(|state| { 
            //let state = state.clone();
            state.lock().unwrap().wires.peek_wire(wire)
        })
    }

    /// Read the value of a register
    pub fn read<T: Copy + std::fmt::Debug + 'static>
        (register: RegisterId<T>) -> T
    {
        STATE.with(|state| { 
            //let state = state.clone();
            state.lock().unwrap().registers.peek_register(register)
        })
    }
}

