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

impl CPU {
    pub fn new(bus: Rc<Bus>) -> Self {
        CPU {
            pc: 0x8000,
            sp: 0xFD,
            acc: 0,
            idx_x: 0,
            idx_y: 0,
            status: 0,
            bus,
            sleep_cycles: 0,
        }
    }

    #[inline]
    fn get_flag(&mut self, flag: u8) -> bool {
        self.status & flag != 0
    }

    #[inline]
    fn set_flag(&mut self, flag: u8) {
        self.status |= flag;
    }

    #[inline]
    fn clear_flag(&mut self, flag: u8) {
        self.status &= !flag;
    }

    #[inline]
    fn check_page_cross(&self, addr: u16) -> bool {
        if self.get_addr_from_operand(AddressingMode::Absolute) & 0xFF00 != addr & 0xFF00 {
            true
        } else {
            false
        }
    }

    #[inline]
    fn check_add_carry(&mut self, sum: u16) {
        if sum > 0xFF {
            self.set_flag(StatusFlag::Carry);
        } else {
            self.clear_flag(StatusFlag::Carry);
        }
    }

    #[inline]
    fn check_add_overflow(&mut self, sum: u8, addend_1: u8, addend_2: u8) {
        if (addend_1 ^ addend_2) & 0x80 == 0 && (addend_1 ^ sum as u8) & 0x80 != 0 {
            self.set_flag(StatusFlag::Overflow);
        } else {
            self.clear_flag(StatusFlag::Overflow);
        }
    }
    
    #[inline]
    fn check_zero(&mut self, oper: u8) {
        if oper == 0 {
            self.set_flag(StatusFlag::Zero);
        } else {
            self.clear_flag(StatusFlag::Zero);
        }
    }

    #[inline]
    fn check_negative(&mut self, oper: u8) {
        if oper & 0b1000_0000 != 0 {
            self.set_flag(StatusFlag::Negative);
        } else {
            self.clear_flag(StatusFlag::Negative);
        }
    }

    #[inline]
    fn push_stack(&mut self, oper: u8) {
        self.bus.mem_write(0x0100 + self.sp as u16, oper);
        self.sp = self.sp.wrapping_sub(1);
    }

    #[inline]
    fn pop_stack(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.bus.mem_read(0x0100 + self.sp as u16)
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
            // <--| ADC |-->
            0x69 /* <-- [ Immediate ] --> */ => {
                // Add with carry immediate
                // 2 bytes, 2 cycles
                self.sleep_cycles = 1;

                let addr = self.get_addr_from_operand(AddressingMode::Immediate);
                let oper = self.bus.mem_read(addr);

                let before_sum = self.acc;
                let sum = self.acc as u16 + oper as u16 + self.get_flag(StatusFlag::Carry) as u16;
                self.acc = sum as u8;

                // skip next byte
                self.pc += 1;
                
                self.check_add_overflow(sum as u8, before_sum, oper);
                self.check_add_carry(sum);
                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            0x65 /* <-- [ Zero Page ] --> */ => {
                // Add with carry zero page
                // 2 bytes, 3 cycles
                self.sleep_cycles = 2;

                let addr = self.get_addr_from_operand(AddressingMode::ZeroPage);
                let oper = self.bus.mem_read(addr);

                let before_sum = self.acc;
                let sum = self.acc as u16 + oper as u16 + self.get_flag(StatusFlag::Carry) as u16;
                self.acc = sum as u8;

                // skip next byte
                self.pc += 1;
                
                self.check_add_overflow(sum as u8, before_sum, oper);               
                self.check_add_carry(sum);
                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            0x75 /* <-- [ Zero Page, X ] --> */ => {
                // Add with carry zero page, X
                // 2 bytes, 4 cycles
                self.sleep_cycles = 3;

                let addr = self.get_addr_from_operand(AddressingMode::ZeroPageX);
                let oper = self.bus.mem_read(addr);

                let before_sum = self.acc;
                let sum = self.acc as u16 + oper as u16 + self.get_flag(StatusFlag::Carry) as u16;
                self.acc = sum as u8;

                // skip next byte
                self.pc += 1;
                
                self.check_add_overflow(sum as u8, before_sum, oper);               
                self.check_add_carry(sum);
                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            0x6D /* <-- [ Absolute ] --> */ => {
                // Add with carry absolute
                // 3 bytes, 4 cycles
                self.sleep_cycles = 3;

                let addr = self.get_addr_from_operand(AddressingMode::Absolute);
                let oper = self.bus.mem_read(addr);

                let before_sum = self.acc;
                let sum = self.acc as u16 + oper as u16 + self.get_flag(StatusFlag::Carry) as u16;
                self.acc = sum as u8;

                // skip next 2 bytes
                self.pc += 2;
                
                self.check_add_overflow(sum as u8, before_sum, oper);               
                self.check_add_carry(sum);
                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            0x7D /* <-- [ Absolute, X ] --> */ => {
                // Add with carry absolute, X
                // 3 bytes, 4 cycles (+1 if page crossed)
                self.sleep_cycles = 3;

                let addr = self.get_addr_from_operand(AddressingMode::AbsoluteX);
                let oper = self.bus.mem_read(addr);

                let before_sum = self.acc;
                let sum = self.acc as u16 + oper as u16 + self.get_flag(StatusFlag::Carry) as u16;
                self.acc = sum as u8;

                // skip next 2 bytes
                self.pc += 2;

                // if page crossed, add 1 cycle
                if self.check_page_cross(addr) {
                    self.sleep_cycles += 1;
                }
                
                self.check_add_overflow(sum as u8, before_sum, oper);               
                self.check_add_carry(sum);
                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            0x79 /* <-- [ Absolute, Y ] --> */ => {
                // Add with carry absolute, Y
                // 3 bytes, 4 cycles (+1 if page crossed)
                self.sleep_cycles = 3;

                let addr = self.get_addr_from_operand(AddressingMode::AbsoluteY);
                let oper = self.bus.mem_read(addr);

                let before_sum = self.acc;
                let sum = self.acc as u16 + oper as u16 + self.get_flag(StatusFlag::Carry) as u16;
                self.acc = sum as u8;

                // skip next 2 bytes
                self.pc += 2;

                // if page crossed, add 1 cycle
                if self.check_page_cross(addr) {
                    self.sleep_cycles += 1;
                }
                
                self.check_add_overflow(sum as u8, before_sum, oper);               
                self.check_add_carry(sum);
                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            0x61 /* <-- [ Indirect, X ] --> */ => {
                // Add with carry indirect, X
                // 2 bytes, 6 cycles
                self.sleep_cycles = 5;

                let addr = self.get_addr_from_operand(AddressingMode::IndirectX);
                let oper = self.bus.mem_read(addr);

                let before_sum = self.acc;
                let sum = self.acc as u16 + oper as u16 + self.get_flag(StatusFlag::Carry) as u16;
                self.acc = sum as u8;

                // skip next byte
                self.pc += 1;
                
                self.check_add_overflow(sum as u8, before_sum, oper);               
                self.check_add_carry(sum);
                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            0x71 /* <-- [ Indirect, Y ] --> */ => {
                // Add with carry indirect, Y
                // 2 bytes, 5 cycles (+1 if page crossed)
                self.sleep_cycles = 4;

                let addr = self.get_addr_from_operand(AddressingMode::IndirectY);
                let oper = self.bus.mem_read(addr);

                let before_sum = self.acc;
                let sum = self.acc as u16 + oper as u16 + self.get_flag(StatusFlag::Carry) as u16;
                self.acc = sum as u8;

                // skip next byte
                self.pc += 1;

                // if page crossed, add 1 cycle
                if self.check_page_cross(addr) {
                    self.sleep_cycles += 1;
                }
                
                self.check_add_overflow(sum as u8, before_sum, oper);               
                self.check_add_carry(sum);
                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }

            // <--| LDA |-->
            0xA9 /* <-- [ Immediate ] --> */ => {
                // Load accumulator with immediate oper
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
                // Load accumulator with zero page oper
                // 2 bytes, 3 cycles
                self.sleep_cycles = 2;

                // set accumulator to oper
                let addr = self.get_addr_from_operand(AddressingMode::ZeroPage);
                self.acc = self.bus.mem_read(addr);

                // skip next byte
                self.pc += 1;

                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            0xB5 /* <-- [ Zero Page, X ] --> */ => {
                // Load accumulator with zero page oper
                // 2 bytes, 4 cycles
                self.sleep_cycles = 3;

                // set accumulator to oper
                let addr = self.get_addr_from_operand(AddressingMode::ZeroPageX);
                self.acc = self.bus.mem_read(addr);

                // skip next byte
                self.pc += 1;

                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            0xAD /* <-- [ Absolute ] --> */ => {
                // Load accumulator with data at absolute oper
                // 3 bytes, 4 cycles
                self.sleep_cycles = 3;

                // set accumulator to oper
                let addr = self.get_addr_from_operand(AddressingMode::Absolute);
                self.acc = self.bus.mem_read(addr);

                // skip next 2 bytes
                self.pc += 2;

                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            0xBD /* <-- [ Absolute, X ] --> */ => {
                // Load accumulator with data at absolute oper + X
                // 3 bytes, 4 cycles
                self.sleep_cycles = 3;

                // set accumulator to oper
                let addr = self.get_addr_from_operand(AddressingMode::AbsoluteX);
                self.acc = self.bus.mem_read(addr);

                // if page crossed, add 1 cycle
                if self.check_page_cross(addr) {
                    self.sleep_cycles += 1;
                }

                // skip next 2 bytes
                self.pc += 2;

                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            0xB9 /* <-- [ Absolute, Y ] --> */ => {
                // Load accumulator with data at absolute oper + X
                // 3 bytes, 4 cycles
                self.sleep_cycles = 3;

                // set accumulator to oper
                let addr = self.get_addr_from_operand(AddressingMode::AbsoluteY);
                self.acc = self.bus.mem_read(addr);

                // if page crossed, add 1 cycle
                if self.check_page_cross(addr) {
                    self.sleep_cycles += 1;
                }

                // skip next 2 bytes
                self.pc += 2;

                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            0xA1 /* <-- [ Indirect, X ] --> */ => {
                // Load accumulator with data at indirect oper + X
                // 2 bytes, 6 cycles
                self.sleep_cycles = 5;

                // set accumulator to oper
                let addr = self.get_addr_from_operand(AddressingMode::IndirectX);
                self.acc = self.bus.mem_read(addr);

                // skip next byte
                self.pc += 1;

                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            0xB1 /* <-- [ Indirect, Y ] --> */ => {
                // Load accumulator with data at indirect oper + Y
                // 2 bytes, 5 cycles
                self.sleep_cycles = 4;

                // set accumulator to oper
                let addr = self.get_addr_from_operand(AddressingMode::IndirectY);
                self.acc = self.bus.mem_read(addr);

                // if page crossed, add 1 cycle
                if self.check_page_cross(addr) {
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

                // set PC to oper
                let addr = self.get_addr_from_operand(AddressingMode::Absolute);
                self.pc = self.bus.mem_read_u16(addr);

                // PC is incremented at end of cycle
                dont_increment_pc = true;
            }
            0x6C /* <-- [ Indirect ] --> */ => {
                // Jump to indirect address
                // 3 bytes, 5 cycles
                self.sleep_cycles = 4;

                // set PC to oper
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
                // Increment oper at zero page address
                // 2 bytes, 5 cycles
                self.sleep_cycles = 4;

                // increment oper
                let addr = self.get_addr_from_operand(AddressingMode::ZeroPage);
                let val = self.bus.mem_read(addr).wrapping_add(1);
                self.bus.mem_write(addr, val);

                // skip next byte
                self.pc += 1;

                self.check_zero(val);
                self.check_negative(val);
            }
            0xF6 /* <-- [ Zero Page, X ] --> */ => {
                // Increment oper at zero page address + X
                // 2 bytes, 6 cycles
                self.sleep_cycles = 5;

                // increment oper
                let addr = self.get_addr_from_operand(AddressingMode::ZeroPageX);
                let val = self.bus.mem_read(addr).wrapping_add(1);
                self.bus.mem_write(addr, val);

                // skip next byte
                self.pc += 1;

                self.check_zero(val);
                self.check_negative(val);
            }
            0xEE /* <-- [ Absolute ] --> */ => {
                // Increment oper at absolute address
                // 3 bytes, 6 cycles
                self.sleep_cycles = 5;

                // increment oper
                let addr = self.get_addr_from_operand(AddressingMode::Absolute);
                let val = self.bus.mem_read(addr).wrapping_add(1);
                self.bus.mem_write(addr, val);

                // skip next 2 bytes
                self.pc += 2;

                self.check_zero(val);
                self.check_negative(val);
            }
            0xFE /* <-- [ Absolute, X ] --> */ => {
                // Increment oper at absolute address + X
                // 3 bytes, 7 cycles
                self.sleep_cycles = 6;

                // increment oper
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

            // <--| JSR |-->
            0x20 /* <-- [ Absolute ] --> */ => {
                // Jump to subroutine
                // 3 bytes, 6 cycles
                self.sleep_cycles = 5;

                // push PC to stack in little endian
                self.push_stack((self.pc & 0xFF) as u8); // low byte
                self.push_stack((self.pc >> 8) as u8); // high byte

                // set PC to address
                self.pc = self.get_addr_from_operand(AddressingMode::Absolute);

                dont_increment_pc = true;
            }

            // <--| RTS |-->
            0x60 /* <-- [ None ] --> */ => {
                // Return from subroutine
                // 1 byte, 6 cycles
                self.sleep_cycles = 5;

                // pop PC from stack, it is in little endian from front to back
                let high_byte = self.pop_stack();
                let low_byte = self.pop_stack();
                self.pc = ((high_byte as u16) << 8) | (low_byte as u16);

                // PC will be incremented after this, so no need to +1
            }



            _ => { }//unimplemented!("Opcode {:#X} not implemented", opcode)},
        }

        // Increment PC
        if !dont_increment_pc {
            self.pc += 1;
        }
    }

    /// Returns the address of the operand for the current instruction, and increments the PC
    fn get_addr_from_operand(&self, mode: AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => {
                // immediate addressing mode, addr is next byte
                self.pc + 1
            }
            AddressingMode::ZeroPage => {
                // zero page addressing mode, addr is next byte's oper
                self.bus.mem_read(self.pc + 1) as u16
            }
            AddressingMode::ZeroPageX => {
                // zero page X addressing mode, addr is next byte's oper + X
                (self.bus.mem_read(self.pc + 1) + self.idx_x) as u16
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
        }
    }
}

// stupid enum implementation, need it because rust doesn't support bitflags on enums
#[allow(non_upper_case_globals)]
#[allow(dead_code)]
#[allow(non_snake_case)]
mod StatusFlag {
    pub const Negative: u8 =         0b1000_0000;
    pub const Overflow: u8 =         0b0100_0000;
    pub const Unused: u8 =           0b0010_0000;
    pub const Break: u8 =            0b0001_0000;
    pub const DecimalMode: u8 =      0b0000_1000;
    pub const InterruptDisable: u8 = 0b0000_0100;
    pub const Zero: u8 =             0b0000_0010;
    pub const Carry: u8 =            0b0000_0001;
}

enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
}
