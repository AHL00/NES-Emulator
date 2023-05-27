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

    let iters_clone = iters_last_sec.clone();

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
        }

        println!(
            "Cycles per second: {}",
            cycles as f64 / start.elapsed().as_secs_f64()
        );
    })
}