
use mafic::*;
use std::sync::*;

pub struct ModuleA { 
    msg_out: WireId<usize>,
    resp_in: WireId<usize>,
}
impl ModuleLike for ModuleA {
    fn new_instance(state: &mut EngineState) -> Self { 
        Self { 
            msg_out: state.wires.alloc(),
            resp_in: state.wires.alloc(),
        }
    }
    async fn run(&self) {
        // Drive a constant value on this wire (to module B)
        self.msg_out.drive(0xdeadbeef).await;

        // Wait for a message (from module B)
        let input = self.resp_in.sample().await;

        assert!(input == 0xdeadbeef+1);
    }
}

pub struct ModuleB { 
    msg_in: WireId<usize>,
    resp_out: WireId<usize>,
}
impl ModuleLike for ModuleB {
    fn new_instance(state: &mut EngineState) -> Self { 
        Self { 
            msg_in: state.wires.alloc(),
            resp_out: state.wires.alloc(),
        }
    }

    async fn run(&self) {
        // Wait for a message (from module A)
        let input = self.msg_in.sample().await;

        // Drive the response (to module A)
        self.resp_out.drive(input+1).await;
    }
}

#[test]
fn simple_test_wires() {
    let mut state = EngineState::new_shareable();

    let a = ModuleA::new_instance(&mut state.lock().unwrap());
    let b = ModuleB::new_instance(&mut state.lock().unwrap());


    let mut e = Engine::new(state.clone());

    e.schedule("poke", async {
        b.msg_in.assign(a.msg_out).await;
        a.resp_in.assign(b.resp_out).await;

    });
    e.schedule_module(&a);
    e.schedule_module(&b);
    e.run();

    drop(e);
}


