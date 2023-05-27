use egui_sdl2_gl::{egui, DpiScaling, ShaderVersion};
use sdl2::video::Window;

pub struct Gui {
    pub egui_ctx: egui::CtxRef,
    pub egui_state: egui_sdl2_gl::EguiStateHandler,
    pub painter: egui_sdl2_gl::painter::Painter,
    pub ui_closure: Box<dyn Fn(&egui::CtxRef)>,
    _start_time: std::time::Instant,
}

impl Gui {
    pub fn new(window: &sdl2::video::Window) -> Self {
        let (painter, egui_state) =
            egui_sdl2_gl::with_sdl2(&window, ShaderVersion::Default, DpiScaling::Default);
        let egui_ctx: egui::CtxRef = egui::CtxRef::default();

        let _start_time = std::time::Instant::now();

        let ui_closure = Box::new(|_ctx: &egui::CtxRef| {});

        Gui {
            egui_ctx,
            egui_state,
            painter,
            ui_closure,
            _start_time,
        }
    }

    pub fn render(&mut self, window: &Window) {
        self.egui_state.input.time = Some(self._start_time.elapsed().as_secs_f64());
        self.egui_ctx.begin_frame(self.egui_state.input.take());

        self.ui_closure.as_ref()(&self.egui_ctx);

        let (egui_output, paint_cmds) = self.egui_ctx.end_frame();
        self.egui_state.process_output(&window, &egui_output);

        let paint_jobs = self.egui_ctx.tessellate(paint_cmds);

        // Note: passing a bg_color to paint_jobs will clear any previously drawn stuff.
        self.painter
            .paint_jobs(None, paint_jobs, &self.egui_ctx.font_image());
    }

    pub fn set_ui(&mut self, ui_closure: Box<dyn Fn(&egui::CtxRef)>) {
        self.ui_closure = ui_closure;
    }

    pub fn process_event(&mut self, window: &Window, event: sdl2::event::Event) {
        self.egui_state
            .process_input(&window, event, &mut self.painter);
    }
}
