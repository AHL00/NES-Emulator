use std::{cell::RefCell, rc::Rc};

use super::ram::RAM;

pub struct CPU {
    pub pc: u16,
    pub sp: u8,
    pub acc: u8,
    pub idx_x: u8,
    pub idx_y: u8,
    pub status: u8,
    memory: Rc<RefCell<RAM>>,
    sleep_cycles: u8, // counter for sleep cycles
}

impl CPU {
    pub fn new(memory: Rc<RefCell<RAM>>) -> Self {
        CPU {
            pc: 0,
            sp: 0,
            acc: 0,
            idx_x: 0,
            idx_y: 0,
            status: 0,
            memory,
            sleep_cycles: 0,
        }
    }

    fn get_flag(&mut self, flag: u8) -> bool {
        self.status & flag != 0
    }

    fn set_flag(&mut self, flag: u8) {
        self.status |= flag;
    }

    fn clear_flag(&mut self, flag: u8) {
        self.status &= !flag;
    }

    pub fn cycle(&mut self) {
        // sleep for cycles until sleep_cycles is 0
        if self.sleep_cycles > 0 {
            self.sleep_cycles -= 1;
            return;
        }

        // Fetch
        let opcode = self.memory.borrow_mut()[self.pc as u16];

        // Decode
        match opcode {
            0xA9 => {
                // <-- LDA [ Immediate ] -->
                // Load accumulator with immediate value
                // 2 bytes, 2 cycles
                self.sleep_cycles = 1;

                // set accumulator to value
                self.pc += 1;
                self.acc = self.memory.borrow_mut()[self.pc as u16];

                // set zero flag if accumulator is 0
                if self.acc == 0 {
                    self.set_flag(Flag::Zero);
                } else {
                    self.clear_flag(Flag::Zero);
                }

                // MSB not 1, set negative flag
                if self.acc & 0b1000_0000 != 0 {
                    self.set_flag(Flag::Negative);
                } else {
                    self.clear_flag(Flag::Negative);
                }    
                
            },
            0xAA => {
                // <-- TAX [ None ] -->
                // Transfer accumulator to index X
                // 1 byte, 2 cycles
                self.sleep_cycles = 1;

                self.idx_x = self.acc;

                // set zero flag if index X is 0
                if self.idx_x == 0 {
                    self.set_flag(Flag::Zero);
                } else {
                    self.clear_flag(Flag::Zero);
                }

                // MSB not 1, set negative flag
                if self.idx_x & 0b1000_0000 != 0 {
                    self.set_flag(Flag::Negative);
                } else {
                    self.clear_flag(Flag::Negative);
                }
            },
            
            _ => unimplemented!("Opcode {:#X} not implemented", opcode),
        }

        // Increment PC
        self.pc += 1;
    }
}

// stupid enum implementation, need it because rust doesn't support bitflags on enums
mod Flag {
    pub const Carry: u8 = 0b0000_0001;
    pub const Zero: u8 = 0b0000_0010;
    pub const InterruptDisable: u8 = 0b0000_0100;
    pub const DecimalMode: u8 = 0b0000_1000;
    pub const Break: u8 = 0b0001_0000;
    pub const Unused: u8 = 0b0010_0000;
    pub const Overflow: u8 = 0b0100_0000;
    pub const Negative: u8 = 0b1000_0000;
}

#[allow(dead_code)]
enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndirectX,
    IndirectY,
    NoneAddressing,
 }