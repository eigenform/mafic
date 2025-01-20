# mafic

[An experimental] library for building executable hardware models. 

## Rationale

In this context, a "hardware model" is a set of parallel processes which: 

- Use *wires* to communicate with one another
- Use *registers* to keep track of state

In an actual circuit, signalling over wires is *asynchronous*. 
This is difficult to describe outside of a parallel programming model
(like you'd find in some kind of HDL). 

The idea here is to try and [ab]use Rust's `async/await` features in an 
attempt to let users write Rust code that behaves vaguely like simulated RTL. 
This is basically the same thing as a "discrete-event simulator." 

In Rust, `async` functions are effectively *state machines* where transitions
between states occur at places where the function is `.await`-ing a future 
value. This splitting of `async` functions into concurrent tasks lets us write 
code that "locally" looks like a parallel process. 

## Features

- [x] Undefined and probably incorrect semantics
- [x] Poorly performing single-threaded cycle scheduler
- [x] Unstable and almost unusable API

## Usage

A model consists of the following parts: 

- Types implementing [`ModuleLike`] are simulated "hardware modules"
- [`WireId`] and [`RegisterId`] are references to simulated wires/registers
- [`EngineState`] tracks the state of simulated wires and registers
- [`Engine`] is an `async` executor that performs simulated logic

The [`ModuleLike`] trait is intended to be implemented on `struct` types that 
represent "hardware modules". The `struct` members are intended to represent 
input/output wires and registers belonging to the module. 

When writing a module, input/output wires and registers are represented with 
the [`WireId`] and [`RegisterId`] types. When creating an *instance* of some 
type implementing [`ModuleLike`], all wires and registers must be allocated 
by [`EngineState`]. 

An [`Engine`] is an `async` executor responsible for running a simulation. 
Users are expected to describe the logic associated with a module by 
implementing the [`ModuleLike::run`] method as an `async` function.


## Implementation Notes

Each "step" of the engine involves the following stages: 

1. Cycle through tasks until all futures have completed
2. Update the state of all registers
3. Reset the state of all wires

At the start of a simulated clock cycle, the values in all wires are treated
as "undefined." As we simulate logic, progress through the simulation occurs
when values are successfully driven/sampled from wires. 

In this context, all interactions with wires involve awaiting on a future.
When a wire's value is undefined, the simulated logic must block until the 
value on a wire is known. This can only occur if another task is scheduled
and drives a value on the wire. 

## Example

```
use mafic::*;

pub struct MyModule {
    input: WireId<usize>,
    output: WireId<usize>,
    reg: RegisterId<usize>,
}
impl ModuleLike for MyModule { 
    async fn run(&self) {
        // Sample the input wire
        let x = self.input.sample().await;

        // Add one to the value and send it to a register
        self.reg.drive(x + 1).await;

        // Sample the current register value
        let output = self.reg.sample().await;

        // Drive the register's value on the output wire
        self.output.drive(output).await;
    }
}

pub fn main() {
    let mut state = EngineState::new_shareable();

    // Create an instance of MyModule
    let my_module = {
        let mut state = state.lock().unwrap();
        MyModule { 
            input:  state.wires.alloc(),
            output: state.wires.alloc(),
            reg:    state.registers.alloc(0),
        }
    };

    let mut e = Engine::new(state.clone());

    // Add processes to the schedule for this cycle
    e.schedule_module(&my_module);

    // (This is stimulus for 'my_module.input')
    e.schedule("poke", async { 
        my_module.input.drive(1).await;
    });

    // Run a single cycle to completion
    e.run();

    // The output should be 0! 
    assert!(state.lock().unwrap().wires.peek_wire(my_module.output).unwrap() == 0);

    // Update registers and reset wires for the next cycle
    e.update_registers();
    e.reset_wires();

    // Add processes to the schedule for this cycle (again!)
    e.schedule_module(&my_module);
    e.schedule("poke", async { 
        my_module.input.drive(0).await;
    });

    e.run();

    // The output should be 2! 
    assert!(state.lock().unwrap().wires.peek_wire(my_module.output).unwrap() == 2);
}
```

