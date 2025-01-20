//! Types for representing simulated registers.

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


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RegisterId<T> { 
    _t: PhantomData<T>,
    id: usize,
}
impl <T: std::fmt::Debug + 'static> RegisterId<T> {
    pub fn new(id: usize) -> Self { 
        Self { 
            _t: PhantomData, 
            id 
        } 
    }
    pub fn id(&self) -> usize { 
        self.id
    }
}
impl <T: Copy + std::fmt::Debug + 'static> RegisterId<T> {
    /// Sample this wire
    pub async fn sample<'a>(&self) -> T 
    {
        SyncFuture::from_signal(self.clone()).await
    }
    /// Drive this wire
    pub async fn drive<'a>(&self, data: T)
    {
        SyncDriveFuture::for_signal(self.clone(), data).await
    }
}

pub struct SyncFuture<T> { 
    register: RegisterId<T>,
}
impl <T> SyncFuture<T> {
    pub fn from_signal(register: RegisterId<T>) -> Self { 
        Self { register }
    }
}
impl <T> Future for SyncFuture<T> 
where T: Copy + std::fmt::Debug + 'static
{
    type Output = T;
    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {

        let state: &mut Arc<Mutex<EngineState>> = ctx.ext().downcast_mut().unwrap();

        // Use the signal ID to get a reference to the signal's state
        let s: Rc<RefCell<Box<dyn RegisterLike>>> = state.lock().unwrap().registers.data.get(&self.register.id)
            .unwrap().clone();

        // Take ownership over the state
        //let mut s = s.lock().unwrap();
        let mut s = s.borrow_mut();

        // Downcast the signal's state into the concrete type
        let s = s.as_any_mut().downcast_mut::<RegisterState<T>>().unwrap();

        Poll::Ready(s.data)
    }
}

pub struct SyncDriveFuture<T> { 
    register: RegisterId<T>,
    data: T,
}
impl <T> SyncDriveFuture<T> {
    pub fn for_signal(register: RegisterId<T>, data: T) -> Self { 
        Self { register, data }
    }
}
impl <T> Future for SyncDriveFuture<T> 
where T: Copy + std::fmt::Debug + 'static
{
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        let state: &mut Arc<Mutex<EngineState>> = ctx.ext().downcast_mut().unwrap();

        // Use the signal ID to get a reference to the signal's state
        let s: Rc<RefCell<Box<dyn RegisterLike>>> = state.lock().unwrap().registers.data.get(&self.register.id)
            .unwrap().clone();

        // Take ownership over the state
        //let mut s = s.lock().unwrap();
        let mut s = s.borrow_mut();

        // Downcast the signal's state into the concrete type
        let s = s.as_any_mut().downcast_mut::<RegisterState<T>>().unwrap();

        s.next = Some(self.data);

        Poll::Ready(())
    }
}


/// The simulated state of a wire tracked by [`Engine`](crate::engine::Engine).
#[derive(Debug)]
pub struct RegisterState<T: Clone + std::fmt::Debug> {

    /// The state of this register. 
    pub data: T,
    /// The state of this register on reset.
    pub reset_data: T,
    /// Abstract "input wire" to this register
    pub next: Option<T>,
}
impl <T: Clone + std::fmt::Debug + 'static> RegisterLike for RegisterState<T> {
    fn reset(&mut self) {
        self.data = self.reset_data.clone();
    }
    fn update(&mut self) {
        if let Some(data) = self.next.take() { 
            self.data = data;
        }
    }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

pub trait RegisterLike { 
    fn reset(&mut self);
    fn update(&mut self);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub type RegisterMapInner = Rc<RefCell<Box<dyn Any + 'static>>>;
pub struct RegisterMap {
    /// Type-erased container for [RegisterState] 
    data: BTreeMap<usize, Rc<RefCell<Box<dyn RegisterLike>>>>,
    next_sid: usize,
}
impl RegisterMap {
    pub fn new() -> Self { 
        Self { 
            data: BTreeMap::new(),
            next_sid: 1,
        }
    }
    pub fn new_arc() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::new()))
    }

    pub fn alloc<T: Copy + std::fmt::Debug + 'static>(&mut self, init: T)
        -> RegisterId<T> 
    {
        let id = self.next_sid;
        let res = RegisterId::new(id);
        //self.signals.insert(id, Arc::new(Mutex::new(Box::new(init))));

        self.data.insert(id, 
            Rc::new(RefCell::new(Box::new(RegisterState::<T> { 
                data: init,
                reset_data: init,
                next: None,
            })))
        );
        self.next_sid += 1;
        res
    }

    pub fn peek_register<T: Copy + std::fmt::Debug + 'static>
        (&self, register: RegisterId<T>) -> T
    {
        //let s: Arc<Mutex<Box<dyn Any>>>; 
        let s: Rc<RefCell<Box<dyn RegisterLike>>>; 

        s = self.data.get(&register.id()).unwrap().clone();

        // Take ownership over the state
        //let mut s = s.lock().unwrap();
        let mut s = s.borrow_mut();

        // Downcast the signal's state into the concrete type
        let s = s.as_any_mut().downcast_mut::<RegisterState<T>>().unwrap();

        // Read the signal state
        //let data = *s;
        //println!("read signal {:?}, {:x?}", self.signal, s);

        s.data
    }

    /// Propagate updates to all tracked registers.
    pub fn update(&mut self) {
        for item in &self.data {
            let mut b = item.1.borrow_mut();
            b.update();
        }
    }

}





