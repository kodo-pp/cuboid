extern crate sdl2;
extern crate gcd;

mod clock;
mod render;
mod geometry;

use crate::geometry::{Point, Point3d, BasicTriangle};
use crate::render::{RGB, Render, Renderer};
use crate::clock::{Clock, EventsPerSecondTracker, ApproximateTimer};

use sdl2::{Sdl, VideoSubsystem, EventPump};
use sdl2::event::Event;
use sdl2::video::Window;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::time::{Duration, Instant};


#[derive(Debug)]
pub struct SdlError {
    description: String,
}

impl SdlError {
    pub fn new(description: String) -> SdlError {
        SdlError { description }
    }
}

impl From<String> for SdlError {
    fn from(s: String) -> SdlError {
        SdlError::new(s)
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
    let context = sdl2::init()?;
    let video = context.video()?;
    Ok(SdlEnv { context, video })
}


fn make_window(sdl: &SdlEnv, title: &str, width: u32, height: u32) -> Result<Window, SdlError> {
    sdl
        .video
        .window(title, width, height)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| SdlError::new(e.to_string()))
}


fn main_loop(window: &Window, event_pump: &mut EventPump) -> Result<(), SdlError> {
    let mut clock = Clock::new();
    let mut fps_tracker = EventsPerSecondTracker::new();
    let mut approximate_timer = ApproximateTimer::new(Duration::from_secs(1));

    let spinning_triangle = SpinningTriangle::new();

    loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => return Ok(()),
                _ => {}
            }
        }

        render::render_frame(&spinning_triangle, window.surface(event_pump)?)?;
        fps_tracker.event();
        let tick_duration = clock.tick(60.0);
        if approximate_timer.update(tick_duration) != 0 {
            let fps = fps_tracker.mean();
            fps_tracker.reset();
            println!("FPS: {}", fps);
        }
    }
}


fn main() -> Result<(), Box<dyn Error>> {
    let sdl_env = init_sdl()?;
    let window = make_window(&sdl_env, "My fucking window", 800, 600)?;

    let mut event_pump = sdl_env.context.event_pump()?;
    main_loop(&window, &mut event_pump)?;

    Ok(())
}


struct SpinningTriangle {
    origin: Instant,
}

impl SpinningTriangle {
    pub fn new() -> SpinningTriangle {
        SpinningTriangle {origin: Instant::now()}
    }
}

impl Render for SpinningTriangle {
    fn render<'a>(&self, renderer: &mut Renderer<'a>) {
        let t = self.origin.elapsed().as_secs_f64();
        let a = Point3d {x:  100.0 * t.cos(), y:  30.0, z: 200.0 + 100.0 * t.sin()};
        let b = Point3d {x:  100.0 * t.cos(), y: -30.0, z: 200.0 + 100.0 * t.sin()};
        let c = Point3d {x: -100.0 * t.cos(), y:  30.0, z: 200.0 - 100.0 * t.sin()};
        let triangle = BasicTriangle::new(a, b, c);
        renderer.fill_triangle(triangle, &|Point {x, y}| RGB::new((x % 256) as u8, (y % 256) as u8, 255u8));
    }
}
