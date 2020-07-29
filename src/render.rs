use crate::geometry::{Point, Triangle};

use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::video::WindowSurfaceRef;
use super::SdlError;

pub struct Renderer {}

impl Renderer {
    pub fn new() -> Renderer {
        Renderer{}
    }

    pub fn render<'a>(&mut self, mut surface_ref: WindowSurfaceRef<'a>) -> Result<(), SdlError> {
        surface_ref.fill_rect(None, Color::BLACK)?;
        let width = surface_ref.width();
        let height = surface_ref.height();

        let pixel_format_enum = surface_ref.pixel_format_enum();
        match pixel_format_enum {
            PixelFormatEnum::RGB888 | PixelFormatEnum::RGBA8888 | PixelFormatEnum::RGBX8888 => {},
            _ => panic!("Unsupported pixel format: {:?}", pixel_format_enum),
        }

        let bpp = pixel_format_enum.byte_size_per_pixel();
        assert_eq!(bpp, 4, "Non 4-byte pixels are not supported");
        
        surface_ref.with_lock_mut(|data| {
            Renderer::render_pixels(&mut RenderContext { data, width, height })
        });
        surface_ref.finish()?;
        Ok(())
    }

    fn render_pixels<'a>(context: &mut RenderContext<'a>) {
        let a = Point {x: 50, y: 100};
        let b = Point {x: 200, y: 500};
        let c = Point {x: 800, y: 400};
        let tri = Triangle {a, b, c};
        context.fill_triangle(&tri, RGB {r: 255, g: 0, b: 0});
    }
}


#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct RGB {
    r: u8,
    g: u8,
    b: u8,
}


struct RenderContext<'a> {
    pub data: &'a mut [u8],
    pub width: u32,
    pub height: u32,
}

impl RenderContext<'_> {
    pub fn set(&mut self, x: u32, y: u32, value: RGB) {
        self.data[self.index_at(x, y, 0)] = value.b;
        self.data[self.index_at(x, y, 1)] = value.g;
        self.data[self.index_at(x, y, 2)] = value.r;
    }

    #[inline]
    pub fn index_at(&self, x: u32, y: u32, component: u32) -> usize {
        // TODO: maybe introduce bound checks?
        ((self.width * y + x) * 4 + component) as usize
    }

    pub fn fill_triangle(&mut self, tri: &Triangle, value: RGB) {
        let rect = tri.bounding_rect();
        for y in rect.top()..rect.bottom() {
            for x in rect.left()..rect.right() {
                if tri.contains(Point{x, y}) {
                    self.set(x as u32, y as u32, value);
                }
            }
        }
    }
}
