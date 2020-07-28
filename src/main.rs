extern crate sdl2;

use sdl2::{Sdl, VideoSubsystem, EventPump};
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::render::{WindowCanvas, RenderTarget, Canvas};
use std::error::Error;
use std::thread;
use std::time::Duration;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
struct SdlError {
    description: String,
}

impl SdlError {
    pub fn new(description: String) -> SdlError {
        SdlError { description }
    }
}

impl Display for SdlError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "SDL error: {}", self.description)
    }
}

impl Error for SdlError {}


struct SdlEnv {
    pub context: Sdl,
    pub video: VideoSubsystem,
}


fn init_sdl() -> Result<SdlEnv, SdlError> {
    let context = sdl2::init().map_err(SdlError::new)?;
    let video = context.video().map_err(SdlError::new)?;
    Ok(SdlEnv { context, video })
}


fn make_canvas(sdl: &SdlEnv, title: &str, width: u32, height: u32) -> Result<WindowCanvas, SdlError> {
    sdl
        .video
        .window(title, width, height)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| SdlError::new(e.to_string()))?
        .into_canvas()
        .build()
        .map_err(|e| SdlError::new(e.to_string()))
}


fn main_loop(canvas: &mut Canvas<impl RenderTarget>, event_pump: &mut EventPump) -> Result<(), SdlError> {
    loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => return Ok(()),
                _ => {}
            }
        }

        canvas.clear();
        canvas.present();
        thread::sleep(Duration::from_secs_f64(1f64 / 60f64));
    }
}


fn main() -> Result<(), Box<dyn Error>> {
    let sdl_env = init_sdl()?;
    let mut canvas = make_canvas(&sdl_env, "My window", 800, 600)?;
    canvas.set_draw_color(Color::RGB(255, 255, 0));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_env.context.event_pump().map_err(SdlError::new)?;
    main_loop(&mut canvas, &mut event_pump)?;

    Ok(())
}
