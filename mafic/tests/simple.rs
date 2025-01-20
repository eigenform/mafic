
use mafic::*;
use std::sync::*;

pub struct ModuleA { 
    msg_out: WireId<usize>,
    resp_in: WireId<usize>,
}
impl ModuleLike for ModuleA {
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
    let mut e = Engine::new(state.clone());

    let msg  = state.lock().unwrap().wires.alloc();
    let resp = state.lock().unwrap().wires.alloc();

    let a = ModuleA { msg_out: msg, resp_in: resp, };
    let b = ModuleB { msg_in: msg, resp_out: resp, };

    e.schedule("MyModule", a.run());
    e.schedule("MyModule2", b.run());
    e.run();

    drop(e);
}


