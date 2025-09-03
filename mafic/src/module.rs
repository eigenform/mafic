//! Types for representing simulated components/modules. 

use crate::wire::*;
use crate::engine::{ Engine, EngineState };

/// Trait implemented on types that represent a simulated "module".
///
/// Implementation Notes
/// ====================
///
/// - [`ModuleLike::run`] takes a **immutable** reference to `Self`. 
///   A type implementing [`ModuleLike`] is expected to carry [`WireId`] 
///   and/or [`RegisterId`] struct members which are *trivially-copyable* 
///   indirect references to the simulated values. 
///
/// - The actual value on a wire is only obtained/mutated by an 
///   [`Engine`](crate::engine::Engine) when futures associated with the 
///   signal are polled. 
///
/// - This means that, when reading a [`WireId`], the simulation necessarily
///   blocks until the wire has been driven by some other module being 
///   simulated concurrently. 
///
///
///
pub trait ModuleLike { 
    /// Creates an instance this module. 
    ///
    /// The implementation of this method *must* allocate all 
    /// wires/ports/registers used by the module, and *also* must call 
    /// this method when creating all submodules. 
    fn new_instance(e: &mut EngineState) -> Self;

    /// Describes the simulated behavior for this module.
    /// 
    /// The future returned by this function is scheduled on an [`Engine`]. 
    async fn run(&self);


    ///// [Asynchronously] sample a signal.
    //async fn sample<'a, T: Copy + std::fmt::Debug + 'static>
    //    (&self, signal: WireId<T>) -> T 
    //{
    //    CombFuture::from_wire(signal).await
    //}

    ///// [Asynchronously] drive a signal.
    //async fn drive<'a, T: Copy + std::fmt::Debug + 'static>
    //    (&self, signal: WireId<T>, data: T)
    //{
    //    CombDriveFuture::for_wire(signal, data).await
    //}

}


//pub struct ModuleFuture { 
//}
//impl Future for ModuleFuture {
//    type Output = ();
//    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
//        Poll::Ready(())
//    }
//}


