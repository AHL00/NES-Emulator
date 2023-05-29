use std::ops::{Index, IndexMut, Range};

pub struct RAM {
    pub array: [u8; 0xFFFF],
}

impl IndexMut<u16> for RAM {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        match index {
            0x0000..=0x1FFF => {
                // CPU ram access
                &mut self.array[index as usize]
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
                &mut self.array[index as usize]
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
                &self.array[index as usize]
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
                &self.array[index as usize]
            },
            _ => panic!("Invalid memory address: {:#X}", index),
        }
    }
}

impl IndexMut<Range<u16>> for RAM {
    fn index_mut(&mut self, index: Range<u16>) -> &mut Self::Output {
        match index.start {
            0x0000..=0x1FFF => {
                // CPU ram access
                &mut self.array[index.start as usize..index.end as usize]
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
                &mut self.array[index.start as usize..index.end as usize]
            },
            _ => panic!("Invalid memory address: {:#X}", index.start),
        }
    }
}

impl Index<Range<u16>> for RAM {
    type Output = [u8];

    fn index(&self, index: Range<u16>) -> &Self::Output {
        match index.start {
            0x0000..=0x1FFF => {
                // CPU ram access
                &self.array[index.start as usize..index.end as usize]
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
                &self.array[index.start as usize..index.end as usize]
            },
            _ => panic!("Invalid memory address: {:#X}", index.start),
        }
    }
}

impl RAM {
    pub fn new() -> Self {
        RAM {
            array: [0; 0xFFFF],
        }
    }

    pub fn reset(&mut self) {
        self.array = [0; 0xFFFF];
    }
}