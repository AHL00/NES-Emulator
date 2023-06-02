use std::{cell::RefCell, rc::Rc};

use super::{ppu::PPU, memory::{RAM, ROM}};

pub struct Bus {
    ram: Rc<RefCell<RAM>>,
    rom: Rc<RefCell<ROM>>,
    ppu: Rc<RefCell<PPU>>,
}

impl Bus {
    pub fn new(ram: Rc<RefCell<RAM>>, ppu: Rc<RefCell<PPU>>, rom: Rc<RefCell<ROM>>) -> Self {
        Bus { ram, ppu, rom }
    }

    // Mapping stuff
    pub fn mem_read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram.borrow()[addr & 0x07FF], // RAM
            0x2000..=0x3FFF => 0, // IO Registers
            0x4000..=0x401F => 0,          // More IO Registers
            0x4020..=0x5FFF => 0,          // Expansion ROM
            0x6000..=0x7FFF => 0,          // SRAM
            0x8000..=0xFFFF => self.prg_rom_read(addr - 0x8000), // PRG-ROM
        }  
    }

    pub fn mem_read_signed(&self, addr: u16) -> i8 {
        self.mem_read(addr) as i8
    }

    pub fn mem_write(&self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram.borrow_mut()[addr & 0x07FF] = data, // RAM
            0x2000..=0x3FFF => panic!("Not implemented!"), // IO Registers
            0x4000..=0x401F => panic!("Not implemented!"),          // More IO Registers
            0x4020..=0x5FFF => panic!("Not implemented!"),          // Expansion ROM
            0x6000..=0x7FFF => panic!("Not implemented!"),          // SRAM
            0x8000..=0xFFFF => panic!("Attempted to write to program rom!"), // PRG-ROM
        }  
    }

    pub fn mem_write_signed(&self, addr: u16, data: i8) {
        self.mem_write(addr, data as u8);
    }

    /// Only works from 0x0000 to 0x1FFF
    pub fn mem_read_u16(&self, addr: u16) -> u16 {
        (self.mem_read(addr) as u16) | ((self.mem_read(addr + 1) as u16) << 8)
    }

    /// Only works from 0x0000 to 0x1FFF
    pub fn mem_write_u16(&self, addr: u16, data: u16) {
        self.mem_write(addr, (data & 0x00FF) as u8);
        self.mem_write(addr + 1, ((data & 0xFF00) >> 8) as u8);
    }

    fn prg_rom_read(&self, addr: u16) -> u8 {
        match self.rom.borrow().mapper {
            0 => { // NROM
                // only one bank
                self.rom.borrow().prg_rom[0].data[addr as usize]
            },
            _ => panic!("Program rom mapper not implemented"),
        }
    }
    // pub fn memory_mapper(&self, addr: u16) -> u16 {
    //     match addr {
    //         0x0000..=0x1FFF => addr & 0x07FF, // RAM
    //         0x2000..=0x3FFF => addr & 0x0007, // IO Registers
    //         0x4000..=0x401F => addr,          // More IO Registers
    //         0x4020..=0x5FFF => addr,          // Expansion ROM
    //         0x6000..=0x7FFF => addr,          // SRAM
    //         0x8000..=0xBFFF => addr,          // PRG-ROM Bank 1
    //         0xC000..=0xFFFF => addr,          // PRG-ROM Bank 2
    //     }
    // }
}

// pub fn mem_read(&self, addr: u16) -> u8 {
//     self.ram.borrow()[addr]
// }

// pub fn mem_write(&self, addr: u16, data: u8) {
//     self.ram.borrow_mut()[addr] = data;
// }

// pub fn mem_read_u16(&self, addr: u16) -> u16 {
//     u16::from_le_bytes([self.mem_read(addr), self.mem_read(addr + 1)])
// }

// pub fn mem_write_u16(&self, addr: u16, data: u16) {
//     let bytes = data.to_le_bytes();
//     self.ram.borrow_mut()[addr] = bytes[0];
//     self.ram.borrow_mut()[addr + 1] = bytes[1];
// }
