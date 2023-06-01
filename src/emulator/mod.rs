use std::{cell::RefCell, rc::Rc};

use self::{cpu::CPU, memory::{RAM, ROM}, bus::Bus, ppu::PPU};

pub mod cpu;
pub mod ppu;
pub mod memory;
pub mod bus;
mod tests;

pub struct Emulator {
    pub memory: Rc<RefCell<RAM>>,
    pub cpu: CPU,
    pub bus: Rc<Bus>,
    pub ppu: Rc<RefCell<PPU>>,
    pub rom: Rc<RefCell<ROM>>,
}

impl Emulator {
    pub fn new() -> Self {
        let memory = Rc::new(RefCell::new(RAM::new()));
        let ppu = Rc::new(RefCell::new(PPU::new()));
        let rom = Rc::new(RefCell::new(ROM::new()));
        let bus = Rc::new(Bus::new(memory.clone(), ppu.clone(), rom.clone()));

        Emulator {
            memory: memory.clone(),
            cpu: CPU::new(bus.clone()),
            bus: bus.clone(),
            ppu: ppu.clone(),
            rom: rom.clone(),
        }
    }

    pub fn toggle_debug(&mut self) {
        self.cpu.debug_mode = !self.cpu.debug_mode;
    }

    pub fn load_rom(&mut self, rom_bytes: Vec<u8>) {
        // let rom_bytes =  // read rom bytes from file
        //     std::fs::read(rom_path)
        //     .expect(&format!("Failed to read file: {}", rom_path));

        self.rom.borrow_mut().read_from_bytes(&rom_bytes);

        self.cpu.pc = 0x8000;
    }
}
