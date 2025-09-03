//! Types for representing simulated wires. 

use std::collections::*;
use std::sync::*;
use std::rc::*;
use std::cell::*;
use std::marker::PhantomData;
use std::future::Future;
use std::task::{ Context, Poll };
use std::pin::Pin;
use std::any::*;

use crate::engine::EngineState;

/// The direction of a wire
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction { Input, Output, None }

//pub struct Port<T: Copy + std::fmt::Debug + 'static> {
//    _t: PhantomData<T>,
//}
//impl <T: Copy + std::fmt::Debug + 'static> Port<T> {
//}

/// A token for a simulated wire whose state is tracked by [`EngineState`]. 
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct WireId<T> { 
    /// Marker for the underlying type of this wire
    _t: PhantomData<T>,

    /// Identifier for this wire
    id: usize,

    direction: Direction,

    //name: &'static str,
}
impl <T: std::fmt::Debug + 'static> WireId<T> {
    pub fn new(id: usize) -> Self { 
        Self { 
            _t: PhantomData, 
            direction: Direction::None,
            id 
        }
    }

    pub fn id(&self) -> usize { self.id }
}

impl <T: Copy + std::fmt::Debug + 'static> WireId<T> {
    /// Sample the value on this wire
    pub async fn sample<'a>(&self) -> T 
    {
        CombFuture::from_wire(self.clone()).await
    }
    /// Drive this wire with the given value
    pub async fn drive<'a>(&self, data: T)
    {
        CombDriveFuture::for_wire(self.clone(), data).await
    }

    /// Drive this wire with the value on another wire
    pub async fn assign<'a>(&self, other: Self)
    {
        AssignFuture::for_wires(other, self.clone()).await
    }

}


/// Future representing the result of an asynchronous ["combinational"] read
/// from a simulated wire. 
pub struct CombFuture<T> { 
    /// The target [`WireId`]
    wire: WireId<T>,
}
impl <T> CombFuture<T> {
    pub fn from_wire(wire: WireId<T>) -> Self { 
        Self { wire }
    }
}
impl <T> Future for CombFuture<T> 
where T: Copy + std::fmt::Debug + 'static
{
    type Output = T;
    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {

        let state: &mut Arc<Mutex<EngineState>> = {
            ctx.ext().downcast_mut().unwrap()
        };

        let wire_data = state.lock().unwrap().read_wire(self.wire);

        // Read the wire state.
        // When the wire contains 'None', we must be waiting for the value 
        // to be driven by some other simulated process. 
        if let Some(result) = wire_data {
            Poll::Ready(result)
        } else { 
            Poll::Pending
        }
    }
}

pub struct CombDriveFuture<T> { 
    wire: WireId<T>,
    data: T,
}
impl <T> CombDriveFuture<T> {
    pub fn for_wire(wire: WireId<T>, data: T) -> Self { 
        Self { wire, data }
    }
}
impl <T> Future for CombDriveFuture<T> 
where T: Copy + std::fmt::Debug + 'static
{
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context<'_>)
        -> Poll<Self::Output> 
    {
        let state: &mut Arc<Mutex<EngineState>> = ctx.ext().downcast_mut().unwrap();
        state.lock().unwrap().write_wire(self.wire, self.data);
        Poll::Ready(())
    }
}

pub struct AssignFuture<T> {
    src: WireId<T>,
    tgt: WireId<T>,
}
impl <T> AssignFuture<T> {
    pub fn for_wires(src: WireId<T>, tgt: WireId<T>) -> Self { 
        Self { src, tgt }
    }
}
impl <T> Future for AssignFuture<T> 
where T: Copy + std::fmt::Debug + 'static
{
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) 
        -> Poll<Self::Output> 
    {
        let state: &mut Arc<Mutex<EngineState>> = 
            ctx.ext().downcast_mut().unwrap();

        // Read the source wire. 
        // If the state of the source wire is undefined, we need to defer this
        // task until the source wire actually obtains a value ...
        let src_value = state.lock().unwrap().read_wire(self.src);
        if src_value.is_none() {
            return Poll::Pending;
        }

        // Write to the target wire
        let src_data = src_value.unwrap();
        state.lock().unwrap().write_wire(self.tgt, src_data);

        Poll::Ready(())
    }
}


/// The simulated state of a wire tracked by [`Engine`](crate::engine::Engine).
#[derive(Debug)]
pub struct WireState<T: std::fmt::Debug> {

    /// The state of this wire. 
    ///
    /// - `None` indicates that a value has *not* yet been driven to this 
    ///   wire during the current clock cycle
    ///
    /// - `Some` indicates that a value has been driven to this wire earlier
    ///   during the current clock cyce [and is available to be read]
    ///
    pub data: Option<T>,
}
impl <T: std::fmt::Debug + 'static> WireLike for WireState<T> {
    fn reset(&mut self) { self.data = None; }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

/// Trait implemented on types that represent the simulated state of a wire. 
pub trait WireLike {
    /// Reset the value of this wire
    fn reset(&mut self);

    /// Return a type-erased reference to this object 
    fn as_any(&self) -> &dyn Any;

    /// Return a type-erased mutable reference to this object 
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait WireAllocator {
    fn alloc_wire<T>(&mut self, name: &'static str) -> WireId<T>
        where T: Copy + std::fmt::Debug + 'static;

}


pub type WireMapInner = Rc<RefCell<Box<dyn Any + 'static>>>;
pub struct WireMap {
    /// Type-erased container for [WireState] 
    pub data: BTreeMap<usize, Rc<RefCell<Box<dyn WireLike>>>>,

    pub connections: BTreeMap<usize, BTreeSet<usize>>,

    pub next_sid: usize,
}
impl WireMap {
    pub fn new() -> Self { 
        Self { 
            data: BTreeMap::new(),
            connections: BTreeMap::new(),
            next_sid: 1,
        }
    }
    pub fn new_arc() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::new()))
    }

    pub fn alloc<T: Copy + std::fmt::Debug + 'static>(&mut self)
        -> WireId<T> 
    {
        let id = self.next_sid;
        let res = WireId::new(id);

        self.data.insert(id, 
            Rc::new(RefCell::new(Box::new(WireState::<T> { 
                data: None,
            })))
        );
        self.next_sid += 1;
        res
    }

    pub fn peek_wire<T: Copy + std::fmt::Debug + 'static>
        (&self, wire: WireId<T>) -> Option<T>
    {
        let s: Rc<RefCell<Box<dyn WireLike>>> = {
            self.data.get(&wire.id()).unwrap().clone()
        };

        // Take ownership over the state
        let mut s = s.borrow_mut();

        // Downcast the wire's state into the concrete type
        let s = s.as_any_mut().downcast_mut::<WireState<T>>().unwrap();

        // Return the state of this wire
        s.data
    }

    /// Reset all of the wires.
    pub fn reset(&mut self) {
        for item in &self.data {
            let mut b = item.1.borrow_mut();
            b.reset();
        }
    }



}





