use mafic::*;
use std::sync::*;

/// A read request 
pub struct ReadPortReq { 
    /// Index
    idx: WireId<usize>,
    /// Enable
    en: WireId<bool>,
}
impl ReadPortReq {
    pub fn new(e: &mut WireMap) -> Self { 
        Self { 
            idx: e.alloc(),
            en: e.alloc(),
        }
    }
}

/// A read response
pub struct ReadPortResp {
    /// Read result
    data: WireId<usize>,
}
impl ReadPortResp {
    pub fn new(e: &mut WireMap) -> Self { 
        Self { 
            data: e.alloc(),
        }
    }
}

/// A read port
pub struct ReadPort {
    /// Request
    req: ReadPortReq,
    /// Response
    resp: ReadPortResp,
}
impl ReadPort {
    pub fn new(e: &mut WireMap) -> Self { 
        Self { 
            req: ReadPortReq::new(e),
            resp: ReadPortResp::new(e),
        }
    }
}

/// A read-only memory device
pub struct ROM<const NUM_RP: usize> {
    rp: [ReadPort; NUM_RP],
}
impl <const NUM_RP: usize> ROM<NUM_RP> {
    const DATA: [usize; 16] = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
    ];
}
impl <const NUM_RP: usize> ModuleLike for ROM<NUM_RP> {
    async fn run(&self) {
        // For each read port...
        for pid in 0..NUM_RP {

            // Wait for the index and enable bit
            let idx = self.rp[pid].req.idx.sample().await;
            let en = self.rp[pid].req.en.sample().await;
            assert!(idx < 16);

            // Drive a response when the enable bit is high
            if en {
                self.rp[pid].resp.data.drive(Self::DATA[idx]).await;
            }
        }
    }
}

pub struct ROMTestbench {
    rom: ROM<2>,
}
impl ModuleLike for ROMTestbench {
    async fn run(&self) { 

        self.rom.rp[0].req.idx.drive(5).await;
        self.rom.rp[0].req.en.drive(true).await;
        self.rom.rp[1].req.idx.drive(0).await;
        self.rom.rp[1].req.en.drive(false).await;

        let data = self.rom.rp[0].resp.data.sample().await;
        assert!(data == 5);

    }
}

#[test]
fn test_rom() {
    let mut state = EngineState::new_shareable();
    let mut e = Engine::new(state.clone());
    let rp = std::array::from_fn(|f| { 
        ReadPort::new(&mut state.lock().unwrap().wires)
    });
    let rom = ROMTestbench { 
        rom: ROM { rp },
    };

    e.schedule_module(&rom);
    e.schedule_module(&rom.rom);
    e.run();

    drop(e);
}









