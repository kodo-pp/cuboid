use crate::geometry::{
    Angle,
    Azimuth,
    BasicPoint,
    BasicTriangle,
    BasicVector,
    GluedTriangle,
    HorizontalSegment,
    Line,
    Norm,
    Par3d,
    Point,
    Point3d,
    Triangle,
    Triangle3d,
    Triangular,
};
use crate::linalg::{Matrix2d, Basis};
use crate::with::With;
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
    position: Point3d,
    azimuth: Angle,
    vertical_angle: Angle,
    hfov: Angle,
    vfov: Angle,
    hfov_half_cot: f64,
}

impl Camera {
    pub fn new() -> Camera {
        let hfov = Angle::from_degrees(100.0);
        Camera {
            position: Point3d{x: 0.0, y: 0.0, z: 0.0},
            azimuth: Angle::quarter_circle(),
            vertical_angle: Angle::zero(),
            hfov,
            vfov: Angle::from_degrees(70.0),
            hfov_half_cot: (hfov / 2.0).as_radians().tan().recip(),
        }
    }

    pub fn translate(&self, point: Point3d) -> (BasicPoint<f64>, f64) {
        // Adjust the cartesian coordinates of the point
        let point = point - self.position.as_vector();

        // Calculate azimuth
        let vector = point.as_vector();
        let horizontal_vector = {
            let mut result = vector;
            result.y = 0.0;
            result
        };
        let azimuth = {
            let plane_vector = BasicVector::<f64> {x: horizontal_vector.x, y: horizontal_vector.z};
            plane_vector.azimuth()
        };

        // Adjust azimuth
        let azimuth = (azimuth - self.azimuth).into_plus_minus_pi_interval();

        // Calculate vertical angle
        let vertical_angle = {
            let corrected_horizontal_vector_norm = horizontal_vector.norm() * azimuth.as_radians().cos();
            let vertical_distance = vector.y;
            Angle::from_radians(vertical_distance.atan2(corrected_horizontal_vector_norm))
        };

        // Adjust angles
        let vertical_angle = vertical_angle - self.vertical_angle;
        // Transform angles to 2D coordinates
        let coord_x = azimuth.as_radians().tan() * self.hfov_half_cot * 0.5 + 0.5;
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


struct DepthBuffer {
    depth_buffer: Vec<f32>,
    width: u32,
    height: u32,
}

impl DepthBuffer {
    pub fn new(width: u32, height: u32) -> DepthBuffer {
        let buffer_size = width as usize * height as usize;
        let mut depth_buffer = Vec::<f32>::with_capacity(buffer_size);
        depth_buffer.resize(buffer_size, f32::INFINITY);
        DepthBuffer {depth_buffer, width, height}
    }

    pub fn try_update(&mut self, x: u32, y: u32, value: f32) -> bool {
        if let Some(index) = self.index_at_checked(x, y) {
            if value < self.depth_buffer[index] {
                self.depth_buffer[index] = value;
                return true;
            }
        }
        false
    }

    fn index_at(&self, x: u32, y: u32) -> usize {
        let x = x as usize;
        let y = y as usize;
        y * self.width as usize + x
    }

    fn index_at_checked(&self, x: u32, y: u32) -> Option<usize> {
        if x < self.width && y < self.height {
            Some(self.index_at(x, y))
        } else {
            None
        }
    }
}


pub struct Renderer<'a> {
    rasterizer: Rasterizer<'a>,
    depth_buffer: DepthBuffer,
    camera: Camera,
    viewport: Viewport,
}

impl Renderer<'_> {
    pub fn new<'a>(rasterizer: Rasterizer<'a>, width: u32, height: u32) -> Renderer<'a> {
        Renderer {
            rasterizer,
            depth_buffer: DepthBuffer::new(width, height),
            camera: Camera::new(),
            viewport: Viewport::new(width, height),
        }
    }
    
    pub fn fill_triangle<
        Fill: ParFill + TranslateCoords,
        Constructor: With<Triangle, Output = Fill>
    >(&mut self, tri: Triangle3d, filler_constructor: Constructor) {
        let maybe_tri_and_depths = self.translate_tri(tri);
        if let Some((triangle_on_screen, depths)) = maybe_tri_and_depths {
            let filler = filler_constructor.with(triangle_on_screen);
            let mut adapter = ParFillDepthBufferAdapter::new(depths, filler, &mut self.depth_buffer);
            //let mut adapter = self.make_filler(filler, depths);
            self.rasterizer.fill_triangle(triangle_on_screen, &mut adapter);
        }
    }

    fn translate_tri(&self, tri: Triangle3d) -> Option<(Triangle, (f64, f64, f64))> {
        let (a, da) = self.translate_point(tri.a);
        let (b, db) = self.translate_point(tri.b);
        let (c, dc) = self.translate_point(tri.c);
        Triangle::try_new(a, b, c).and_then(|tri| Some((tri, (da, db, dc))))
    }

    pub fn fill_parallelogram<
        Fill: ParFill + TranslateCoords,
        Constructor: With<Triangle, Output = Fill>,
    >(&mut self, par: Par3d, filler_constructor: Constructor) {
        let (tri1, tri2) = par.to_triangles();
        self
            .translate_tri(tri1)
            .and_then(|(tri1_on_screen, depths)| {
                self.translate_tri(tri2).map(|(tri2_on_screen, _)| {
                    (tri1_on_screen, tri2_on_screen, depths)
                })
            })
            .map(|(tri1_on_screen, tri2_on_screen, depths)| {
            let mut filler = ParFillDepthBufferAdapter::new(
                depths,
                filler_constructor.with(tri1_on_screen),
                &mut self.depth_buffer
            );
            self.rasterizer.fill_triangle(tri1_on_screen, &mut filler);
            self.rasterizer.fill_triangle(tri2_on_screen, &mut filler);
        });
    }

    fn translate_point(&self, point: Point3d) -> (Point, f64) { 
        let (viewport_agnostic_point, distance) = self.camera.translate(point);
        (self.viewport.translate(viewport_agnostic_point), distance)
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

    pub fn fill_triangle(&mut self, tri: Triangle, filler: &mut impl ParFill) {
        let (a, b, c) = tri.ysort();
        let line_hb = Line::horizontal(b.y);
        let line_ac = Line::from_points(a, c);
        let split_point = line_hb.intersect(line_ac);
        let horizontal_segment = HorizontalSegment::from_points(b, split_point);

        if let Some(glued_top) = GluedTriangle::try_new(horizontal_segment, a) {
            self.fill_glued_triangle(glued_top, filler);
        }
        if let Some(glued_bottom) = GluedTriangle::try_new(horizontal_segment, c) {
            self.fill_glued_triangle(glued_bottom, filler);
        }
    }

    pub fn fill_glued_triangle(&mut self, glued_tri: GluedTriangle, filler: &mut impl ParFill) {
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
                let point = Point {x, y};
                if filler.should_draw(point) {
                    self.set(x as u32, y as u32, filler.color(point));
                }
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


pub trait ParFill {
    fn color(&self, point: Point) -> RGB;
    fn should_draw(&mut self, _point: Point) -> bool {
        true
    }
}


pub trait TranslateCoords {
    fn translate_coords(&self, point: Point) -> BasicPoint<f64>;
}


pub struct CoordsTranslator {
    origin: Point,
    basis: Basis<f64>,
}

impl CoordsTranslator {
    pub fn new(tri: Triangle) -> Self {
        CoordsTranslator {
            origin: tri.a,
            basis: CoordsTranslator::triangle_to_basis(tri),
        }
    }

    fn triangle_to_basis(tri: Triangle) -> Basis<f64> {
        let u = (tri.b - tri.a).map(&|x| x as f64);
        let v = (tri.c - tri.a).map(&|x| x as f64);
        let matrix = Matrix2d::from_columns(u.into(), v.into());
        Basis::new(matrix)
    }
}

impl TranslateCoords for CoordsTranslator {
    fn translate_coords(&self, point: Point) -> BasicPoint<f64> {
        BasicPoint::from(self.basis.coords_of((point - self.origin).map(&|x| x as f64)))
    }
}


struct ParFillDepthBufferAdapter<'a, Filler> {
    tri_depths: (f64, f64, f64),
    filler: Filler,
    depth_buffer: &'a mut DepthBuffer,
}

impl<'a, Filler> ParFillDepthBufferAdapter<'a, Filler> {
    pub fn new(
        tri_depths: (f64, f64, f64),
        filler: Filler,
        depth_buffer: &'a mut DepthBuffer
    ) -> Self {
        ParFillDepthBufferAdapter {tri_depths, filler, depth_buffer}
    }
}

impl<Filler: TranslateCoords> ParFillDepthBufferAdapter<'_, Filler> {
    fn get_depth(&self, point: Point) -> f64 {
        let tri_point = self.filler.translate_coords(point);
        let base_depth = self.tri_depths.0;
        let delta_depth_b = self.tri_depths.1 - base_depth;
        let delta_depth_c = self.tri_depths.2 - base_depth;
        base_depth + tri_point.x * delta_depth_b + tri_point.y * delta_depth_c
    }
}

impl<Filler: ParFill + TranslateCoords> ParFill for ParFillDepthBufferAdapter<'_, Filler> {
    fn color(&self, point: Point) -> RGB {
        self.filler.color(point)
    }

    fn should_draw(&mut self, point: Point) -> bool {
        if !self.filler.should_draw(point) {
            return false;
        }
        self.depth_buffer.try_update(point.x as u32, point.y as u32, self.get_depth(point) as f32)
    }
}
