use mafic::*;

pub struct ReadPort<T: Copy + std::fmt::Debug + 'static> { 
    idx: WireId<usize>,
    en: WireId<bool>,
    data: WireId<T>,
}

pub struct WritePort<T: Copy + std::fmt::Debug + 'static> { 
    idx: WireId<usize>,
    en: WireId<bool>,
    data: WireId<T>,
}

pub struct RAM<T: Copy + std::fmt::Debug + 'static, const SZ: usize> { 
    rp: ReadPort<T>,
    wp: WritePort<T>,
    data: [RegisterId<T>; SZ],
}
impl <T: Copy + std::fmt::Debug + 'static, const SZ: usize> RAM<T, SZ> 
{ 
    async fn do_readport(&self) {
        let idx = self.rp.idx.sample().await;
        let en = self.rp.en.sample().await;
        if en { 
            let val = self.data[idx].sample().await;
            self.rp.data.drive(val).await;
        }
    }
    async fn do_writeport(&self) {
        let idx = self.wp.idx.sample().await;
        let en = self.wp.en.sample().await;
        if en { 
            let val = self.wp.data.sample().await;
            self.data[idx].drive(val).await;
        }
    }
}
impl <T: Copy + Default + std::fmt::Debug + 'static, const SZ: usize> 
ModuleLike for RAM<T, SZ> 
{
    fn new_instance(state: &mut EngineState) -> Self { 
        Self { 
            rp: ReadPort { 
                idx: state.wires.alloc(),
                en: state.wires.alloc(),
                data: state.wires.alloc(),
            },
            wp: WritePort { 
                idx: state.wires.alloc(),
                en: state.wires.alloc(),
                data: state.wires.alloc(),
            },
            data: [state.registers.alloc(T::default()); SZ],
        }
    }

    async fn run(&self) {
        self.do_readport().await;
        self.do_writeport().await;
    }
}

#[test]
fn simple_ram() {

    let ram: RAM<usize, 8> = Mafic::with_state(|state| {
        RAM { 
            rp: ReadPort { 
                idx: state.wires.alloc(),
                en: state.wires.alloc(),
                data: state.wires.alloc(),
            },
            wp: WritePort { 
                idx: state.wires.alloc(),
                en: state.wires.alloc(),
                data: state.wires.alloc(),
            },
            data: [state.registers.alloc(0); 8],
        }
    });

    let mut e = Mafic::init_engine();

    // ----------------------------
    // Cycle 1 - read from idx 0, write to idx 0
    e.schedule("poke", async {
        ram.rp.en.drive(true).await;
        ram.rp.idx.drive(0).await;
        ram.wp.en.drive(true).await;
        ram.wp.idx.drive(0).await;
        ram.wp.data.drive(0xdeadbeef).await;
    });
    e.schedule_module(&ram);
    e.run();

    // ----------------------------
    // Cycle 2 - read from idx 0

    let x = Mafic::peek(ram.rp.data).unwrap();
    assert!(x == 0x00000000);
    e.step();

    e.schedule("poke", async {
        ram.rp.en.drive(true).await;
        ram.rp.idx.drive(0).await;
        ram.wp.en.drive(false).await;
        ram.wp.idx.drive(0).await;
        ram.wp.data.drive(0).await;
    });
    e.schedule_module(&ram);
    e.run();

    let x = Mafic::peek(ram.rp.data).unwrap();
    assert!(x == 0xdeadbeef);

    drop(e);
}


