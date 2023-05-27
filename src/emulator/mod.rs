use std::{cell::RefCell, rc::Rc};

use self::{cpu::CPU, ram::RAM};

pub mod cpu;
pub mod ppu;
pub mod ram;

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
}

#[cfg(test)]
mod test {
    use super::*;

    fn load_test_program(emulator: &mut Emulator, program: Vec<u8>) {
        // load test program to memory and set PC to it
        for (i, byte) in program.iter().enumerate() {
            emulator.memory.borrow_mut()[i as u16] = *byte;
        }
    }

    fn run(emulator: &mut Emulator, program: Vec<u8>) {      
        // load test program to memory and set PC to it
        let cycles = program.len();
        load_test_program(emulator, program);

        for _ in 0..cycles {
            emulator.cpu.cycle();
        }
    }

    #[test]
    fn lda_imm() {
        let mut emulator = Emulator::new();
        
        // load test program to memory and set PC to it
        run(&mut emulator, vec![0xA9, 0x00, 0xA9, 0x20]);

        println!("Accumulator: {:#X}", emulator.cpu.acc);
        println!("Flags: {:08b}", emulator.cpu.status);
    }

    #[test]
    fn lda_imm_tax() {
        let mut emulator = Emulator::new();
        
        run(&mut emulator, vec![0xA9, 0x21, 0xAA]);

        println!("Accumulator: {:#X}", emulator.cpu.acc);
        println!("Flags: {:08b}", emulator.cpu.status);
        println!("Index X: {:#X}", emulator.cpu.idx_x);
    }


}
