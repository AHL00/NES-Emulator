use std::ops::{Index, IndexMut};

pub struct RAM {
    pub memory: [u8; 0xFFFF],
}

impl IndexMut<u16> for RAM {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        match index {
            0x0000..=0x1FFF => {
                // CPU ram access
                &mut self.memory[index as usize % 0x0800]
            },
            0x2000..=0x401F => {
                // IO registers
                unimplemented!("IO registers not implemented")
            },
            0x4021..=0x5FFF => {
                // Expansion ROM
                unimplemented!("Expansion ROM not implemented")
            },
            0x6000..=0x7FFF => {
                // Save RAM
                unimplemented!("SRAM not implemented")
            },
            0x8000..=0xFFFF => {
                // PRG-ROM
                unimplemented!("PRG-ROM not implemented")
            },
            _ => panic!("Invalid memory address: {:#X}", index),
        }
    }
}

impl Index<u16> for RAM {
    type Output = u8;

    fn index(&self, index: u16) -> &Self::Output {
        match index {
            0x0000..=0x1FFF => {
                // CPU ram access
                &self.memory[index as usize % 0x0800]
            },
            0x2000..=0x401F => {
                // IO registers
                unimplemented!("IO registers not implemented")
            },
            0x4021..=0x5FFF => {
                // Expansion ROM
                unimplemented!("Expansion ROM not implemented")
            },
            0x6000..=0x7FFF => {
                // Save RAM
                unimplemented!("SRAM not implemented")
            },
            0x8000..=0xFFFF => {
                // PRG-ROM
                unimplemented!("PRG-ROM not implemented")
            },
            _ => panic!("Invalid memory address: {:#X}", index),
        }
    }
}

impl RAM {
    pub fn new() -> Self {
        RAM {
            memory: [0; 0xFFFF],
        }
    }
}