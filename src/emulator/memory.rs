use std::ops::{Index, IndexMut, Range};

pub struct RAM {
    data: [u8; 0x0800],
}

impl IndexMut<u16> for RAM {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        &mut self.data[index as usize]
    }
}

impl Index<u16> for RAM {
    type Output = u8;

    fn index(&self, index: u16) -> &Self::Output {
        &self.data[index as usize]
    }
}

impl IndexMut<Range<u16>> for RAM {
    fn index_mut(&mut self, index: Range<u16>) -> &mut Self::Output {
        &mut self.data[index.start as usize..index.end as usize]
    }
}

impl Index<Range<u16>> for RAM {
    type Output = [u8];

    fn index(&self, index: Range<u16>) -> &Self::Output {
        &self.data[index.start as usize..index.end as usize]
    }
}

impl RAM {
    pub fn new() -> Self {
        RAM { data: [0; 0x0800] }
    }

    pub fn reset(&mut self) {
        self.data = [0; 0x0800];
    }
}

pub struct ROM {
    pub prg_rom: Vec<ProgramRomBank>,
    pub chr_rom: Vec<CharRomBank>,
    pub mapper: u8,
}

impl ROM {
    pub fn new() -> Self {
        // Empty ROM
        ROM {
            prg_rom: Vec::new(),
            chr_rom: Vec::new(),
            mapper: 0,
        }
    }

    // Bank switching here?

    pub fn read_from_bytes(&mut self, bytes: &[u8]) {
        // read the mapper number from the ROM header
        self.mapper = bytes[6] >> 4;

        // Make sure rom object vecs are empty
        self.prg_rom.clear();
        self.chr_rom.clear();

        match self.mapper {
            0 => {
                // <-- NROM -->
                // Check if the ROM is valid
                if bytes[0..4] != [0x4E, 0x45, 0x53, 0x1A] {
                    panic!("Invalid ROM file!");
                }

                // Check if need to skip trainer
                let trainer = bytes[6] & 0x04 == 0x04;
                let trainer_len = if trainer { 512 } else { 0 };

                // Read the 1 prg-rom bank at 0x10
                let mut prg_rom_bank = ProgramRomBank::new();
                prg_rom_bank
                    .data
                    .copy_from_slice(&bytes[0x10 + trainer_len..0x4010 + trainer_len]);

                // Read the 1 chr-rom bank at 0x10 + 1 prg-rom bank size
                let mut chr_rom_bank = CharRomBank::new();
                chr_rom_bank
                    .data
                    .copy_from_slice(&bytes[0x4010 + trainer_len..0x6010 + trainer_len]);

                // Push the banks to the rom object
                self.prg_rom.push(prg_rom_bank);
                self.chr_rom.push(chr_rom_bank);
            }
            _ => panic!("Mapper {} not supported!", self.mapper),
        }
    }
}

pub struct ProgramRomBank {
    pub data: [u8; 0x4000],
}

impl ProgramRomBank {
    pub fn new() -> Self {
        ProgramRomBank { data: [0; 0x4000] }
    }
}

pub struct CharRomBank {
    pub data: [u8; 0x2000],
}

impl CharRomBank {
    pub fn new() -> Self {
        CharRomBank { data: [0; 0x2000] }
    }
}
