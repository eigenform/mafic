
use mafic::*;

pub struct Adder { 
    x: WireId<u32>,
    y: WireId<u32>,
    z: WireId<u32>,
}
impl ModuleLike for Adder { 
    fn new_instance(state: &mut EngineState) -> Self { 
        Self { 
            x: state.wires.alloc(),
            y: state.wires.alloc(),
            z: state.wires.alloc(),
        }
    }
    async fn run(&self) {
        let x = self.x.sample().await;
        let y = self.y.sample().await;
        self.z.drive(x + y).await;
    }
}

pub struct Top { 
    x: WireId<u32>,
    y: WireId<u32>,
    z: WireId<u32>,
    adder: Adder,
}
impl ModuleLike for Top { 
    fn new_instance(state: &mut EngineState) -> Self { 
        Self { 
            x: state.wires.alloc(),
            y: state.wires.alloc(),
            z: state.wires.alloc(),
            adder: Adder::new_instance(state),
        }
    }

    async fn run(&self) {
        self.adder.x.assign(self.x).await;
        self.adder.y.assign(self.y).await;
        self.z.assign(self.adder.z).await;
    }
}


#[test]
fn nested_module() {

    let top = Mafic::with_state(|state| {
        Top::new_instance(state)
    });


    let mut e = Mafic::init_engine();
    e.schedule("poke", async {
        top.x.drive(0x1111_1111).await;
        top.y.drive(0x2222_2222).await;
    });
    e.schedule_module(&top);
    e.schedule_module(&top.adder);
    e.run();

    let x = Mafic::peek(top.z).unwrap();
    assert!(x == 0x3333_3333);

    e.step();

    e.schedule("poke", async {
        top.x.drive(0x1111_1111).await;
        top.y.drive(0x2222_2222).await;
    });
    e.schedule_module(&top);
    e.schedule_module(&top.adder);
    e.run();

    let x = Mafic::peek(top.z).unwrap();
    assert!(x == 0x3333_3333);



}


