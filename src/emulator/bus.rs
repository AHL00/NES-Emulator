use std::{cell::RefCell, rc::Rc};

use super::{ram::RAM, ppu::PPU};

pub struct Bus {
    ram: Rc<RefCell<RAM>>,
    ppu: Rc<RefCell<PPU>>,
}

impl Bus {
    pub fn new(ram: Rc<RefCell<RAM>>, ppu: Rc<RefCell<PPU>>) -> Self {
        Bus {
            ram,
            ppu,
        }
    }

    // Mapping stuff
    pub fn mem_read(&self, addr: u16) -> u8 {
        self.ram.borrow_mut()[addr]
    }

    pub fn mem_write(&self, addr: u16, data: u8) {
        self.ram.borrow_mut()[addr] = data;
    }

    pub fn mem_read_u16(&self, addr: u16) -> u16 {
        u16::from_le_bytes([self.mem_read(addr), self.mem_read(addr + 1)])
    }

    pub fn mem_write_u16(&self, addr: u16, data: u16) {
        let bytes = data.to_le_bytes();
        self.ram.borrow_mut()[addr] = bytes[0];
        self.ram.borrow_mut()[addr + 1] = bytes[1];
    }
}