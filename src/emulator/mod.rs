use std::{cell::RefCell, rc::Rc};

use self::{cpu::CPU, ram::RAM, bus::Bus, ppu::PPU};

pub mod cpu;
pub mod ppu;
pub mod ram;
pub mod bus;
mod tests;

pub struct Emulator {
    pub memory: Rc<RefCell<RAM>>,
    pub cpu: CPU,
    pub bus: Rc<Bus>,
}

impl Emulator {
    pub fn new() -> Self {
        let memory = Rc::new(RefCell::new(RAM::new()));
        let ppu = Rc::new(RefCell::new(PPU::new()));
        let bus = Rc::new(Bus::new(memory.clone(), ppu.clone()));

        Emulator {
            memory: memory.clone(),
            cpu: CPU::new(bus.clone()),
            bus: bus.clone(),
        }
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.cpu.load(program);
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
    }
}
