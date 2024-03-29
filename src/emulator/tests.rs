#[cfg(test)]
pub mod tests {
    use crate::emulator::Emulator;

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

        run(&mut emulator, vec![0xB1, 0x0A, 0xA9, 0x12], 6);
        // if page crossing occurs, an extra cycle is required, which means
        // 0x12 will not be loaded into the accumulator

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

    #[test]
    fn jmp_abs() {
        let mut emulator = Emulator::new();

        emulator.memory.borrow_mut()[0x0001] = 0x10;
        emulator.memory.borrow_mut()[0x0000] = 0x00;

        run(&mut emulator, vec![0x4C, 0x00, 0x00], 3);

        assert_eq!(emulator.cpu.pc, 0x1000);
    }

    #[test]
    fn jmp_indirect() {
        let mut emulator = Emulator::new();

        emulator.memory.borrow_mut()[0x0001] = 0x10;
        emulator.memory.borrow_mut()[0x0000] = 0x00; // 0x1000 is the target address
        emulator.memory.borrow_mut()[0x1001] = 0x10;
        emulator.memory.borrow_mut()[0x1000] = 0x20; // target address points to 0x1020

        run(&mut emulator, vec![0x6C, 0x00, 0x00], 5);

        assert_eq!(emulator.cpu.pc, 0x1020);
    }

    #[test]
    fn sta_zp() {
        let mut emulator = Emulator::new();

        emulator.cpu.acc = 0x21;

        run(&mut emulator, vec![0x85, 0x10], 3);

        assert_eq!(emulator.memory.borrow()[0x0010], 0x21);
    }

    #[test]
    fn sta_zp_x() {
        let mut emulator = Emulator::new();

        emulator.cpu.acc = 0x21;
        emulator.cpu.idx_x = 0x05;

        run(&mut emulator, vec![0x95, 0x10], 4);

        assert_eq!(emulator.memory.borrow()[0x0015], 0x21);
    }

    #[test]
    fn sta_abs() {
        let mut emulator = Emulator::new();

        emulator.cpu.acc = 0x21;

        run(&mut emulator, vec![0x8D, 0x00, 0x10], 4);

        assert_eq!(emulator.memory.borrow()[0x1000], 0x21);
    }

    #[test]
    fn sta_abs_x() {
        let mut emulator = Emulator::new();

        emulator.cpu.acc = 0x21;
        emulator.cpu.idx_x = 0x05;

        run(&mut emulator, vec![0x9D, 0x00, 0x10], 5);

        assert_eq!(emulator.memory.borrow()[0x1005], 0x21);
    }

    #[test]
    fn sta_abs_y() {
        let mut emulator = Emulator::new();

        emulator.cpu.acc = 0x21;
        emulator.cpu.idx_y = 0x05;

        run(&mut emulator, vec![0x99, 0x00, 0x10], 5);

        assert_eq!(emulator.memory.borrow()[0x1005], 0x21);
    }

    #[test]
    fn sta_indirect_x() {
        let mut emulator = Emulator::new();

        emulator.cpu.acc = 0x21;
        emulator.cpu.idx_x = 0x05;

        // Load test program into memory
        emulator.memory.borrow_mut()[0x000F] = 0x20; // Low byte of target address
        emulator.memory.borrow_mut()[0x0010] = 0x10; // High byte of target address

        run(&mut emulator, vec![0x81, 0x0A], 6);

        // Perform assertions
        assert_eq!(emulator.memory.borrow()[0x1020], 0x21);
    }

    #[test]
    fn sta_indirect_y() {
        let mut emulator = Emulator::new();

        emulator.cpu.acc = 0x21;
        emulator.cpu.idx_y = 0x05;

        // Load test program into memory
        emulator.memory.borrow_mut()[0x000A] = 0x20; // Low byte of target address
        emulator.memory.borrow_mut()[0x000B] = 0x10; // High byte of target address

        run(&mut emulator, vec![0x91, 0x0A], 6);

        // Perform assertions
        assert_eq!(emulator.memory.borrow()[0x1025], 0x21);
    }
}