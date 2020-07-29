use crate::geometry::{Point, Triangle, Line, HorizontalSegment, GluedTriangle, Triangular};
use super::SdlError;

use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::video::WindowSurfaceRef;
use std::mem;

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
        for i in 1..10000 { 
            let base_x = i % 777;
            let base_y = (71 * i) % 555;
            let a = Point{x: base_x, y: base_y};
            let b = Point{x: base_x + 4, y: base_y + 8};
            let c = Point{x: base_x + 10, y: base_y + 4};
            context.fill_triangle(Triangle {a, b, c}, RGB::new((i % 255) as u8, 255, 0)); 
        }
        /*
        context.fill_glued_triangle(
            GluedTriangle {
                horizontal_segment: HorizontalSegment::from_points(
                    Point {x: 100, y: 100},
                    Point {x: 700, y: 100}
                ),
                free_point: Point {x: 600, y: 500},
            },
            RGB {r: 255, g: 0, b: 0}
        );
        */
    }
}


#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct RGB {
    pub b: u8,
    pub g: u8,
    pub r: u8,
}

impl RGB {
    pub fn new(r: u8, g: u8, b: u8) -> RGB {
        RGB { r, g, b }
    }
}


struct RenderContext<'a> {
    pub data: &'a mut [u8],
    pub width: u32,
    pub height: u32,
}

impl RenderContext<'_> {
    #[inline]
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

    pub fn fill_triangle(&mut self, tri: Triangle, value: RGB) {
        let (a, b, c) = tri.ysort();
        let line_hb = Line::horizontal(b.y);
        let line_ac = Line::from_points(a, c);
        let split_point = line_hb.intersect(line_ac);
        let horizontal_segment = HorizontalSegment::from_points(b, split_point);
        let glued_top = GluedTriangle::new(horizontal_segment, a);
        let glued_bottom = GluedTriangle::new(horizontal_segment, c);
        self.fill_glued_triangle(glued_top, value);
        self.fill_glued_triangle(glued_bottom, value);
    }

    fn fill_glued_triangle(&mut self, glued_tri: GluedTriangle, value: RGB) {
        let mut min = glued_tri.horizontal_segment.y();
        let mut max = glued_tri.free_point.y;
        if min > max {
            mem::swap(&mut min, &mut max);
        }

        let  left_line = Line::from_points(glued_tri.horizontal_segment.left(),  glued_tri.free_point);
        let right_line = Line::from_points(glued_tri.horizontal_segment.right(), glued_tri.free_point);

        for y in (min.max(0))..(max.min(self.height as i32)) {
            let horizontal_line = Line::horizontal(y);
            let  left_isect = horizontal_line.intersect(left_line);
            let right_isect = horizontal_line.intersect(right_line);

            for x in (left_isect.x.max(0))..(right_isect.x.min(self.width as i32)) {
                self.set(x as u32, y as u32, value);
            }
        }
    }
}
