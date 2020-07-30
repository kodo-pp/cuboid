use crate::geometry::{
    Angle,
    AngleWith,
    Azimuth,
    BasicPoint,
    BasicTriangle,
    BasicVector,
    GluedTriangle,
    HorizontalSegment,
    Line,
    Norm,
    Point,
    Point3d,
    Triangle,
    Triangular,
};
use super::SdlError;

use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::video::WindowSurfaceRef;
use std::mem;
use std::fmt::Debug;


pub fn render_frame<'a>(
    renderable: &impl Render,
    mut surface_ref: WindowSurfaceRef<'a>
) -> Result<(), SdlError> {
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
        let rasterizer = Rasterizer::new(data, width, height);
        let mut renderer = Renderer::new(rasterizer, width, height);
        renderable.render(&mut renderer);
    });
    surface_ref.finish()?;
    Ok(())
}


#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct RGB {
    pub b: u8,
    pub g: u8,
    pub r: u8,
}

impl RGB {
    pub fn new(r: u8, g: u8, b: u8) -> RGB {
        RGB { r, g, b }
    }
}


pub struct Camera {
    pub position: Point3d,
    pub azimuth: Angle,
    pub vertical_angle: Angle,
    pub hfov: Angle,
    pub vfov: Angle,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            position: Point3d{x: 0.0, y: 0.0, z: 0.0},
            azimuth: Angle::quarter_circle(),
            vertical_angle: Angle::zero(),
            hfov: Angle::from_degrees(100.0),
            vfov: Angle::from_degrees(70.0),
        }
    }

    pub fn translate(&self, point: Point3d) -> (BasicPoint<f64>, f64) {
        // Adjust the cartesian coordinates of the point
        let point = point - self.position.as_vector();

        // Calculate angles
        let vector = point.as_vector();
        let horizontal_vector = {
            let mut result = vector;
            result.y = 0.0;
            result
        };
        let vertical_angle_abs = vector.angle_with(&horizontal_vector);
        let vertical_angle = vertical_angle_abs * vector.y.signum();
        let azimuth = {
            let plane_vector = BasicVector::<f64> {x: horizontal_vector.x, y: horizontal_vector.z};
            plane_vector.azimuth()
        };

        // Adjust angles
        let vertical_angle = vertical_angle - self.vertical_angle;
        let azimuth = (azimuth - self.azimuth).into_plus_minus_pi_interval();
        // Transform angles to 2D coordinates
        let coord_x = azimuth / self.hfov + 0.5;
        let coord_y = vertical_angle / self.vfov + 0.5;
        (BasicPoint{x: coord_x, y: coord_y}, vector.norm())
    }
}


#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
}

impl Viewport {
    pub fn new(width: u32, height: u32) -> Viewport {
        Viewport {width, height}
    }

    pub fn translate(self, viewport_agnostic_point: BasicPoint<f64>) -> Point {
        Point {
            x: (viewport_agnostic_point.x * (self.width  as f64 - 1.0)).round() as i32,
            y: (viewport_agnostic_point.y * (self.height as f64 - 1.0)).round() as i32,
        }
    }
}


pub struct Renderer<'a> {
    rasterizer: Rasterizer<'a>,
    //depth_buffer: Vec<f32>,
    camera: Camera,
    viewport: Viewport,
}

impl Renderer<'_> {
    pub fn new<'a>(rasterizer: Rasterizer<'a>, width: u32, height: u32) -> Renderer<'a> {
        //let buffer_size = width as usize * height as usize;
        //let depth_buffer = Vec::<f32>::with_capacity(buffer_size);
        //depth_buffer.resize(buffer_size, );
        Renderer {
            rasterizer,
            //depth_buffer,
            camera: Camera::new(),
            viewport: Viewport::new(width, height),
        }
    }
    
    pub fn fill_triangle(&mut self, tri: BasicTriangle<Point3d>, color_func: &impl Fn(Point) -> RGB) {
        let (a, _da) = self.translate(tri.a);
        let (b, _db) = self.translate(tri.b);
        let (c, _dc) = self.translate(tri.c);
        if let Some(triangle_on_screen) = Triangle::try_new(a, b, c) {
            self.rasterizer.fill_triangle(triangle_on_screen, color_func);
        }
    }

    fn translate(&self, point: Point3d) -> (Point, f64) { 
        println!("Translate {:?}", point);
        let (viewport_agnostic_point, distance) = self.camera.translate(point);
        println!("  viewport_agnostic_point = {:?}", viewport_agnostic_point);
        let result = (self.viewport.translate(viewport_agnostic_point), distance);
        println!("  result = {:?}", result);
        result
    }
}


pub struct Rasterizer<'a> {
    data: &'a mut [u8],
    width: u32,
    height: u32,
}

impl Rasterizer<'_> {
    pub fn new<'a>(data: &'a mut [u8], width: u32, height: u32) -> Rasterizer<'a> {
        Rasterizer {data, width, height}
    }

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

    pub fn fill_triangle(&mut self, tri: Triangle, color_func: &impl Fn(Point) -> RGB) {
        let (a, b, c) = tri.ysort();
        let line_hb = Line::horizontal(b.y);
        let line_ac = Line::from_points(a, c);
        let split_point = line_hb.intersect(line_ac);
        let horizontal_segment = HorizontalSegment::from_points(b, split_point);

        if let Some(glued_top) = GluedTriangle::try_new(horizontal_segment, a) {
            self.fill_glued_triangle(glued_top, color_func);
        }
        if let Some(glued_bottom) = GluedTriangle::try_new(horizontal_segment, c) {
            self.fill_glued_triangle(glued_bottom, color_func);
        }
    }

    pub fn fill_glued_triangle(&mut self, glued_tri: GluedTriangle, color_func: &impl Fn(Point) -> RGB) {
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
                self.set(x as u32, y as u32, color_func(Point {x, y}));
            }
        }
    }
}


pub trait Render {
    fn render<'a>(&self, renderer: &mut Renderer<'a>);
}


pub trait Rasterize {
    fn rasterize<'a>(&self, rasterizer: &mut Rasterizer<'a>);
}
