use std::rc::Rc;

use super::bus::Bus;

pub struct CPU {
    pub pc: u16,
    pub sp: u8,
    pub acc: u8,
    pub idx_x: u8,
    pub idx_y: u8,
    pub status: u8,
    pub debug_mode: bool,
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
            debug_mode: false,
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
    fn check_page_cross(&self, addr_1: u16, addr_2: u16) -> bool {
        if addr_1 & 0xFF00 != addr_2 & 0xFF00 {
            if self.debug_mode { print!("Page crossed | ") };
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
    pub fn push_stack(&mut self, oper: u8) {
        self.bus.mem_write(0x0100 + self.sp as u16, oper);
        self.sp = self.sp.wrapping_sub(1);
    }

    #[inline]
    pub fn pop_stack(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let res = self.bus.mem_read(0x0100 + self.sp as u16);
        res
    }

    pub fn cycle(&mut self) {
        if self.debug_mode { print!("{:04X} | ", self.pc) };
        // sleep for cycles until sleep_cycles is 0
        if self.sleep_cycles > 0 {
            self.sleep_cycles -= 1;
            if self.debug_mode { println!("/\\ sleeping") };
            return;
        }
        
        // Fetch
        let opcode = self.bus.mem_read(self.pc);
        let mut dont_increment_pc = false;

        // Decode, Execute
        // TODO: Add debug mode prints
        match opcode {
            // <--| ADC |-->
            0x69 /* <-- [ Immediate ] --> */ => {
                // Add with carry immediate
                // 2 bytes, 2 cycles
                self.sleep_cycles = 1;

                if self.debug_mode { print!("ADC: Immediate | "); }

                let addr = self.get_addr(AddressingMode::Immediate);
                let value = self.bus.mem_read(addr);

                let before_sum = self.acc;
                let sum = self.acc as u16 + value as u16 + self.get_flag(StatusFlag::Carry) as u16;
                self.acc = sum as u8;

                // skip next byte
                self.pc += 1;
                
                self.check_add_overflow(sum as u8, before_sum, value);
                self.check_add_carry(sum);
                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            0x65 /* <-- [ Zero Page ] --> */ => {
                // Add with carry zero page
                // 2 bytes, 3 cycles
                self.sleep_cycles = 2;

                if self.debug_mode { print!("ADC: Zero Page | "); }

                let addr = self.get_addr(AddressingMode::ZeroPage);
                let value = self.bus.mem_read(addr);

                let before_sum = self.acc;
                let sum = self.acc as u16 + value as u16 + self.get_flag(StatusFlag::Carry) as u16;
                self.acc = sum as u8;

                // skip next byte
                self.pc += 1;
                
                self.check_add_overflow(sum as u8, before_sum, value);               
                self.check_add_carry(sum);
                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            0x75 /* <-- [ Zero Page, X ] --> */ => {
                // Add with carry zero page, X
                // 2 bytes, 4 cycles
                self.sleep_cycles = 3;

                if self.debug_mode { print!("ADC: Zero Page, X | "); }

                let addr = self.get_addr(AddressingMode::ZeroPageX);
                let value = self.bus.mem_read(addr);

                let before_sum = self.acc;
                let sum = self.acc as u16 + value as u16 + self.get_flag(StatusFlag::Carry) as u16;
                self.acc = sum as u8;

                // skip next byte
                self.pc += 1;
                
                self.check_add_overflow(sum as u8, before_sum, value);               
                self.check_add_carry(sum);
                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            0x6D /* <-- [ Absolute ] --> */ => {
                // Add with carry absolute
                // 3 bytes, 4 cycles
                self.sleep_cycles = 3;

                if self.debug_mode { print!("ADC: Absolute | "); }

                let addr = self.get_addr(AddressingMode::Absolute);
                let value = self.bus.mem_read(addr);

                let before_sum = self.acc;
                let sum = self.acc as u16 + value as u16 + self.get_flag(StatusFlag::Carry) as u16;
                self.acc = sum as u8;

                // skip next 2 bytes
                self.pc += 2;
                
                self.check_add_overflow(sum as u8, before_sum, value);               
                self.check_add_carry(sum);
                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            0x7D /* <-- [ Absolute, X ] --> */ => {
                // Add with carry absolute, X
                // 3 bytes, 4 cycles (+1 if page crossed)
                self.sleep_cycles = 3;

                if self.debug_mode { print!("ADC: Absolute, X | "); }

                let addr = self.get_addr(AddressingMode::AbsoluteX);
                let value = self.bus.mem_read(addr);

                let before_sum = self.acc;
                let sum = self.acc as u16 + value as u16 + self.get_flag(StatusFlag::Carry) as u16;
                self.acc = sum as u8;

                // if page crossed, add 1 cycle
                if self.check_page_cross(addr, self.get_addr(AddressingMode::Absolute)) {
                    self.sleep_cycles += 1;
                }
                
                // skip next 2 bytes
                self.pc += 2;
                
                self.check_add_overflow(sum as u8, before_sum, value);               
                self.check_add_carry(sum);
                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            0x79 /* <-- [ Absolute, Y ] --> */ => {
                // Add with carry absolute, Y
                // 3 bytes, 4 cycles (+1 if page crossed)
                self.sleep_cycles = 3;

                if self.debug_mode { print!("ADC: Absolute, Y | "); }

                let addr = self.get_addr(AddressingMode::AbsoluteY);
                let value = self.bus.mem_read(addr);

                let before_sum = self.acc;
                let sum = self.acc as u16 + value as u16 + self.get_flag(StatusFlag::Carry) as u16;
                self.acc = sum as u8;

                // if page crossed, add 1 cycle
                if self.check_page_cross(addr, self.get_addr(AddressingMode::Absolute)) {
                    self.sleep_cycles += 1;
                }
                
                // skip next 2 bytes
                self.pc += 2;
                
                self.check_add_overflow(sum as u8, before_sum, value);               
                self.check_add_carry(sum);
                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            0x61 /* <-- [ Indirect, X ] --> */ => {
                // Add with carry indirect, X
                // 2 bytes, 6 cycles
                self.sleep_cycles = 5;

                if self.debug_mode { print!("ADC: Indirect, X | "); }

                let addr = self.get_addr(AddressingMode::IndirectX);
                let value = self.bus.mem_read(addr);

                let before_sum = self.acc;
                let sum = self.acc as u16 + value as u16 + self.get_flag(StatusFlag::Carry) as u16;
                self.acc = sum as u8;

                // skip next byte
                self.pc += 1;
                
                self.check_add_overflow(sum as u8, before_sum, value);               
                self.check_add_carry(sum);
                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            0x71 /* <-- [ Indirect, Y ] --> */ => {
                // Add with carry indirect, Y
                // 2 bytes, 5 cycles (+1 if page crossed)
                self.sleep_cycles = 4;

                if self.debug_mode { print!("ADC: Indirect, Y | "); }


                let addr = self.get_addr(AddressingMode::IndirectY);
                let value = self.bus.mem_read(addr);

                let before_sum = self.acc;
                let sum = self.acc as u16 + value as u16 + self.get_flag(StatusFlag::Carry) as u16;
                self.acc = sum as u8;

                // if page crossed, add 1 cycle
                if self.check_page_cross(addr, self.get_addr(AddressingMode::Indirect)) {
                    self.sleep_cycles += 1;
                }
                
                // skip next byte
                self.pc += 1;
                
                self.check_add_overflow(sum as u8, before_sum, value);               
                self.check_add_carry(sum);
                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }

            // <--| AND |-->
            0x29 /* <-- [ Immediate ] --> */ => {
                // Logical AND immediate
                // 2 bytes, 2 cycles
                self.sleep_cycles = 1;

                if self.debug_mode { print!("AND: Immediate | "); }

                let value = self.bus.mem_read(self.pc + 1);
                self.acc &= value;

                // skip next byte
                self.pc += 1;

                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            0x25 /* <-- [ Zero Page ] --> */ => {
                // Logical AND zero page
                // 2 bytes, 3 cycles
                self.sleep_cycles = 2;

                if self.debug_mode { print!("AND: Zero Page | "); }

                let addr = self.get_addr(AddressingMode::ZeroPage);
                let value = self.bus.mem_read(addr);
                self.acc &= value;

                // skip next byte
                self.pc += 1;

                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            0x35 /* <-- [ Zero Page, X ] --> */ => {
                // Logical AND zero page, X
                // 2 bytes, 4 cycles
                self.sleep_cycles = 3;

                if self.debug_mode { print!("AND: Zero Page, X | "); }

                let addr = self.get_addr(AddressingMode::ZeroPageX);
                let value = self.bus.mem_read(addr);
                self.acc &= value;

                // skip next byte
                self.pc += 1;

                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            0x2D /* <-- [ Absolute ] --> */ => {
                // Logical AND absolute
                // 3 bytes, 4 cycles
                self.sleep_cycles = 3;

                if self.debug_mode { print!("AND: Absolute | "); }

                let addr = self.get_addr(AddressingMode::Absolute);
                let value = self.bus.mem_read(addr);
                self.acc &= value;

                // skip next 2 bytes
                self.pc += 2;

                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            0x3D /* <-- [ Absolute, X ] --> */ => {
                // Logical AND absolute, X
                // 3 bytes, 4 cycles (+1 if page crossed)
                self.sleep_cycles = 3;

                if self.debug_mode { print!("AND: Absolute, X | "); }

                let addr = self.get_addr(AddressingMode::AbsoluteX);
                let value = self.bus.mem_read(addr);
                self.acc &= value;

                // if page crossed, add 1 cycle
                if self.check_page_cross(addr, self.get_addr(AddressingMode::Absolute)) {
                    self.sleep_cycles += 1;
                }

                // skip next 2 bytes
                self.pc += 2;

                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            0x39 /* <-- [ Absolute, Y ] --> */ => {
                // Logical AND absolute, Y
                // 3 bytes, 4 cycles (+1 if page crossed)
                self.sleep_cycles = 3;

                if self.debug_mode { print!("AND: Absolute, Y | "); }

                let addr = self.get_addr(AddressingMode::AbsoluteY);
                let value = self.bus.mem_read(addr);
                self.acc &= value;

                // if page crossed, add 1 cycle
                if self.check_page_cross(addr, self.get_addr(AddressingMode::Absolute)) {
                    self.sleep_cycles += 1;
                }

                // skip next 2 bytes
                self.pc += 2;

                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            0x21 /* <-- [ Indirect, X ] --> */ => {
                // Logical AND indirect, X
                // 2 bytes, 6 cycles
                self.sleep_cycles = 5;

                if self.debug_mode { print!("AND: Indirect, X | "); }

                let addr = self.get_addr(AddressingMode::IndirectX);
                let value = self.bus.mem_read(addr);
                self.acc &= value;

                // skip next byte
                self.pc += 1;

                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            0x31 /* <-- [ Indirect, Y ] --> */ => {
                // Logical AND indirect, Y
                // 2 bytes, 5 cycles (+1 if page crossed)
                self.sleep_cycles = 4;

                if self.debug_mode { print!("AND: Indirect, Y | "); }

                let addr = self.get_addr(AddressingMode::IndirectY);
                let value = self.bus.mem_read(addr);
                self.acc &= value;

                // if page crossed, add 1 cycle
                if self.check_page_cross(addr, self.get_addr(AddressingMode::Indirect)) {
                    self.sleep_cycles += 1;
                }

                // skip next byte
                self.pc += 1;

                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }

            // <--| ASL |-->
            0x0A /* <-- [ Accumulator ] --> */ => {
                // Arithmetic shift left accumulator
                // 1 byte, 2 cycles
                self.sleep_cycles = 1;

                if self.debug_mode { print!("ASL: Accumulator | "); }

                if self.acc & 0x80 != 0 {
                    self.set_flag(StatusFlag::Carry);
                } else {
                    self.clear_flag(StatusFlag::Carry);
                }

                self.acc = self.acc << 1;

                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            0x06 /* <-- [ Zero Page ] --> */ => {
                // Arithmetic shift left zero page
                // 2 bytes, 5 cycles
                self.sleep_cycles = 4;

                if self.debug_mode { print!("ASL: Zero Page | "); }

                let addr = self.get_addr(AddressingMode::ZeroPage);
                let mut value = self.bus.mem_read(addr);

                if value & 0x80 != 0 {
                    self.set_flag(StatusFlag::Carry);
                } else {
                    self.clear_flag(StatusFlag::Carry);
                }

                value = value << 1;
                self.bus.mem_write(addr, value);

                // skip next byte
                self.pc += 1;

                self.check_zero(value);
                self.check_negative(value);
            }
            0x16 /* <-- [ Zero Page, X ] --> */ => {
                // Arithmetic shift left zero page, X
                // 2 bytes, 6 cycles
                self.sleep_cycles = 5;

                if self.debug_mode { print!("ASL: Zero Page, X | "); }

                let addr = self.get_addr(AddressingMode::ZeroPageX);
                let mut value = self.bus.mem_read(addr);

                if value & 0x80 != 0 {
                    self.set_flag(StatusFlag::Carry);
                } else {
                    self.clear_flag(StatusFlag::Carry);
                }

                value = value << 1;
                self.bus.mem_write(addr, value);

                // skip next byte
                self.pc += 1;

                self.check_zero(value);
                self.check_negative(value);
            }
            0x0E /* <-- [ Absolute ] --> */ => {
                // Arithmetic shift left absolute
                // 3 bytes, 6 cycles
                self.sleep_cycles = 5;

                if self.debug_mode { print!("ASL: Absolute | "); }

                let addr = self.get_addr(AddressingMode::Absolute);
                let mut value = self.bus.mem_read(addr);

                if value & 0x80 != 0 {
                    self.set_flag(StatusFlag::Carry);
                } else {
                    self.clear_flag(StatusFlag::Carry);
                }

                value = value << 1;
                self.bus.mem_write(addr, value);

                // skip next 2 bytes
                self.pc += 2;

                self.check_zero(value);
                self.check_negative(value);
            }
            0x1E /* <-- [ Absolute, X ] --> */ => {
                // Arithmetic shift left absolute, X
                // 3 bytes, 7 cycles
                self.sleep_cycles = 6;

                if self.debug_mode { print!("ASL: Absolute, X | "); }

                let addr = self.get_addr(AddressingMode::AbsoluteX);
                let mut value = self.bus.mem_read(addr);

                if value & 0x80 != 0 {
                    self.set_flag(StatusFlag::Carry);
                } else {
                    self.clear_flag(StatusFlag::Carry);
                }

                value = value << 1;
                self.bus.mem_write(addr, value);

                // skip next 2 bytes
                self.pc += 2;

                self.check_zero(value);
                self.check_negative(value);
            }

            // // <--| BRK |-->
            // 0x00 /* <-- [ Implied ] --> */ => {
            //     // Force interrupt
            //     // 1 byte, 7 cycles
            //     self.sleep_cycles = 6;

            //     if self.debug_mode { print!("BRK: Implied | "); }

            //     self.set_flag(StatusFlag::Break);
            //     self.push_stack(((self.pc + 2) >> 8) as u8);
            //     self.push_stack((self.pc + 2) as u8);
            //     self.push_stack(self.status | 0b00110000);
            //     self.set_flag(StatusFlag::Interrupt);
            //     self.pc = self.bus.mem_read_u16(0xFFFE);

            //     dont_increment_pc = true;
            // }

            // // <--| RTI |-->
            // 0x40 /* <-- [ Implied ] --> */ => {
            //     // Return from interrupt
            //     // 1 byte, 6 cycles
            //     self.sleep_cycles = 5;

            //     if self.debug_mode { print!("RTI: Implied | "); }

            //     self.status = self.pop_stack();
            //     self.pc = self.pop_stack() as u16 | ((self.pop_stack() as u16) << 8);

            //     dont_increment_pc = true;
            // }

            // <--| CLC |-->
            0x18 /* <-- [ Implied ] --> */ => {
                // Clear carry flag
                // 1 byte, 2 cycles
                self.sleep_cycles = 1;

                if self.debug_mode { print!("CLC: Implied | "); }

                self.clear_flag(StatusFlag::Carry);
            }

            // <--| CLD |-->
            0xD8 /* <-- [ Implied ] --> */ => {
                // Clear decimal flag
                // 1 byte, 2 cycles
                self.sleep_cycles = 1;

                if self.debug_mode { print!("CLD: Implied | "); }

                self.clear_flag(StatusFlag::Decimal);
            }

            // <--| CLI |-->
            0x58 /* <-- [ Implied ] --> */ => {
                // Clear interrupt flag
                // 1 byte, 2 cycles
                self.sleep_cycles = 1;

                if self.debug_mode { print!("CLI: Implied | "); }

                self.clear_flag(StatusFlag::Interrupt);
            }

            // <--| CLV |-->
            0xB8 /* <-- [ Implied ] --> */ => {
                // Clear overflow flag
                // 1 byte, 2 cycles
                self.sleep_cycles = 1;

                if self.debug_mode { print!("CLV: Implied | "); }

                self.clear_flag(StatusFlag::Overflow);
            }

            // <--| DEC |-->
            0xC6 /* <-- [ Zero Page ] --> */ => {
                // Decrement zero page
                // 2 bytes, 5 cycles
                self.sleep_cycles = 4;

                if self.debug_mode { print!("DEC: Zero Page | "); }

                let addr = self.get_addr(AddressingMode::ZeroPage);
                let value = self.bus.mem_read(addr);

                let res = value.wrapping_sub(1);
                self.bus.mem_write(addr, res);

                // skip next byte
                self.pc += 1;

                self.check_zero(res);
                self.check_negative(res);
            }
            0xD6 /* <-- [ Zero Page, X ] --> */ => {
                // Decrement zero page, X
                // 2 bytes, 6 cycles
                self.sleep_cycles = 5;

                if self.debug_mode { print!("DEC: Zero Page, X | "); }

                let addr = self.get_addr(AddressingMode::ZeroPageX);
                let value = self.bus.mem_read(addr);

                let dec = value.wrapping_sub(1);
                self.bus.mem_write(addr, dec);

                // skip next byte
                self.pc += 1;

                self.check_zero(dec);
                self.check_negative(dec);
            }
            0xCE /* <-- [ Absolute ] --> */ => {
                // Decrement absolute
                // 3 bytes, 6 cycles
                self.sleep_cycles = 5;

                if self.debug_mode { print!("DEC: Absolute | "); }

                let addr = self.get_addr(AddressingMode::Absolute);
                let value = self.bus.mem_read(addr);

                let dec = value.wrapping_sub(1);
                self.bus.mem_write(addr, dec);

                // skip next 2 bytes
                self.pc += 2;

                self.check_zero(dec);
                self.check_negative(dec);
            }
            0xDE /* <-- [ Absolute, X ] --> */ => {
                // Decrement absolute, X
                // 3 bytes, 7 cycles
                self.sleep_cycles = 6;

                if self.debug_mode { print!("DEC: Absolute, X | "); }

                let addr = self.get_addr(AddressingMode::AbsoluteX);
                let value = self.bus.mem_read(addr);

                let dec = value.wrapping_sub(1);
                self.bus.mem_write(addr, dec);

                // skip next 2 bytes
                self.pc += 2;

                self.check_zero(dec);
                self.check_negative(dec);
            }

            // <--| DEX |-->
            0xCA /* <-- [ Implied ] --> */ => {
                // Decrement X
                // 1 byte, 2 cycles
                self.sleep_cycles = 1;

                if self.debug_mode { print!("DEX: Implied | "); }

                self.idx_x = self.idx_x.wrapping_sub(1);

                self.check_zero(self.idx_x);
                self.check_negative(self.idx_x);
            }

            // <--| DEY |-->
            0x88 /* <-- [ Implied ] --> */ => {
                // Decrement Y
                // 1 byte, 2 cycles
                self.sleep_cycles = 1;

                if self.debug_mode { print!("DEY: Implied | "); }

                self.idx_y = self.idx_y.wrapping_sub(1);

                self.check_zero(self.idx_y);
                self.check_negative(self.idx_y);
            }

            // <--| LDA |-->
            0xA9 /* <-- [ Immediate ] --> */ => {
                // Load accumulator with immediate value
                // 2 bytes, 2 cycles
                self.sleep_cycles = 1;

                if self.debug_mode { print!("LDA: Immediate | "); }

                let addr = self.get_addr(AddressingMode::Immediate);
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

                if self.debug_mode { print!("LDA: Zero Page | "); }

                // set accumulator to value
                let addr = self.get_addr(AddressingMode::ZeroPage);
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

                if self.debug_mode { print!("LDA: Zero Page, X | "); }

                // set accumulator to value
                let addr = self.get_addr(AddressingMode::ZeroPageX);
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

                if self.debug_mode { print!("LDA: Absolute | "); }

                // set accumulator to value
                let addr = self.get_addr(AddressingMode::Absolute);
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

                if self.debug_mode { print!("LDA: Absolute, X | "); }

                // set accumulator to value
                let addr = self.get_addr(AddressingMode::AbsoluteX);
                self.acc = self.bus.mem_read(addr);
                
                // if page crossed, add 1 cycle
                if self.check_page_cross(addr, self.get_addr(AddressingMode::Absolute)) {
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
                let addr = self.get_addr(AddressingMode::AbsoluteY);
                self.acc = self.bus.mem_read(addr);
                
                // if page crossed, add 1 cycle
                if self.check_page_cross(addr, self.get_addr(AddressingMode::Absolute)) {
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
                let addr = self.get_addr(AddressingMode::IndirectX);
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
                let addr = self.get_addr(AddressingMode::IndirectY);
                self.acc = self.bus.mem_read(addr);

                // if page crossed, add 1 cycle
                if self.check_page_cross(addr, self.get_addr(AddressingMode::Indirect)) {
                    self.sleep_cycles += 1;
                }
                
                // skip next byte
                self.pc += 1;
                
                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }

            // <--| LDX |-->
            0xA2 /* <-- [ Immediate ] --> */ => {
                // Load X with immediate value
                // 2 bytes, 2 cycles
                self.sleep_cycles = 1;

                if self.debug_mode { print!("LDX: Immediate | "); }

                let addr = self.get_addr(AddressingMode::Immediate);
                self.idx_x = self.bus.mem_read(addr);

                // skip next byte
                self.pc += 1;

                self.check_zero(self.idx_x);
                self.check_negative(self.idx_x);
            }
            0xA6 /* <-- [ Zero Page ] --> */ => {
                // Load X with zero page value
                // 2 bytes, 3 cycles
                self.sleep_cycles = 2;

                if self.debug_mode { print!("LDX: Zero Page | "); }

                let addr = self.get_addr(AddressingMode::ZeroPage);
                self.idx_x = self.bus.mem_read(addr);

                // skip next byte
                self.pc += 1;

                self.check_zero(self.idx_x);
                self.check_negative(self.idx_x);
            }
            0xB6 /* <-- [ Zero Page, Y ] --> */ => {
                // Load X with zero page value + Y
                // 2 bytes, 4 cycles
                self.sleep_cycles = 3;

                if self.debug_mode { print!("LDX: Zero Page, Y | "); }

                let addr = self.get_addr(AddressingMode::ZeroPageY);
                self.idx_x = self.bus.mem_read(addr);

                // skip next byte
                self.pc += 1;

                self.check_zero(self.idx_x);
                self.check_negative(self.idx_x);
            }
            0xAE /* <-- [ Absolute ] --> */ => {
                // Load X with absolute value
                // 3 bytes, 4 cycles
                self.sleep_cycles = 3;

                if self.debug_mode { print!("LDX: Absolute | "); }

                let addr = self.get_addr(AddressingMode::Absolute);
                self.idx_x = self.bus.mem_read(addr);

                // skip next 2 bytes
                self.pc += 2;

                self.check_zero(self.idx_x);
                self.check_negative(self.idx_x);
            }
            0xBE /* <-- [ Absolute, Y ] --> */ => {
                // Load X with absolute value + Y
                // 3 bytes, 4 cycles
                self.sleep_cycles = 3;

                if self.debug_mode { print!("LDX: Absolute, Y | "); }

                let addr = self.get_addr(AddressingMode::AbsoluteY);
                self.idx_x = self.bus.mem_read(addr);

                // if page crossed, add 1 cycle
                if self.check_page_cross(addr, self.get_addr(AddressingMode::Absolute)) {
                    self.sleep_cycles += 1;
                }

                // skip next 2 bytes
                self.pc += 2;

                self.check_zero(self.idx_x);
                self.check_negative(self.idx_x);
            }

            // <--| LDY |-->
            0xA0 /* <-- [ Immediate ] --> */ => {
                // Load Y with immediate value
                // 2 bytes, 2 cycles
                self.sleep_cycles = 1;

                if self.debug_mode { print!("LDY: Immediate | "); }

                let addr = self.get_addr(AddressingMode::Immediate);
                self.idx_y = self.bus.mem_read(addr);

                // skip next byte
                self.pc += 1;

                self.check_zero(self.idx_y);
                self.check_negative(self.idx_y);
            }
            0xA4 /* <-- [ Zero Page ] --> */ => {
                // Load Y with zero page value
                // 2 bytes, 3 cycles
                self.sleep_cycles = 2;

                if self.debug_mode { print!("LDY: Zero Page | "); }

                let addr = self.get_addr(AddressingMode::ZeroPage);
                self.idx_y = self.bus.mem_read(addr);

                // skip next byte
                self.pc += 1;

                self.check_zero(self.idx_y);
                self.check_negative(self.idx_y);
            }
            0xB4 /* <-- [ Zero Page, X ] --> */ => {
                // Load Y with zero page value + X
                // 2 bytes, 4 cycles
                self.sleep_cycles = 3;

                if self.debug_mode { print!("LDY: Zero Page, X | "); }

                let addr = self.get_addr(AddressingMode::ZeroPageX);
                self.idx_y = self.bus.mem_read(addr);

                // skip next byte
                self.pc += 1;

                self.check_zero(self.idx_y);
                self.check_negative(self.idx_y);
            }
            0xAC /* <-- [ Absolute ] --> */ => {
                // Load Y with absolute value
                // 3 bytes, 4 cycles
                self.sleep_cycles = 3;

                if self.debug_mode { print!("LDY: Absolute | "); }

                let addr = self.get_addr(AddressingMode::Absolute);
                self.idx_y = self.bus.mem_read(addr);

                // skip next 2 bytes
                self.pc += 2;

                self.check_zero(self.idx_y);
                self.check_negative(self.idx_y);
            }
            0xBC /* <-- [ Absolute, X ] --> */ => {
                // Load Y with absolute value + X
                // 3 bytes, 4 cycles
                self.sleep_cycles = 3;

                if self.debug_mode { print!("LDY: Absolute, X | "); }

                let addr = self.get_addr(AddressingMode::AbsoluteX);
                self.idx_y = self.bus.mem_read(addr);

                // if page crossed, add 1 cycle
                if self.check_page_cross(addr, self.get_addr(AddressingMode::Absolute)) {
                    self.sleep_cycles += 1;
                }

                // skip next 2 bytes
                self.pc += 2;

                self.check_zero(self.idx_y);
                self.check_negative(self.idx_y);
            }

            // <--| LSR |-->
            0x4A /* <-- [ Accumulator ] --> */ => {
                // Shift right accumulator
                // 1 byte, 2 cycles
                self.sleep_cycles = 1;

                if self.debug_mode { print!("LSR: Accumulator | "); }

                if self.acc & 0x1 == 1 {
                    self.set_flag(StatusFlag::Carry);
                } else {
                    self.clear_flag(StatusFlag::Carry);
                }
                
                self.acc >>= 1;
                
                self.check_zero(self.acc);
                self.clear_flag(StatusFlag::Negative);
            }
            0x46 /* <-- [ Zero Page ] --> */ => {
                // Shift right zero page value
                // 2 bytes, 5 cycles
                self.sleep_cycles = 4;

                if self.debug_mode { print!("LSR: Zero Page | "); }

                let addr = self.get_addr(AddressingMode::ZeroPage);
                let mut val = self.bus.mem_read(addr);

                if val & 0x1 == 1 {
                    self.set_flag(StatusFlag::Carry);
                } else {
                    self.clear_flag(StatusFlag::Carry);
                }

                val >>= 1;

                self.bus.mem_write(addr, val);

                self.check_zero(val);
                self.clear_flag(StatusFlag::Negative);
            }
            0x56 /* <-- [ Zero Page, X ] --> */ => {
                // Shift right zero page value + X
                // 2 bytes, 6 cycles
                self.sleep_cycles = 5;

                if self.debug_mode { print!("LSR: Zero Page, X | "); }

                let addr = self.get_addr(AddressingMode::ZeroPageX);
                let mut val = self.bus.mem_read(addr);

                if val & 0x1 == 1 {
                    self.set_flag(StatusFlag::Carry);
                } else {
                    self.clear_flag(StatusFlag::Carry);
                }

                val >>= 1;

                self.bus.mem_write(addr, val);

                self.check_zero(val);
                self.clear_flag(StatusFlag::Negative);
            }
            0x4E /* <-- [ Absolute ] --> */ => {
                // Shift right absolute value
                // 3 bytes, 6 cycles
                self.sleep_cycles = 5;

                if self.debug_mode { print!("LSR: Absolute | "); }

                let addr = self.get_addr(AddressingMode::Absolute);
                let mut val = self.bus.mem_read(addr);

                if val & 0x1 == 1 {
                    self.set_flag(StatusFlag::Carry);
                } else {
                    self.clear_flag(StatusFlag::Carry);
                }

                val >>= 1;

                self.bus.mem_write(addr, val);

                self.check_zero(val);
                self.clear_flag(StatusFlag::Negative);
            }
            0x5E /* <-- [ Absolute, X ] --> */ => {
                // Shift right absolute value + X
                // 3 bytes, 7 cycles
                self.sleep_cycles = 6;

                if self.debug_mode { print!("LSR: Absolute, X | "); }

                let addr = self.get_addr(AddressingMode::AbsoluteX);
                let mut val = self.bus.mem_read(addr);

                if val & 0x1 == 1 {
                    self.set_flag(StatusFlag::Carry);
                } else {
                    self.clear_flag(StatusFlag::Carry);
                }

                val >>= 1;

                self.bus.mem_write(addr, val);

                self.check_zero(val);
                self.clear_flag(StatusFlag::Negative);
            }

            // <--| JMP |-->
            0x4C /* <-- [ Absolute ] --> */ => {
                // Jump to absolute address
                // 3 bytes, 3 cycles
                self.sleep_cycles = 2;

                // set PC to value
                let addr = self.get_addr(AddressingMode::Absolute);
                self.pc = self.bus.mem_read_u16(addr);

                // PC is incremented at end of cycle
                dont_increment_pc = true;
            }
            0x6C /* <-- [ Indirect ] --> */ => {
                // Jump to indirect address
                // 3 bytes, 5 cycles
                self.sleep_cycles = 4;

                // set PC to value
                let addr = self.get_addr(AddressingMode::Indirect);
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
                let addr = self.get_addr(AddressingMode::ZeroPage);
                self.bus.mem_write(addr, self.acc);

                // skip next byte
                self.pc += 1;
            }
            0x95 /* <-- [ Zero Page, X ] --> */ => {
                // Store accumulator at zero page address + X
                // 2 bytes, 4 cycles
                self.sleep_cycles = 3;

                // set memory to accumulator
                let addr = self.get_addr(AddressingMode::ZeroPageX);
                self.bus.mem_write(addr, self.acc);

                // skip next byte
                self.pc += 1;
            }
            0x8D /* <-- [ Absolute ] --> */ => {
                // Store accumulator at absolute address
                // 3 bytes, 4 cycles
                self.sleep_cycles = 3;

                // set memory to accumulator
                let addr = self.get_addr(AddressingMode::Absolute);
                self.bus.mem_write(addr, self.acc);

                // skip next 2 bytes
                self.pc += 2;
            }
            0x9D /* <-- [ Absolute, X ] --> */ => {
                // Store accumulator at absolute address + X
                // 3 bytes, 5 cycles
                self.sleep_cycles = 4;

                // set memory to accumulator
                let addr = self.get_addr(AddressingMode::AbsoluteX);
                self.bus.mem_write(addr, self.acc);

                // skip next 2 bytes
                self.pc += 2;
            }
            0x99 /* <-- [ Absolute, Y ] --> */ => {
                // Store accumulator at absolute address + Y
                // 3 bytes, 5 cycles
                self.sleep_cycles = 4;

                // set memory to accumulator
                let addr = self.get_addr(AddressingMode::AbsoluteY);
                self.bus.mem_write(addr, self.acc);

                // skip next 2 bytes
                self.pc += 2;
            }
            0x81 /* <-- [ Indirect, X ] --> */ => {
                // Store accumulator at indirect address + X
                // 2 bytes, 6 cycles
                self.sleep_cycles = 5;

                // set memory to accumulator
                let addr = self.get_addr(AddressingMode::IndirectX);
                self.bus.mem_write(addr, self.acc);

                // skip next byte
                self.pc += 1;
            }
            0x91 /* <-- [ Indirect, Y ] --> */ => {
                // Store accumulator at indirect address + Y
                // 2 bytes, 6 cycles
                self.sleep_cycles = 5;

                // set memory to accumulator
                let addr = self.get_addr(AddressingMode::IndirectY);
                self.bus.mem_write(addr, self.acc);

                // skip next byte
                self.pc += 1;
            }

            // <--| STX |-->
            0x86 /* <-- [ Zero Page ] --> */ => {
                // Store X at zero page address
                // 2 bytes, 3 cycles
                self.sleep_cycles = 2;

                // set memory to X
                let addr = self.get_addr(AddressingMode::ZeroPage);
                self.bus.mem_write(addr, self.idx_x);

                // skip next byte
                self.pc += 1;
            }
            0x96 /* <-- [ Zero Page, Y ] --> */ => {
                // Store X at zero page address + Y
                // 2 bytes, 4 cycles
                self.sleep_cycles = 3;

                // set memory to X
                let addr = self.get_addr(AddressingMode::ZeroPageY);
                self.bus.mem_write(addr, self.idx_x);

                // skip next byte
                self.pc += 1;
            }
            0x8E /* <-- [ Absolute ] --> */ => {
                // Store X at absolute address
                // 3 bytes, 4 cycles
                self.sleep_cycles = 3;

                // set memory to X
                let addr = self.get_addr(AddressingMode::Absolute);
                self.bus.mem_write(addr, self.idx_x);

                // skip next 2 bytes
                self.pc += 2;
            }

            // <--| STY |-->
            0x84 /* <-- [ Zero Page ] --> */ => {
                // Store Y at zero page address
                // 2 bytes, 3 cycles
                self.sleep_cycles = 2;

                // set memory to Y
                let addr = self.get_addr(AddressingMode::ZeroPage);
                self.bus.mem_write(addr, self.idx_y);

                // skip next byte
                self.pc += 1;
            }
            0x94 /* <-- [ Zero Page, X ] --> */ => {
                // Store Y at zero page address + X
                // 2 bytes, 4 cycles
                self.sleep_cycles = 3;

                // set memory to Y
                let addr = self.get_addr(AddressingMode::ZeroPageX);
                self.bus.mem_write(addr, self.idx_y);

                // skip next byte
                self.pc += 1;
            }
            0x8C /* <-- [ Absolute ] --> */ => {
                // Store Y at absolute address
                // 3 bytes, 4 cycles
                self.sleep_cycles = 3;

                // set memory to Y
                let addr = self.get_addr(AddressingMode::Absolute);
                self.bus.mem_write(addr, self.idx_y);

                // skip next 2 bytes
                self.pc += 2;
            }

            // <--| INC |-->
            0xE6 /* <-- [ Zero Page ] --> */ => {
                // Increment value at zero page address
                // 2 bytes, 5 cycles
                self.sleep_cycles = 4;

                // increment value
                let addr = self.get_addr(AddressingMode::ZeroPage);
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
                let addr = self.get_addr(AddressingMode::ZeroPageX);
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
                let addr = self.get_addr(AddressingMode::Absolute);
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
                let addr = self.get_addr(AddressingMode::AbsoluteX);
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

            // <--| TSX |-->
            0xBA /* <-- [ None ] --> */ => {
                // Transfer stack pointer to index X
                // 1 byte, 2 cycles
                self.sleep_cycles = 1;

                self.idx_x = self.sp;

                self.check_zero(self.idx_x);
                self.check_negative(self.idx_x);
            }

            // <--| TXA |-->
            0x8A /* <-- [ None ] --> */ => {
                // Transfer index X to accumulator
                // 1 byte, 2 cycles
                self.sleep_cycles = 1;

                self.acc = self.idx_x;

                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }

            // <--| TXS |-->
            0x9A /* <-- [ None ] --> */ => {
                // Transfer index X to stack pointer
                // 1 byte, 2 cycles
                self.sleep_cycles = 1;

                self.sp = self.idx_x;
            }

            // <--| TYA |-->
            0x98 /* <-- [ None ] --> */ => {
                // Transfer index Y to accumulator
                // 1 byte, 2 cycles
                self.sleep_cycles = 1;

                self.acc = self.idx_y;

                self.check_zero(self.acc);
                self.check_negative(self.acc);
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
                self.pc = self.get_addr(AddressingMode::Absolute);

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

            // <--| SEC |-->
            0x38 /* <-- [ None ] --> */ => {
                // Set carry flag
                // 1 byte, 2 cycles
                self.sleep_cycles = 1;

                self.set_flag(StatusFlag::Carry);
            }

            // <--| SED |-->
            0xF8 /* <-- [ None ] --> */ => {
                // Set decimal flag
                // 1 byte, 2 cycles
                self.sleep_cycles = 1;

                self.set_flag(StatusFlag::Decimal);
            }

            // <--| SEI |-->
            0x78 /* <-- [ None ] --> */ => {
                // Set interrupt disable flag
                // 1 byte, 2 cycles
                self.sleep_cycles = 1;

                self.set_flag(StatusFlag::Interrupt);
            }

            // <--| PHP |--> 
            0x08 /* <-- [ None ] --> */ => {
                // Push processor status to stack
                // 1 byte, 3 cycles
                self.sleep_cycles = 2;

                // push status, set bit 6 and 5 to 1
                self.push_stack(self.status | 0b00110000);
            }

            // <--| PHA |--> 
            0x48 /* <-- [ None ] --> */ => {
                // Push accumulator to stack
                // 1 byte, 3 cycles
                self.sleep_cycles = 2;

                self.push_stack(self.acc);
            }
            
            // <--| PLP |-->
            0x28 /* <-- [ None ] --> */ => {
                // Pull processor status from stack
                // 1 byte, 4 cycles
                self.sleep_cycles = 3;

                // pull status, ignore bit 5 and 4
                self.status = self.pop_stack() & 0b11001111;
            }
            
            // <--| PLA |-->
            0x68 /* <-- [ None ] --> */ => {
                // Pull accumulator from stack
                // 1 byte, 4 cycles
                self.sleep_cycles = 3;

                self.acc = self.pop_stack();

                self.check_zero(self.acc);
                self.check_negative(self.acc);
            }
            
            _ => { 
                if self.debug_mode { print!("Unknown instr | ") };
            }//unimplemented!("Opcode {:#X} not implemented", opcode)},
        }

        if self.debug_mode { println!() };

        // Increment PC
        if !dont_increment_pc {
            self.pc += 1;
        }
    }

    /// Returns the address of the operand for the current instruction, and increments the PC
    fn get_addr(&self, mode: AddressingMode) -> u16 {
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
            AddressingMode::ZeroPageY => {
                // zero page Y addressing mode, addr is next byte's oper + Y
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
                let oper = self.bus.mem_read(self.pc + 1);
                self.bus.mem_read_u16(oper as u16)
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
    pub const Decimal: u8 =      0b0000_1000;
    pub const Interrupt: u8 = 0b0000_0100;
    pub const Zero: u8 =             0b0000_0010;
    pub const Carry: u8 =            0b0000_0001;
}

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
}
