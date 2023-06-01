#[cfg(test)]
pub mod tests {
    use crate::emulator::Emulator;

    fn run(emulator: &mut Emulator, test_program: Vec<u8>, cycles: usize) {
        // load test program to memory and set PC to it
        let mut program: Vec<u8> = vec![
            0x4E, 0x45, 0x53, 0x1A, 0x02, 0x01, 0x01, 0x00, 0x02, 0x01, 0x01, 0x00, 0x02, 0x01,
            0x01, 0x00, // NROM header
        ];

        let mut prg_arr = [0; 0x4000];
        let chr_arr = [0; 0x2000];

        // copy the program to the start of the prg rom bank
        prg_arr[0..test_program.len()].copy_from_slice(&test_program);

        // add the two arrays to the program array
        program.extend_from_slice(&prg_arr);
        program.extend_from_slice(&chr_arr);

        emulator.load_rom(program);

        emulator.toggle_debug();

        for _ in 0..cycles {
            emulator.cpu.cycle();
        }
    }

    // TODO: Performance benchmarking
    // #[test]
    // fn cycles_per_second() {
    //     let mut emulator = Emulator::new();

    //     // load test program to memory and set PC to it
    //     run(&mut emulator, vec![0x4C, 0x00, 0xC0], 1000000);
    // }

    #[test]
    fn mem_write_u16() {
        let emulator = Emulator::new();

        emulator.bus.mem_write_u16(0x0000, 0x1020);

        assert_eq!(emulator.bus.mem_read(0x0000), 0x20);
        assert_eq!(emulator.bus.mem_read(0x0001), 0x10);
    }

    #[test]
    fn mem_read_u16() {
        let emulator = Emulator::new();

        emulator.bus.mem_write(0x0000, 0x20);
        emulator.bus.mem_write(0x0001, 0x10);

        assert_eq!(emulator.bus.mem_read_u16(0x0000), 0x1020);
    }

    pub mod adc {
        use crate::emulator::{tests::tests::run, Emulator};

        #[test]
        fn adc_imm() {
            let mut emulator = Emulator::new();

            emulator.cpu.acc = 0x00;

            // load test program to memory and set PC to it
            run(&mut emulator, vec![0x69, 0x01], 4);

            assert_eq!(emulator.cpu.acc, 0x01);
            assert_eq!(emulator.cpu.status, 0b00000000);
        }

        #[test]
        fn adc_imm_carry() {
            let mut emulator = Emulator::new();

            emulator.cpu.acc = 0xFF;

            // load test program to memory and set PC to it
            run(&mut emulator, vec![0x69, 0x02], 4);

            assert_eq!(emulator.cpu.acc, 0x01);
            assert_eq!(emulator.cpu.status, 0b00000001);
        }

        #[test]
        fn adc_imm_overflow() {
            let mut emulator = Emulator::new();

            emulator.cpu.acc = 0x7F;

            // load test program to memory and set PC to it
            run(&mut emulator, vec![ 0x69, 0x01], 4);

            assert_eq!(emulator.cpu.acc, 0x80);
            assert_eq!(emulator.cpu.status, 0b11000000);
        }

        #[test]
        fn adc_zp() {
            let mut emulator = Emulator::new();

            emulator.cpu.acc = 0x10;

            // Load test program into memory
            emulator.bus.mem_write(0x0020, 0x01);

            run(&mut emulator, vec![0x65, 0x20], 10);

            // Perform assertions
            assert_eq!(emulator.cpu.acc, 0x11);
            assert_eq!(emulator.cpu.status, 0b00000000);
        }

        #[test]
        fn adc_zp_x() {
            let mut emulator = Emulator::new();

            emulator.cpu.acc = 0x10;
            emulator.cpu.idx_x = 0x05;

            // Load test program into memory
            emulator.bus.mem_write(0x0025, 0x01);

            run(&mut emulator, vec![0x75, 0x20], 11);

            // Perform assertions
            assert_eq!(emulator.cpu.acc, 0x11);
            assert_eq!(emulator.cpu.status, 0b00000000);
        }

        #[test]
        fn adc_abs() {
            let mut emulator = Emulator::new();

            emulator.cpu.acc = 0x10;

            // Load test program into memory
            emulator.bus.mem_write(0x1020, 0x01);

            run(&mut emulator, vec![0x6D, 0x20, 0x10], 10);

            // Perform assertions
            assert_eq!(emulator.cpu.acc, 0x11);
            assert_eq!(emulator.cpu.status, 0b00000000);
        }

        #[test]
        fn adc_abs_x() {
            let mut emulator = Emulator::new();

            emulator.cpu.acc = 0x10;
            emulator.cpu.idx_x = 0x05;

            // Load test program into memory
            emulator.bus.mem_write(0x1025, 0x01);

            run(&mut emulator, vec![0x7D, 0x20, 0x10], 11);

            // Perform assertions
            assert_eq!(emulator.cpu.acc, 0x11);
            assert_eq!(emulator.cpu.status, 0b00000000);
        }

        #[test]
        fn adc_abs_y() {
            let mut emulator = Emulator::new();

            emulator.cpu.acc = 0x10;
            emulator.cpu.idx_y = 0x05;

            // Load test program into memory
            emulator.bus.mem_write(0x1025, 0x01);

            run(&mut emulator, vec![0x79, 0x20, 0x10], 11);

            // Perform assertions
            assert_eq!(emulator.cpu.acc, 0x11);
            assert_eq!(emulator.cpu.status, 0b00000000);
        }

        #[test]
        fn adc_ind_x() {
            let mut emulator = Emulator::new();

            emulator.cpu.acc = 0x10;
            emulator.cpu.idx_x = 0x05;

            // Load test program into memory
            emulator.bus.mem_write(0x0005, 0x20);
            emulator.bus.mem_write(0x0006, 0x10);
            emulator.bus.mem_write(0x1020, 0x01);

            run(&mut emulator, vec![0x61, 0x00], 12);

            // Perform assertions
            assert_eq!(emulator.cpu.acc, 0x11);
            assert_eq!(emulator.cpu.status, 0b00000000);
        }

        #[test]
        fn adc_ind_y() {
            let mut emulator = Emulator::new();

            emulator.cpu.acc = 0x10;
            emulator.cpu.idx_y = 0x05;

            emulator.bus.mem_write(0x0005, 0x20);
            emulator.bus.mem_write(0x0006, 0x10);
            emulator.bus.mem_write(0x1025, 0x01);

            // Load test program into memory
            run(&mut emulator, vec![0x71, 0x05], 12);

            // Perform assertions
            assert_eq!(emulator.cpu.acc, 0x11);
            assert_eq!(emulator.cpu.status, 0b00000000);
        }
    }

    pub mod dec {
        use crate::emulator::{tests::tests::run, Emulator};

        #[test]
        fn dec_zp() {
            let mut emulator = Emulator::new();

            // Load test program into memory
            emulator.bus.mem_write(0x0020, 0x01);

            run(&mut emulator, vec![0xC6, 0x20], 5);

            // Perform assertions
            assert_eq!(emulator.bus.mem_read(0x0020), 0x00);
            assert_eq!(emulator.cpu.status, 0b00000010);
        }

        #[test]
        fn dec_zp_wrap() {
            let mut emulator = Emulator::new();

            // Load test program into memory
            emulator.bus.mem_write(0x0020, 0x00);

            run(&mut emulator, vec![0xC6, 0x20], 5);

            // Perform assertions
            assert_eq!(emulator.bus.mem_read(0x0020), 0xFF);
            assert_eq!(emulator.cpu.status, 0b10000000);
        }

        #[test]
        fn dec_zp_x() {
            let mut emulator = Emulator::new();

            emulator.cpu.idx_x = 0x05;

            // Load test program into memory
            emulator.bus.mem_write(0x0025, 0x01);

            run(&mut emulator, vec![0xD6, 0x20], 6);

            // Perform assertions
            assert_eq!(emulator.bus.mem_read(0x0025), 0x00);
            assert_eq!(emulator.cpu.status, 0b00000010);
        }

        #[test]
        fn dec_abs() {
            let mut emulator = Emulator::new();

            // Load test program into memory
            emulator.bus.mem_write(0x1020, 0x01);

            run(&mut emulator, vec![0xCE, 0x20, 0x10], 6);

            // Perform assertions
            assert_eq!(emulator.bus.mem_read(0x1020), 0x00);
            assert_eq!(emulator.cpu.status, 0b00000010);
        }

        #[test]
        fn dec_abs_x() {
            let mut emulator = Emulator::new();

            emulator.cpu.idx_x = 0x05;

            // Load test program into memory
            emulator.bus.mem_write(0x1025, 0x01);

            run(&mut emulator, vec![0xDE, 0x20, 0x10], 7);

            // Perform assertions
            assert_eq!(emulator.bus.mem_read(0x1025), 0x00);
            assert_eq!(emulator.cpu.status, 0b00000010);
        }
    }

    pub mod dex {
        use crate::emulator::{tests::tests::run, Emulator};

        #[test]
        fn dex() {
            let mut emulator = Emulator::new();

            emulator.cpu.idx_x = 0x01;

            run(&mut emulator, vec![0xCA], 2);

            // Perform assertions
            assert_eq!(emulator.cpu.idx_x, 0x00);
            assert_eq!(emulator.cpu.status, 0b00000010);
        }

        #[test]
        fn dex_wrap() {
            let mut emulator = Emulator::new();

            emulator.cpu.idx_x = 0x00;

            run(&mut emulator, vec![0xCA], 2);

            // Perform assertions
            assert_eq!(emulator.cpu.idx_x, 0xFF);
            assert_eq!(emulator.cpu.status, 0b10000000);
        }
    }

    pub mod lda {
        use crate::emulator::{tests::tests::run, Emulator};

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
            emulator.bus.mem_write(0x0010, 0x2A); // Set value at zero page address 0x10
            emulator.bus.mem_write(0x0020, 0x5F); // Set value at zero page address 0x20

            run(&mut emulator, vec![0xA5, 0x10, 0xA5, 0x20], 10);

            // Perform assertions
            assert_eq!(emulator.cpu.acc, 0x5F);
            assert_eq!(emulator.cpu.status, 0b00000000);
        }

        #[test]
        fn lda_zp_x() {
            let mut emulator = Emulator::new();

            // Load test program into memory
            emulator.bus.mem_write(0x0010, 0x2A); // Set value at zero page address 0x10
            emulator.bus.mem_write(0x0015, 0x5F); // Set value at zero page address 0x15
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
            emulator.bus.mem_write(0x1000, 0x2A);
            emulator.bus.mem_write(0x1002, 0x5F);

            run(&mut emulator, vec![0xAD, 0x00, 0x10, 0xAD, 0x02, 0x10], 10);

            // Perform assertions
            assert_eq!(emulator.cpu.acc, 0x5F);
            assert_eq!(emulator.cpu.status, 0b00000000);
        }

        #[test]
        fn lda_abs_x() {
            let mut emulator = Emulator::new();

            // Load test program into memory
            emulator.bus.mem_write(0x1010, 0x2A);
            emulator.bus.mem_write(0x1015, 0x5F);
            emulator.cpu.idx_x = 0x05; // Set X register value

            run(&mut emulator, vec![0xBD, 0x0B, 0x10, 0xBD, 0x10, 0x10], 11);

            // Perform assertions
            assert_eq!(emulator.cpu.acc, 0x5F);
            assert_eq!(emulator.cpu.status, 0b00000000);
        }

        #[test]
        fn lda_abs_x_page_cross() {
            let mut emulator = Emulator::new();

            // Load test program into memory
            emulator.bus.mem_write(0x10FF, 0x2A);
            emulator.bus.mem_write(0x1104, 0x5F);
            emulator.cpu.idx_x = 0x05; // Set X register value

            run(&mut emulator, vec![0xBD, 0xFA, 0x10, 0xBD, 0xFF, 0x10], 12);

            // Perform assertions
            assert_eq!(emulator.cpu.acc, 0x5F);
            assert_eq!(emulator.cpu.status, 0b00000000);
        }

        #[test]
        fn lda_abs_y() {
            let mut emulator = Emulator::new();

            // Load test program into memory
            emulator.bus.mem_write(0x0004, 0x2A);
            emulator.bus.mem_write(0x1005, 0x5F);
            emulator.cpu.idx_y = 0x05; // Set Y register value

            run(&mut emulator, vec![0xB9, 0x00, 0x10, 0xB9, 0xFF, 0xFF], 11);

            // Perform assertions
            assert_eq!(emulator.cpu.acc, 0x2A);
            assert_eq!(emulator.cpu.status, 0b00000000);
        }

        #[test]
        fn lda_abs_y_page_cross() {
            let mut emulator = Emulator::new();

            // Load test program into memory
            emulator.bus.mem_write(0x00FF, 0x2A);
            emulator.bus.mem_write(0x1104, 0x5F);
            emulator.cpu.idx_y = 0x05; // Set Y register value

            run(&mut emulator, vec![0xB9, 0xFA, 0x00, 0xB9, 0xFF, 0x10], 12);

            // Perform assertions
            assert_eq!(emulator.cpu.acc, 0x5F);
            assert_eq!(emulator.cpu.status, 0b00000000);
        }

        #[test]
        fn lda_indirect_x() {
            let mut emulator = Emulator::new();

            // Load test program into memory
            emulator.bus.mem_write(0x000F, 0x20); // Low byte of target address
            emulator.bus.mem_write(0x0010, 0x10); // High byte of target address
            emulator.bus.mem_write(0x1020, 0xAB); // Value at target address
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
            emulator.bus.mem_write(0x000A, 0x20); // Low byte of target address
            emulator.bus.mem_write(0x000B, 0x10); // High byte of target address
            emulator.bus.mem_write(0x1025, 0xAB); // Value at target address
            emulator.cpu.idx_y = 0x05; // Set Y register value

            run(&mut emulator, vec![0xB1, 0x0A], 6);

            // Perform assertions
            assert_eq!(emulator.cpu.acc, 0xAB);
            assert_eq!(emulator.cpu.status, 0b10000000);
        }

        #[test]
        fn lda_ind_y_page_cross() {
            let mut emulator = Emulator::new();

            // Load test program into memory
            emulator.bus.mem_write(0x000B, 0x10); // High byte of target address
            emulator.bus.mem_write(0x000A, 0xFF); // Low byte of target address
            emulator.bus.mem_write(0x1104, 0xAB); // Value at target address
            emulator.cpu.idx_y = 0x05; // Set Y register value

            run(&mut emulator, vec![0xB1, 0x0A, 0xA9, 0x12], 6);
            // if page crossing occurs, an extra cycle is required, which means
            // 0x12 will not be loaded into the accumulator

            // Perform assertions
            assert_eq!(emulator.cpu.acc, 0xAB);
            assert_eq!(emulator.cpu.status, 0b10000000);
        }
    }

    pub mod jmp {
        use crate::emulator::{tests::tests::run, Emulator};

        #[test]
        fn jmp_abs() {
            let mut emulator = Emulator::new();

            emulator.bus.mem_write_u16(0x0000, 0x1000);

            run(&mut emulator, vec![0x4C, 0x00, 0x00], 3);

            assert_eq!(emulator.cpu.pc, 0x1000);
        }

        #[test]
        fn jmp_indirect() {
            let mut emulator = Emulator::new();

            emulator.bus.mem_write_u16(0x0000, 0x1020);
            emulator.bus.mem_write_u16(0x1020, 0x8000);

            run(&mut emulator, vec![0x6C, 0x00, 0x00], 5);

            assert_eq!(emulator.cpu.pc, 0x8000);
        }
    }

    pub mod jsr_rts {
        use crate::emulator::{tests::tests::run, Emulator};

        #[test]
        fn jsr_rts() {
            let mut emulator = Emulator::new();

            // LDA #$12
            emulator.bus.mem_write(0x0012, 0xA9);
            emulator.bus.mem_write(0x0013, 0x12);

            // RTS
            emulator.bus.mem_write(0x0014, 0x60);

            run(&mut emulator, vec![0x20, 0x12, 0x00, 0x8D, 0x10, 0x00], 40);

            assert_eq!(emulator.cpu.acc, 0x12, "JSR not working");
            assert_eq!(emulator.bus.mem_read(0x0010), 0x12, "RTS not working");
        }
    }

    pub mod sta {
        use crate::emulator::{tests::tests::run, Emulator};

        #[test]
        fn sta_zp() {
            let mut emulator = Emulator::new();

            emulator.cpu.acc = 0x21;

            run(&mut emulator, vec![0x85, 0x10], 3);

            assert_eq!(emulator.bus.mem_read(0x0010), 0x21);
        }

        #[test]
        fn sta_zp_x() {
            let mut emulator = Emulator::new();

            emulator.cpu.acc = 0x21;
            emulator.cpu.idx_x = 0x05;

            run(&mut emulator, vec![0x95, 0x10], 4);

            assert_eq!(emulator.bus.mem_read(0x0015), 0x21);
        }

        #[test]
        fn sta_abs() {
            let mut emulator = Emulator::new();

            emulator.cpu.acc = 0x21;

            run(&mut emulator, vec![0x8D, 0x00, 0x10], 4);

            assert_eq!(emulator.bus.mem_read(0x1000), 0x21);
        }

        #[test]
        fn sta_abs_x() {
            let mut emulator = Emulator::new();

            emulator.cpu.acc = 0x21;
            emulator.cpu.idx_x = 0x05;

            run(&mut emulator, vec![0x9D, 0x00, 0x10], 5);

            assert_eq!(emulator.bus.mem_read(0x1005), 0x21);
        }

        #[test]
        fn sta_abs_y() {
            let mut emulator = Emulator::new();

            emulator.cpu.acc = 0x21;
            emulator.cpu.idx_y = 0x05;

            run(&mut emulator, vec![0x99, 0x00, 0x10], 5);

            assert_eq!(emulator.bus.mem_read(0x1005), 0x21);
        }

        #[test]
        fn sta_indirect_x() {
            let mut emulator = Emulator::new();

            emulator.cpu.acc = 0x21;
            emulator.cpu.idx_x = 0x05;

            // Load test program into memory
            emulator.bus.mem_write(0x000F, 0x20); // Low byte of target address
            emulator.bus.mem_write(0x0010, 0x10); // High byte of target address

            run(&mut emulator, vec![0x81, 0x0A], 6);

            // Perform assertions
            assert_eq!(emulator.bus.mem_read(0x1020), 0x21);
        }

        #[test]
        fn sta_indirect_y() {
            let mut emulator = Emulator::new();

            emulator.cpu.acc = 0x21;
            emulator.cpu.idx_y = 0x05;

            // Load test program into memory
            emulator.bus.mem_write(0x000A, 0x20); // Low byte of target address
            emulator.bus.mem_write(0x000B, 0x10); // High byte of target address

            run(&mut emulator, vec![0x91, 0x0A], 6);

            // Perform assertions
            assert_eq!(emulator.bus.mem_read(0x1025), 0x21);
        }
    }

    pub mod inc {
        use crate::emulator::{tests::tests::run, Emulator};

        #[test]
        fn inc_zp() {
            let mut emulator = Emulator::new();

            emulator.bus.mem_write(0x0010, 0x21);

            run(
                &mut emulator,
                vec![0xE6, 0x10, 0xE6, 0x10, 0xE6, 0x10, 0xE6, 0x10, 0xE6, 0x10],
                5 * 5,
            );

            assert_eq!(emulator.bus.mem_read(0x0010), 0x26);
        }

        #[test]
        fn inc_zp_wrapping() {
            let mut emulator = Emulator::new();

            emulator.bus.mem_write(0x0010, 0xFF);

            run(
                &mut emulator,
                vec![0xE6, 0x10, 0xE6, 0x10, 0xE6, 0x10, 0xE6, 0x10, 0xE6, 0x10],
                5 * 5,
            );

            assert_eq!(emulator.bus.mem_read(0x0010), 0x04);
        }

        #[test]
        fn inc_zp_x() {
            let mut emulator = Emulator::new();

            emulator.bus.mem_write(0x0015, 0x21);
            emulator.cpu.idx_x = 0x05;

            run(
                &mut emulator,
                vec![0xF6, 0x10, 0xF6, 0x10, 0xF6, 0x10, 0xF6, 0x10, 0xF6, 0x10],
                6 * 5,
            );

            assert_eq!(emulator.bus.mem_read(0x0015), 0x26);
        }

        #[test]
        fn inc_abs() {
            let mut emulator = Emulator::new();

            emulator.bus.mem_write(0x1000, 0x21);

            run(
                &mut emulator,
                vec![0xEE, 0x00, 0x10, 0xEE, 0x00, 0x10],
                6 * 2,
            );

            assert_eq!(emulator.bus.mem_read(0x1000), 0x23);
        }

        #[test]
        fn inc_abs_x() {
            let mut emulator = Emulator::new();

            emulator.bus.mem_write(0x1005, 0x21);
            emulator.cpu.idx_x = 0x05;

            run(
                &mut emulator,
                vec![0xFE, 0x00, 0x10, 0xFE, 0x00, 0x10],
                7 * 2,
            );

            assert_eq!(emulator.bus.mem_read(0x1005), 0x23);
        }
    }

    pub mod inx {
        use crate::emulator::{tests::tests::run, Emulator};

        #[test]
        fn inx() {
            let mut emulator = Emulator::new();

            emulator.cpu.idx_x = 0x21;

            run(&mut emulator, vec![0xE8, 0xE8, 0xE8, 0xE8, 0xE8], 2 * 5);

            assert_eq!(emulator.cpu.idx_x, 0x26);
        }
    }

    pub mod iny {
        use crate::emulator::{tests::tests::run, Emulator};

        #[test]
        fn iny() {
            let mut emulator = Emulator::new();

            emulator.cpu.idx_y = 0x21;

            run(&mut emulator, vec![0xC8, 0xC8, 0xC8, 0xC8, 0xC8], 2 * 5);

            assert_eq!(emulator.cpu.idx_y, 0x26);
        }
    }

    pub mod tax {
        use crate::emulator::{tests::tests::run, Emulator};

        #[test]
        fn tax() {
            let mut emulator = Emulator::new();

            emulator.cpu.acc = 0x21;

            run(&mut emulator, vec![0xAA, 0xAA, 0xAA, 0xAA, 0xAA], 2 * 5);

            assert_eq!(emulator.cpu.idx_x, 0x21);
        }
    }

    pub mod tay {
        use crate::emulator::{tests::tests::run, Emulator};

        #[test]
        fn tay() {
            let mut emulator = Emulator::new();

            emulator.cpu.acc = 0x21;

            run(&mut emulator, vec![0xA8, 0xA8, 0xA8, 0xA8, 0xA8], 2 * 5);

            assert_eq!(emulator.cpu.idx_y, 0x21);
        }
    }

    pub mod nop {
        use crate::emulator::{tests::tests::run, Emulator};

        #[test]
        fn nop() {
            let mut emulator = Emulator::new();

            // load test program to memory and set PC to it
            run(&mut emulator, vec![0xEA, 0xEA, 0xEA], 4);

            // pc should only be incremented by 2

            assert_eq!(emulator.cpu.pc, 0x8002);

            let mut emulator2 = Emulator::new();

            // load test program to memory and set PC to it
            run(&mut emulator2, vec![0xEA, 0xEA], 2);

            assert_ne!(emulator2.cpu.pc, 0x8002);
        }
    }

    pub mod php {
        use crate::emulator::{tests::tests::run, Emulator};

        #[test]
        fn php() {
            let mut emulator = Emulator::new();

            emulator.cpu.status = 0b10001001;

            run(&mut emulator, vec![0x08], 3);

            assert_eq!(emulator.bus.mem_read(0x01FD), 0b10111001);
        }
    }

    pub mod pha {
        use crate::emulator::{tests::tests::run, Emulator};

        #[test]
        fn pha() {
            let mut emulator = Emulator::new();

            emulator.cpu.acc = 0x21;

            run(&mut emulator, vec![0x48], 3);

            assert_eq!(emulator.bus.mem_read(0x01FD), 0x21);
        }
    }

    pub mod pla {
        use crate::emulator::{tests::tests::run, Emulator};

        #[test]
        fn pla() {
            let mut emulator = Emulator::new();

            emulator.cpu.push_stack(0x21);

            run(&mut emulator, vec![0x68], 4);

            assert_eq!(emulator.cpu.acc, 0x21);
        }
    }

    pub mod plp {
        use crate::emulator::{tests::tests::run, Emulator};

        #[test]
        fn plp() {
            let mut emulator = Emulator::new();

            emulator.cpu.push_stack(0b10111101);

            run(&mut emulator, vec![0x28], 4);

            assert_eq!(emulator.cpu.status, 0b10001101);
        }
    }


}
