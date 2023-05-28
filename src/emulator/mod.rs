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

    pub fn load(&mut self, program: Vec<u8>) {
        self.cpu.load(program);
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn run(emulator: &mut Emulator, program: Vec<u8>, cycles: usize) {
        // load test program to memory and set PC to it
        emulator.load(program);

        for _ in 0..cycles {
            emulator.cpu.cycle();
        }
    }

    #[test]
    fn lda_imm() {
        let mut emulator = Emulator::new();

        // load test program to memory and set PC to it
        run(&mut emulator, vec![0xA9, 0x00, 0xA9, 0xF1], 4);

        assert_eq!(emulator.cpu.acc, 0xF1);
        assert_eq!(emulator.cpu.status, 0b10000000);
    }

    #[test]
    fn lda_zp() {
        let mut emulator = Emulator::new();

        // Load test program into memory
        emulator.memory.borrow_mut()[0x0010] = 0x2A; // Set value at zero page address 0x10
        emulator.memory.borrow_mut()[0x0020] = 0x5F; // Set value at zero page address 0x20

        run(&mut emulator, vec![0xA5, 0x10, 0xA5, 0x20], 10);

        // Perform assertions
        assert_eq!(emulator.cpu.acc, 0x5F);
        assert_eq!(emulator.cpu.status, 0b00000000);
    }

    #[test]
    fn lda_zp_x() {
        let mut emulator = Emulator::new();

        // Load test program into memory
        emulator.memory.borrow_mut()[0x0010] = 0x2A; // Set value at zero page address 0x10
        emulator.memory.borrow_mut()[0x0015] = 0x5F; // Set value at zero page address 0x15
        emulator.cpu.idx_x = 0x05; // Set X register value

        run(&mut emulator, vec![0xB5, 0x0B, 0xB5, 0x10], 11);

        // Perform assertions
        assert_eq!(emulator.cpu.acc, 0x5F);
        assert_eq!(emulator.cpu.status, 0b00000000);
    }

    #[test]
    fn lda_abs() {
        let mut emulator = Emulator::new();

        // Load test program into memory
        emulator.memory.borrow_mut()[0x1000] = 0xA9;
        emulator.memory.borrow_mut()[0x1001] = 0x2A;
        emulator.memory.borrow_mut()[0x1002] = 0xA9;
        emulator.memory.borrow_mut()[0x1003] = 0x5F;

        run(&mut emulator, vec![0xAD, 0x00, 0x10, 0xAD, 0x02, 0x10], 10);

        // Perform assertions
        assert_eq!(emulator.cpu.acc, 0xA9);
        assert_eq!(emulator.cpu.status, 0b10000000);
    }

    #[test]
    fn lda_abs_x() {
        let mut emulator = Emulator::new();

        // Load test program into memory
        emulator.memory.borrow_mut()[0x1010] = 0x2A;
        emulator.memory.borrow_mut()[0x1015] = 0x5F;
        emulator.cpu.idx_x = 0x05; // Set X register value

        run(&mut emulator, vec![0xBD, 0x0B, 0x10, 0xBD, 0x10, 0x10], 11);

        // Perform assertions
        assert_eq!(emulator.cpu.acc, 0x5F);
        assert_eq!(emulator.cpu.status, 0b00000000);
    }

    #[test]
    fn lda_abs_y() {
        let mut emulator = Emulator::new();

        // Load test program into memory
        emulator.memory.borrow_mut()[0x0004] = 0x2A;
        emulator.memory.borrow_mut()[0x1005] = 0x5F;
        emulator.cpu.idx_y = 0x05; // Set Y register value

        run(&mut emulator, vec![0xB9, 0x00, 0x10, 0xB9, 0xFF, 0xFF], 11);

        // Perform assertions
        assert_eq!(emulator.cpu.acc, 0x2A);
        assert_eq!(emulator.cpu.status, 0b00000000);
    }

    #[test]
    fn lda_indirect_x() {
        let mut emulator = Emulator::new();

        // Load test program into memory
        emulator.memory.borrow_mut()[0x000F] = 0x20; // Low byte of target address
        emulator.memory.borrow_mut()[0x0010] = 0x10; // High byte of target address
        emulator.memory.borrow_mut()[0x1020] = 0xAB; // Value at target address
        emulator.cpu.idx_x = 0x05; // Set X register value

        run(&mut emulator, vec![0xA1, 0x0A], 6);

        // Perform assertions
        assert_eq!(emulator.cpu.acc, 0xAB);
        assert_eq!(emulator.cpu.status, 0b10000000);
    }

    #[test]
    fn lda_indirect_y() {
        let mut emulator = Emulator::new();

        // Load test program into memory
        emulator.memory.borrow_mut()[0x000A] = 0x20; // Low byte of target address
        emulator.memory.borrow_mut()[0x000B] = 0x10; // High byte of target address
        emulator.memory.borrow_mut()[0x1025] = 0xAB; // Value at target address
        emulator.cpu.idx_y = 0x05; // Set Y register value

        run(&mut emulator, vec![0xB1, 0x0A], 6);

        // Perform assertions
        assert_eq!(emulator.cpu.acc, 0xAB);
        assert_eq!(emulator.cpu.status, 0b10000000);
    }

    #[test]
    fn lda_indirect_y_page_crossing() {
        let mut emulator = Emulator::new();

        // Load test program into memory
        emulator.memory.borrow_mut()[0x000B] = 0x10; // High byte of target address
        emulator.memory.borrow_mut()[0x000A] = 0xFF; // Low byte of target address
        emulator.memory.borrow_mut()[0x1104] = 0xAB; // Value at target address
        emulator.cpu.idx_y = 0x05; // Set Y register value

        run(&mut emulator, vec![0xB1, 0x0A], 6);

        // Perform assertions
        assert_eq!(emulator.cpu.acc, 0xAB);
        assert_eq!(emulator.cpu.status, 0b10000000);
    }

    #[test]
    fn lda_imm_tax() {
        let mut emulator = Emulator::new();

        run(&mut emulator, vec![0xA9, 0x21, 0xAA], 4);

        assert_eq!(emulator.cpu.acc, 0x21);
        assert_eq!(emulator.cpu.idx_x, 0x21);
    }
}
