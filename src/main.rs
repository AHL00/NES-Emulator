extern crate sdl2;

pub mod emulator;
pub mod graphics;

use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant}, rc::Rc, cell::Cell,
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
                ui.label(format!(
                    "FPS: {:.2}",
                    fps_clone.get()
                ));
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

        // test loop program, acc should be 0x12 at the end
        emulator.memory.borrow_mut().write_u16(0x0000, 0x8000);
        emulator.load(vec![
            0xA9, 0x12, // 0x8000 LDA immediate, load value 0x12 into accumulator
            0x4C, 0x00, 0x00, // 0x8002 JMP, jump to the address at 0x0000
            0xA9, 0x34, // 0x8005 LDA immediate, load value 0x34 into accumulator (skipped)
        ]);

        let target_cycle_time = Duration::from_secs_f64(1.0 / 1_789_773.0);

        println!("Target cycle time: {:?}", target_cycle_time);

        let start = Instant::now();
        let mut cycles: i64 = 0;

        loop {
            cycles += 1;

            if start.elapsed().as_secs() >= 1 {
                break;
            }

            emulator.cpu.cycle();

            // debug sleep for 1 second
            //std::thread::sleep(Duration::from_secs(1));
        }

        println!(
            "Cycles per second: {}",
            cycles as f64 / start.elapsed().as_secs_f64()
        );

        println!("Acc at end of loop should be 0x12, it is: 0x{:X}", emulator.cpu.acc);
    })
}