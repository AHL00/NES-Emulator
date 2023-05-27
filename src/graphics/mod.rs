pub mod gui;

pub struct Graphics {
    pub sdl_context: sdl2::Sdl,
    pub video_subsystem: sdl2::VideoSubsystem,
    pub window: sdl2::video::Window,
    pub event_pump: sdl2::EventPump,
    pub gui: gui::Gui,
    _gl_ctx: sdl2::video::GLContext,
}

impl Graphics {
    pub fn new() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let gl_attr: sdl2::video::gl_attr::GLAttr = video_subsystem.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_double_buffer(true);
        gl_attr.set_multisample_samples(4);

        let window = video_subsystem
            .window(
                "Demo: Egui backend for SDL2 + GL",
                800,
                600,
            )
            .opengl()
            .resizable()
            .build()
            .unwrap();

        let _ctx = window.gl_create_context().unwrap();

        let event_pump: sdl2::EventPump = sdl_context.event_pump().unwrap();

        let gui = gui::Gui::new(&window);

        Graphics {
            sdl_context,
            video_subsystem,
            window,
            event_pump,
            gui,
            _gl_ctx: _ctx,
        }
    }

    pub fn render(&mut self) {
        unsafe {
            // Clear the screen to green
            gl::ClearColor(0.3, 0.6, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

    }
}