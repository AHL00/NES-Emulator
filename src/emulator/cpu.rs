use std::rc::Rc;

use super::bus::Bus;

pub struct CPU {
    pub pc: u16,
    pub sp: u8,
    pub acc: u8,
    pub idx_x: u8,
    pub idx_y: u8,
    pub status: u8,
    bus: Rc<Bus>,
    sleep_cycles: u8, // counter for sleep cycles
}

#[allow(dead_code)]
impl CPU {
    pub fn new(bus: Rc<Bus>) -> Self {
        CPU {
            pc: 0,
            sp: 0,
            acc: 0,
            idx_x: 0,
            idx_y: 0,
            status: 0,
            bus,
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

    fn check_zero(&mut self, value: u8) {
        if value == 0 {
            self.set_flag(Flag::Zero);
        } else {
            self.clear_flag(Flag::Zero);
        }
    }

    fn check_negative(&mut self, value: u8) {
        if value & 0b1000_0000 != 0 {
            self.set_flag(Flag::Negative);
        } else {
            self.clear_flag(Flag::Negative);
        }
    }

    pub fn cycle(&mut self) {
        //print!("PC: {:04X} | ", self.pc);
        // sleep for cycles until sleep_cycles is 0
        if self.sleep_cycles > 0 {
            self.sleep_cycles -= 1;
            //println!("/\\ sleeping");
            return;
        }
        //println!();

        // Fetch
        let opcode = self.bus.mem_read(self.pc);
        let mut dont_increment_pc = false;

        // Decode, Execute
        match opcode {
            // <--| LDA |-->
            0xA9 /* <-- [ Immediate ] --> */ => {
                // Load accumulator with immediate value
                // 2 bytes, 2 cycles
                self.sleep_cycles = 1;

                let addr = self.get_addr_from_operand(AddressingMode::Immediate);
                self.acc = self.bus.mem_read(addr);

                // skip next byte
                self.pc += 1;

                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            0xA5 /* <-- [ Zero Page ] --> */ => {
                // Load accumulator with zero page value
                // 2 bytes, 3 cycles
                self.sleep_cycles = 2;

                // set accumulator to value
                let addr = self.get_addr_from_operand(AddressingMode::ZeroPage);
                self.acc = self.bus.mem_read(addr);

                // skip next byte
                self.pc += 1;

                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            0xB5 /* <-- [ Zero Page, X ] --> */ => {
                // Load accumulator with zero page value
                // 2 bytes, 4 cycles
                self.sleep_cycles = 3;

                // set accumulator to value
                let addr = self.get_addr_from_operand(AddressingMode::ZeroPageX);
                self.acc = self.bus.mem_read(addr);

                // skip next byte
                self.pc += 1;

                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            0xAD /* <-- [ Absolute ] --> */ => {
                // Load accumulator with data at absolute value
                // 3 bytes, 4 cycles
                self.sleep_cycles = 3;

                // set accumulator to value
                let addr = self.get_addr_from_operand(AddressingMode::Absolute);
                self.acc = self.bus.mem_read(addr);

                // skip next 2 bytes
                self.pc += 2;

                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            0xBD /* <-- [ Absolute, X ] --> */ => {
                // Load accumulator with data at absolute value + X
                // 3 bytes, 4 cycles
                self.sleep_cycles = 3;

                // set accumulator to value
                let addr = self.get_addr_from_operand(AddressingMode::AbsoluteX);
                self.acc = self.bus.mem_read(addr);

                // if page crossed, add 1 cycle
                let base_addr = self.get_addr_from_operand(AddressingMode::Absolute);
                if self.check_page_cross(base_addr, addr) {
                    self.sleep_cycles += 1;
                }

                // skip next 2 bytes
                self.pc += 2;

                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            0xB9 /* <-- [ Absolute, Y ] --> */ => {
                // Load accumulator with data at absolute value + X
                // 3 bytes, 4 cycles
                self.sleep_cycles = 3;

                // set accumulator to value
                let addr = self.get_addr_from_operand(AddressingMode::AbsoluteY);
                self.acc = self.bus.mem_read(addr);

                // if page crossed, add 1 cycle
                let base_addr = self.get_addr_from_operand(AddressingMode::Absolute);
                if self.check_page_cross(base_addr, addr) {
                    self.sleep_cycles += 1;
                }

                // skip next 2 bytes
                self.pc += 2;

                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            0xA1 /* <-- [ Indirect, X ] --> */ => {
                // Load accumulator with data at indirect value + X
                // 2 bytes, 6 cycles
                self.sleep_cycles = 5;

                // set accumulator to value
                let addr = self.get_addr_from_operand(AddressingMode::IndirectX);
                self.acc = self.bus.mem_read(addr);

                // skip next byte
                self.pc += 1;

                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            0xB1 /* <-- [ Indirect, Y ] --> */ => {
                // Load accumulator with data at indirect value + Y
                // 2 bytes, 5 cycles
                self.sleep_cycles = 4;

                // set accumulator to value
                let addr = self.get_addr_from_operand(AddressingMode::IndirectY);
                self.acc = self.bus.mem_read(addr);

                // if page crossed, add 1 cycle
                let base_addr = self.get_addr_from_operand(AddressingMode::Indirect);
                if self.check_page_cross(base_addr, addr) {
                    self.sleep_cycles += 1;
                }

                // skip next byte
                self.pc += 1;

                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }

            // <--| JMP |-->
            0x4C /* <-- [ Absolute ] --> */ => {
                // Jump to absolute address
                // 3 bytes, 3 cycles
                self.sleep_cycles = 2;

                // set PC to value
                let addr = self.get_addr_from_operand(AddressingMode::Absolute);
                self.pc = self.bus.mem_read_u16(addr);

                // PC is incremented at end of cycle
                dont_increment_pc = true;
            }
            0x6C /* <-- [ Indirect ] --> */ => {
                // Jump to indirect address
                // 3 bytes, 5 cycles
                self.sleep_cycles = 4;

                // set PC to value
                let addr = self.get_addr_from_operand(AddressingMode::Indirect);
                self.pc = self.bus.mem_read_u16(addr);

                // PC is incremented at end of cycle
                dont_increment_pc = true;
            }

            // <--| STA |-->
            0x85 /* <-- [ Zero Page ] --> */ => {
                // Store accumulator at zero page address
                // 2 bytes, 3 cycles
                self.sleep_cycles = 2;

                // set memory to accumulator
                let addr = self.get_addr_from_operand(AddressingMode::ZeroPage);
                self.bus.mem_write(addr, self.acc);

                // skip next byte
                self.pc += 1;
            }
            0x95 /* <-- [ Zero Page, X ] --> */ => {
                // Store accumulator at zero page address + X
                // 2 bytes, 4 cycles
                self.sleep_cycles = 3;

                // set memory to accumulator
                let addr = self.get_addr_from_operand(AddressingMode::ZeroPageX);
                self.bus.mem_write(addr, self.acc);

                // skip next byte
                self.pc += 1;
            }
            0x8D /* <-- [ Absolute ] --> */ => {
                // Store accumulator at absolute address
                // 3 bytes, 4 cycles
                self.sleep_cycles = 3;

                // set memory to accumulator
                let addr = self.get_addr_from_operand(AddressingMode::Absolute);
                self.bus.mem_write(addr, self.acc);

                // skip next 2 bytes
                self.pc += 2;
            }
            0x9D /* <-- [ Absolute, X ] --> */ => {
                // Store accumulator at absolute address + X
                // 3 bytes, 5 cycles
                self.sleep_cycles = 4;

                // set memory to accumulator
                let addr = self.get_addr_from_operand(AddressingMode::AbsoluteX);
                self.bus.mem_write(addr, self.acc);

                // skip next 2 bytes
                self.pc += 2;
            }
            0x99 /* <-- [ Absolute, Y ] --> */ => {
                // Store accumulator at absolute address + Y
                // 3 bytes, 5 cycles
                self.sleep_cycles = 4;

                // set memory to accumulator
                let addr = self.get_addr_from_operand(AddressingMode::AbsoluteY);
                self.bus.mem_write(addr, self.acc);

                // skip next 2 bytes
                self.pc += 2;
            }
            0x81 /* <-- [ Indirect, X ] --> */ => {
                // Store accumulator at indirect address + X
                // 2 bytes, 6 cycles
                self.sleep_cycles = 5;

                // set memory to accumulator
                let addr = self.get_addr_from_operand(AddressingMode::IndirectX);
                self.bus.mem_write(addr, self.acc);

                // skip next byte
                self.pc += 1;
            }
            0x91 /* <-- [ Indirect, Y ] --> */ => {
                // Store accumulator at indirect address + Y
                // 2 bytes, 6 cycles
                self.sleep_cycles = 5;

                // set memory to accumulator
                let addr = self.get_addr_from_operand(AddressingMode::IndirectY);
                self.bus.mem_write(addr, self.acc);

                // skip next byte
                self.pc += 1;
            }

            // <--| INC |-->
            0xE6 /* <-- [ Zero Page ] --> */ => {
                // Increment value at zero page address
                // 2 bytes, 5 cycles
                self.sleep_cycles = 4;

                // increment value
                let addr = self.get_addr_from_operand(AddressingMode::ZeroPage);
                let val = self.bus.mem_read(addr).wrapping_add(1);
                self.bus.mem_write(addr, val);

                // skip next byte
                self.pc += 1;

                self.check_zero(val);
                self.check_negative(val);
            }
            0xF6 /* <-- [ Zero Page, X ] --> */ => {
                // Increment value at zero page address + X
                // 2 bytes, 6 cycles
                self.sleep_cycles = 5;

                // increment value
                let addr = self.get_addr_from_operand(AddressingMode::ZeroPageX);
                let val = self.bus.mem_read(addr).wrapping_add(1);
                self.bus.mem_write(addr, val);

                // skip next byte
                self.pc += 1;

                self.check_zero(val);
                self.check_negative(val);
            }
            0xEE /* <-- [ Absolute ] --> */ => {
                // Increment value at absolute address
                // 3 bytes, 6 cycles
                self.sleep_cycles = 5;

                // increment value
                let addr = self.get_addr_from_operand(AddressingMode::Absolute);
                let val = self.bus.mem_read(addr).wrapping_add(1);
                self.bus.mem_write(addr, val);

                // skip next 2 bytes
                self.pc += 2;

                self.check_zero(val);
                self.check_negative(val);
            }
            0xFE /* <-- [ Absolute, X ] --> */ => {
                // Increment value at absolute address + X
                // 3 bytes, 7 cycles
                self.sleep_cycles = 6;

                // increment value
                let addr = self.get_addr_from_operand(AddressingMode::AbsoluteX);
                let val = self.bus.mem_read(addr).wrapping_add(1);
                self.bus.mem_write(addr, val);

                // skip next 2 bytes
                self.pc += 2;

                self.check_zero(val);
                self.check_negative(val);
            }

            // <--| INX |-->
            0xE8 /* <-- [ None ] --> */ => {
                // Increment index X
                // 1 byte, 2 cycles
                self.sleep_cycles = 1;

                self.idx_x = self.idx_x.wrapping_add(1);

                self.check_zero(self.idx_x);
                self.check_negative(self.idx_x);
            }

            // <--| INY |-->
            0xC8 /* <-- [ None ] --> */ => {
                // Increment index Y
                // 1 byte, 2 cycles
                self.sleep_cycles = 1;

                self.idx_y = self.idx_y.wrapping_add(1);

                self.check_zero(self.idx_y);
                self.check_negative(self.idx_y);
            }

            // <--| TAX |-->
            0xAA /* <-- [ None ] --> */ => {
                // Transfer accumulator to index X
                // 1 byte, 2 cycles
                self.sleep_cycles = 1;

                self.idx_x = self.acc;

                self.check_zero(self.idx_x);
                self.check_negative(self.idx_x);
            }

            // <--| TAY |-->
            0xA8 /* <-- [ None ] --> */ => {
                // Transfer accumulator to index Y
                // 1 byte, 2 cycles
                self.sleep_cycles = 1;

                self.idx_y = self.acc;

                self.check_zero(self.idx_y);
                self.check_negative(self.idx_y);
            }

            // <--| NOP |-->
            0xEA /* <-- [ None ] --> */ => {
                // No operation
                // 1 byte, 2 cycles
                self.sleep_cycles = 1;
            }


            _ => { }//unimplemented!("Opcode {:#X} not implemented", opcode)},
        }

        // Increment PC
        if !dont_increment_pc {
            self.pc += 1;
        }
    }

    // TODO: Tomorrow, complete this and try chatgpt tests, also figure out whatever page crossing is
    /// Returns the address of the operand for the current instruction, and increments the PC
    fn get_addr_from_operand(&self, mode: AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => {
                // immediate addressing mode, addr is next byte
                self.pc + 1
            }
            AddressingMode::ZeroPage => {
                // zero page addressing mode, addr is next byte's value
                self.bus.mem_read(self.pc + 1) as u16
            }
            AddressingMode::ZeroPageX => {
                // zero page X addressing mode, addr is next byte's value + X
                (self.bus.mem_read(self.pc + 1) + self.idx_x) as u16
            }
            AddressingMode::ZeroPageY => {
                // zero page Y addressing mode, addr is next byte's value + Y
                (self.bus.mem_read(self.pc + 1) + self.idx_y) as u16
            }
            AddressingMode::Absolute => {
                // absolute addressing mode, addr is next 2 bytes
                self.bus.mem_read_u16(self.pc + 1)
            }
            AddressingMode::AbsoluteX => {
                // absolute X addressing mode, addr is next 2 bytes + X
                self.bus
                    .mem_read_u16(self.pc + 1)
                    .wrapping_add(self.idx_x as u16)
            }
            AddressingMode::AbsoluteY => {
                // absolute Y addressing mode, addr is next 2 bytes + Y
                self.bus
                    .mem_read_u16(self.pc + 1)
                    .wrapping_add(self.idx_y as u16)
            }
            AddressingMode::Indirect => {
                // indirect addressing mode, addr is at data at next 2 bytes
                let oper = self.bus.mem_read_u16(self.pc + 1);
                self.bus.mem_read_u16(oper)
            }
            AddressingMode::IndirectX => {
                // indirect X addressing mode, addr is at data at (oper + X)
                let oper = self.bus.mem_read(self.pc + 1);
                self.bus.mem_read_u16((oper + self.idx_x) as u16)
            }
            AddressingMode::IndirectY => {
                // indirect Y addressing mode, addr is at data at oper, Y is added later
                let oper = self.bus.mem_read(self.pc + 1);
                self.bus
                    .mem_read_u16(oper as u16)
                    .wrapping_add(self.idx_y as u16)
            }

            _ => panic!("Invalid addressing mode"),
        }
    }

    fn check_page_cross(&mut self, addr1: u16, addr2: u16) -> bool {
        addr1 & 0xFF00 != addr2 & 0xFF00
    }
}

// stupid enum implementation, need it because rust doesn't support bitflags on enums
#[allow(non_upper_case_globals)]
#[allow(dead_code)]
#[allow(non_snake_case)]
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
    Indirect,
    IndirectX,
    IndirectY,
    NoneAddressing,
}
