use std::{cell::RefCell, rc::Rc};

use self::{cpu::CPU, ram::RAM};

pub mod cpu;
pub mod ppu;
pub mod ram;
mod tests;

pub struct Emulator {
    pub memory: Rc<RefCell<RAM>>,
    pub cpu: CPU,
}

impl Emulator {
    pub fn new() -> Self {
        let memory = Rc::new(RefCell::new(RAM::new()));

        Emulator {
            memory: memory.clone(),
            cpu: CPU::new(memory.clone()),
        }
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.cpu.load(program);
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
    }
}
