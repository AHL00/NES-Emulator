extern crate sdl2;

pub mod graphics;

use std::sync::{Arc, Mutex};

use graphics::Graphics;
use sdl2::event::Event;

pub fn main() -> Result<(), String> {
    let mut gfx = Graphics::new();

    let iters = Arc::new(Mutex::new(0));

    let start = std::time::Instant::now();

    let iters_clone = iters.clone();

    gfx.gui.set_ui(Box::new(move |ctx: &egui_sdl2_gl::egui::CtxRef| {
        egui_sdl2_gl::egui::Window::new("Test window").show(ctx, |ui| {
            ui.label(format!("Main loop iters: {:?}", iters_clone.lock().unwrap()));
            ui.separator();
            ui.label(format!("Time: {:.2}s", start.elapsed().as_secs_f64()));
            ui.separator();
            ui.label(format!("FPS: {:.2}", *iters_clone.lock().unwrap() as f64 / start.elapsed().as_secs_f64()));
        });
    }));

    'running: loop {
        *iters.lock().unwrap() += 1;

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