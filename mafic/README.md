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
- [`Engine`] is an `async` executor that uses [`EngineState`] to perform 
  the simulated logic

The [`ModuleLike`] trait is intended to be implemented on `struct` types that 
represent "hardware modules". The `struct` members are intended to represent 
input/output wires and registers belonging to the module. 

When writing a module, input/output wires and registers are represented with 
the [`WireId`] and [`RegisterId`] types. When creating an *instance* of some 
type implementing [`ModuleLike`], all wires/registers/ports must be allocated 
by [`EngineState`]. The implementation of [`ModuleLike::new_instance`] must 
allocate for all wires/registers/ports, and must also call 
[`ModuleLike::new_instance`] on all submodules. 

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


