extern crate sdl2;

pub mod emulator;
pub mod graphics;

use std::{
    cell::Cell,
    rc::Rc,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use graphics::Graphics;
use sdl2::event::Event;

pub fn main() -> Result<(), String> {
    let mut gfx = Graphics::new();

    let iters_last_sec = Arc::new(Mutex::new(0));
    let mut last_second = Instant::now();

    let start = Instant::now();

    let fps = Rc::new(Cell::new(0));

    let fps_clone = fps.clone();

    gfx.gui
        .set_ui(Box::new(move |ctx: &egui_sdl2_gl::egui::CtxRef| {
            egui_sdl2_gl::egui::Window::new("Test window").show(ctx, |ui| {
                ui.label(format!("Time: {:.2}s", start.elapsed().as_secs_f64()));
                ui.separator();
                ui.label(format!("FPS: {:.2}", fps_clone.get()));
            });
        }));

    start_emulator();

    'running: loop {
        if Instant::now().duration_since(last_second) >= Duration::from_secs(1) {
            last_second = Instant::now();
            fps.set(*iters_last_sec.lock().unwrap());
            *iters_last_sec.lock().unwrap() = 0;
        }

        *iters_last_sec.lock().unwrap() += 1;

        gfx.render();

        gfx.gui.render(&gfx.window);

        gfx.window.gl_swap_window();

        for event in gfx.event_pump.poll_iter() {
            gfx.gui.process_event(&gfx.window, event.clone());
            match event {
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }
    }

    Ok(())
}

fn start_emulator() -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut emulator = emulator::Emulator::new();

        // let rom_path = "roms/mario.nes";

        // let rom_bytes =  // read rom bytes from file
        //     std::fs::read(rom_path)
        //     .expect(&format!("Failed to read file: {}", rom_path));

        // emulator.load_rom(rom_bytes);

        emulator.bus.mem_write_u16(0x0000, 0x8000);
        let test_program = vec![
            0xA9, 0x12, // 0x8000 LDA immediate, load value 0x12 into accumulator
            0x4C, 0x00, 0x00, // 0x8002 JMP, jump to the address at 0x0000
            0xA9, 0x34, // 0x8005 LDA immediate, load value 0x34 into accumulator (skipped)
        ];

        let mut program: Vec<u8> = vec![
            0x4E, 0x45, 0x53, 0x1A, 0x02, 0x01, 0x01, 0x00, 0x02, 0x01, 0x01, 0x00, 0x02, 0x01,
            0x01, 0x00, // NROM header
        ];

        let mut prg_arr = [0; 0x4000];
        let chr_arr = [0; 0x2000];

        // copy the program to the start of the prg rom bank
        prg_arr[0..test_program.len()].copy_from_slice(&test_program);

        // add the two arrays to the program array
        program.extend_from_slice(&prg_arr);
        program.extend_from_slice(&chr_arr);

        emulator.load_rom(program);

        // test loop program, acc should be 0x12 at the end
        // emulator.bus.mem_write_u16(0x0000, 0x8000);
        // emulator.load();

        let target_cycle_time = Duration::from_secs_f64(1.0 / 1_789_773.0);
        //let target_cycle_time = Duration::from_secs_f64(1.0 / 3.0);

        println!("Target cycle time: {:?}", target_cycle_time);
        println!("Target hz: {:0}", 1.0 / target_cycle_time.as_secs_f64());

        println!("Running...");

        let mut cycles: i64 = 0;
        let start = Instant::now();
        
        let mut cycles_buffer = 0.0;
        
        loop {
            if start.elapsed().as_secs_f32() > 5.0 {
                break;
            }
            
            emulator.cpu.cycle();
            cycles += 1;
        }
        println!("Max speed: {:0} hz", cycles as f64 / start.elapsed().as_secs_f64());
        
        let start = Instant::now();
        let mut cycles: i64 = 0;
        let run_duration = Duration::from_secs(10);
        let mut last_loop = Instant::now();
        
        loop {
            if last_loop.elapsed() < Duration::from_secs_f32(1.0 / 10000.0) {
                continue;
            }

            let run_cycles = last_loop.elapsed().as_secs_f64() / target_cycle_time.as_secs_f64();

            last_loop = Instant::now();

            if start.elapsed() >= run_duration {
                break;
            }

            // if there are any cycles left over as decimal, add them to a buffer
            cycles_buffer += run_cycles % 1.0 as f64;

            // if the buffer gets over 1.0, add the int part to the cycles
            let mut added_cycles = 0;
            if cycles_buffer >= 1.0 {
                added_cycles = cycles_buffer.trunc() as i32; // truncate to int
                cycles_buffer -= added_cycles as f64;
            }

            // run the cycles
            for _ in 0..run_cycles as i32 + added_cycles {
                cycles += 1;
                emulator.cpu.cycle();
            }
        }

        println!(
            "Cycles per second: {:2}",
            cycles as f64 / start.elapsed().as_secs_f64()
        );

        println!(
            "Acc at end of loop should be 0x12, it is: 0x{:X}",
            emulator.cpu.acc
        );
    })
}
