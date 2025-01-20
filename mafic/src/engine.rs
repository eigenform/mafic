//! Implementation of a simulator. 

use std::future::Future;
use std::task::{ ContextBuilder, Waker };
use std::pin::Pin;

use std::collections::*;
use std::sync::*;

use crate::wire::*;
use crate::register::*;
use crate::module::ModuleLike;

/// Container for a future being executed by an [`Engine`]. 
pub struct EngineTask<'a> { 
    /// Human-readable description of this task
    name: &'static str,

    /// The future associated with this task
    fut: Pin<Box<dyn Future<Output = ()> + 'a>>,
}

/// Container for simulated state. 
pub struct EngineState { 
    /// Tracks the state of all wires
    pub wires: WireMap,

    /// Tracks the state of all registers
    pub registers: RegisterMap,
}
impl EngineState {
    fn new() -> Self { 
        Self { 
            wires: WireMap::new(),
            registers: RegisterMap::new(),
        }
    }
    pub fn new_shareable() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::new()))
    }
}

#[derive(Debug)]
pub enum EngineErr { 

}


/// A [wildly inefficient] `async` executor that completes the simulated logic
/// described by types implementing [`ModuleLike`]. 
///
/// Implementation Notes
/// ====================
///
/// - Presumably, all of this is completely unsound. 
///
/// - For now, focus on running this with a single thread.
///
/// - The task queue is filled up at the start of a simulated clock cycle.
///   For each simulated module, the user is expected to add the future 
///   returned by [`ModuleLike::run`](crate::module::ModuleLike::run) 
///   to the queue. Each of these tasks describes the simulated logic that 
///   occurs in-between clock edges. 
///
/// - Reads and writes to wires *must* be computed asynchronously 
///   [ie. with futures] because we do not want to burden the user [too much]
///   with having to explicitly specify how the work associated with each 
///   module should be scheduled. Instead, when a module wants to read from 
///   a wire, we simply wait until the wire has been updated by a different
///   module.
///
/// - When the task queue has been emptied, it means that values have 
///   successfully propagated through all tasks, and all tasks have driven
///   writes to registers. 
///
/// - When the task queue is emptied, we can update the values of registers,
///   reset the state of all wires, and then reschedule the logic for all 
///   modules to be performed again on the next cycle.
///
pub struct Engine<'a> {
    /// Queue of tasks associated with pending futures
    tasks: VecDeque<EngineTask<'a>>,

    /// Simulated state
    state: Arc<Mutex<EngineState>>,

    /// Number of scheduler steps
    steps: usize,

    /// Number of clock cycles
    cycles: usize,
}
impl <'a> Engine<'a> {

    /// Create a new [`Engine`].
    pub fn new(state: Arc<Mutex<EngineState>>) -> Engine<'a> {
        Engine {
            tasks: VecDeque::new(),
            state,
            steps: 0,
            cycles: 0,
        }
    }

    /// Schedule some [arbitrary] future `F`. 
    pub fn schedule<F: Future<Output = ()> + 'a>
        (&mut self, name: &'static str, fut: F) 
    {
        let t = EngineTask { name, fut: Box::pin(fut) };
        self.tasks.push_back(t);
    }

    /// Schedule an instance of some module.  
    pub fn schedule_module(&mut self, module: &'a impl ModuleLike) {
        let fut = Box::pin(module.run());
        let task = EngineTask { 
            name: "",
            fut
        };
        self.tasks.push_back(task);
    }

    /// Perform a single simulated clock-cycle by running tasks until the
    /// queue is emptied (and all pending futures have completed). 
    pub fn run(&mut self) {

        // NOTE: Depends on the 'noop_waker' feature
        let waker = Waker::noop();

        // NOTE: Depends on the 'context_ext' and 'local_waker' features
        let mut cx = ContextBuilder::from_waker(&waker)
            .ext(&mut self.state).build();

        // Just cycle through tasks until we [hopefully] terminate. 
        //
        // NOTE: At some point, you should probably be smarter about this.
        // Also, it's easy to imagine cases where the user may unintentionally
        // create stall conditions. 
        while let Some(mut task) = self.tasks.pop_front() {

            // FIXME: For now, just limit the number of steps. 
            assert!(self.steps < 32, "step limit");

            // try to complete a task
            println!("polling {}", task.name);
            if task.fut.as_mut().poll(&mut cx).is_pending() {
                self.tasks.push_back(task);
                self.steps += 1;
            } else { 
            println!("completed {}", task.name);
            }
        }
    }

    /// Reset the state of all wires.
    pub fn reset_wires(&self) {
        self.state.lock().unwrap().wires.reset();
    }

    /// Update the state of all registers.
    pub fn update_registers(&self) {
        self.state.lock().unwrap().registers.update();
    }

    pub fn step(&mut self) { 
        self.run();
        self.reset_wires();
        self.update_registers();
        self.cycles += 1;
    }


}



