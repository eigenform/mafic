use mafic::*;

pub struct ModuleA { 
    out: WireId<usize>,
    reg: RegisterId<usize>,
}
impl ModuleLike for ModuleA {
    fn new_instance(state: &mut EngineState) -> Self { 
        Self { 
            out: state.wires.alloc(),
            reg: state.registers.alloc(0),
        }
    }
    async fn run(&self) {
        let value = self.reg.sample().await;
        println!("reg = {:?}", value);
        self.out.drive(value).await;
        self.reg.drive(value+1).await;
    }
}


#[test]
fn simple_register() {
    let mut state = EngineState::new_shareable();
    let mut e = Engine::new(state.clone());

    let out  = state.lock().unwrap().wires.alloc();
    let reg  = state.lock().unwrap().registers.alloc(0);

    let a = ModuleA { out, reg };

    for _ in 0..3 { 
        e.schedule("MyModule", a.run());
        e.run();
        e.update_registers();
        e.reset_wires();
    }

    drop(e);
}


