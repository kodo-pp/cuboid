use crate::geometry::*;
use crate::linalg::Matrix2d;
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


pub enum SegmentTranslatability {
    Translatable,
    SemiTranslatable(Point3d),
    NonTranslatable,
}


pub struct Camera {
    position: Point3d,
    azimuth: Angle,
    vertical_angle: Angle,
    hfov_half_cot_half: f64,
    vfov_half_cot_half: f64,
}

impl Camera {
    pub fn new() -> Camera {
        let hfov = Angle::from_degrees(100.0);
        let vfov = Angle::from_degrees(70.0);
        let hfov_half_cot_half = (hfov * 0.5).as_radians().tan().recip() * 0.5;
        let vfov_half_cot_half = (vfov * 0.5).as_radians().tan().recip() * 0.5;

        Camera {
            position: Point3d{x: 0.0, y: 0.0, z: 0.0},
            azimuth: Angle::quarter_circle(),
            vertical_angle: Angle::zero(),
            hfov_half_cot_half,
            vfov_half_cot_half,
        }
    }

    pub fn adjust(&self, point: Point3d) -> Point3d {
        let point = point - self.position.as_vector();
        point.as_vector().rotate_3d(-self.azimuth, -self.vertical_angle).as_point()
    }

    pub fn can_translate_point(&self, adjusted_point: Point3d) -> bool {
        // More conservative check as compared to the one in `translate`
        adjusted_point.x > 1e-3
    }

    pub fn can_translate_segment(&self, adjusted_segment: Segment<Point3d>) -> SegmentTranslatability {
        let a = adjusted_segment.a();
        let b = adjusted_segment.b();
        let can_a = self.can_translate_point(a);
        let can_b = self.can_translate_point(b);

        use SegmentTranslatability::*;
        match (can_a, can_b) {
            (true, true) => Translatable,
            (false, false) => NonTranslatable,
            _ => {
                // Compute the point of intersection of the segment and `x = 0` plane
                let x = 0.0;
                let y = (a.x * b.y - a.y * b.x) / (a.x - b.x);
                let z = (a.x * b.z - a.z * b.x) / (a.x - b.x);
                SemiTranslatable(Point3d {x, y, z})
            }
        }
    }

    pub fn translate(&self, adjusted_point: Point3d) -> (BasicPoint<f64>, f64) {
        // Perform sanity check
        if adjusted_point.x <= 0.0 {
            // More liberal check as compared to the one in `can_translate_point`
            panic!("A point behind the camera cannot be projected onto the screen");
        }

        // Calculate azimuth
        let vector = adjusted_point.as_vector();
        let azimuth = vector.azimuth();
        let vangle = vector.vangle();

        // Convert angles to 2D coordinates
        let x = azimuth.as_radians().tan() * self.hfov_half_cot_half + 0.5;
        let y =  vangle.as_radians().tan() * self.vfov_half_cot_half + 0.5;

        (BasicPoint{x, y}, vector.norm())
    }

    pub fn onto_screen_plane(&self, viewport_agnostic_point: BasicPoint<f64>) -> Point3d {
        let x = self.hfov_half_cot_half;
        let y = 0.5 - viewport_agnostic_point.y;
        let z = viewport_agnostic_point.x - 0.5;
        Point3d {x, y, z}
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

    pub fn untranslate(self, point_on_screen: Point) -> BasicPoint<f64> {
        BasicPoint {
            x: point_on_screen.x as f64 / (self.width  as f64 - 1.0),
            y: point_on_screen.y as f64 / (self.height as f64 - 1.0),
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

    pub fn try_coords_from_point(&self, point: Point) -> Option<(u32, u32)> {
        if point.x < 0 || point.y < 0 || self.index_at_checked(point.x as u32, point.y as u32).is_none() {
            None
        } else {
            Some((point.x as u32, point.y as u32))
        }
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
    
    pub fn fill_triangle(&mut self, triangle: Triangle3d, color_func: impl OrigColorFunc + Clone) {
        let u = triangle.b - triangle.a;
        let v = triangle.c - triangle.a;
        self.fill_triangle_with_basis_vecs(triangle, color_func, (u, v));
    }

    pub fn fill_triangle_with_basis_vecs(
        &mut self,
        triangle: Triangle3d,
        color_func: impl OrigColorFunc + Clone,
        basis_vecs: (Vector3d, Vector3d),
    ) {
        let adj_a = self.camera.adjust(triangle.a);
        let adj_b = self.camera.adjust(triangle.b);
        let adj_c = self.camera.adjust(triangle.c);
        let triangle = Triangle3d::new(adj_a, adj_b, adj_c);
        let triangles_to_draw = self.split_if_necessary(triangle);
        triangles_to_draw
            .iter()
            .flatten()
            .map(|tri| self.fill_translatable_triangle(*tri, color_func.clone(), basis_vecs))
            .for_each(drop);
    }

    fn split_if_necessary(&self, triangle: Triangle3d) -> [Option<Triangle3d>; 2] {
        let (a, b, c) = triangle.xsort();

        let ac = Segment::from_points(a, c);
        let ab = Segment::from_points(a, b);
        let bc = Segment::from_points(b, c);
        use SegmentTranslatability::*;
        match self.camera.can_translate_segment(bc) {
            Translatable => {
                // (a, b, c) - good
                [Some(triangle), None]
            },
            SemiTranslatable(isect_bc) => {
                // (a, b) - good, c - bad
                if let SemiTranslatable(isect_ac) = self.camera.can_translate_segment(ac) {
                    let tri1 = Triangle3d::new(a, isect_ac, isect_bc);
                    let tri2 = Triangle3d::new(a, b, isect_bc);
                    [Some(tri1), Some(tri2)]
                } else {
                    unreachable!();
                }
            },
            NonTranslatable => {
                match (self.camera.can_translate_segment(ab), self.camera.can_translate_segment(ac)) {
                    (SemiTranslatable(isect_ab), SemiTranslatable(isect_ac)) => {
                        // a - good, (b, c) - bad
                        let tri = Triangle3d::new(a, isect_ab, isect_ac);
                        [Some(tri), None]
                    },
                    (NonTranslatable, NonTranslatable) => {
                        // (a, b, c) - bad
                        [None, None]
                    },
                    _ => unreachable!(),
                }
            }
        }
    }

    fn fill_translatable_triangle(
        &mut self,
        triangle: Triangle3d,
        color_func: impl OrigColorFunc,
        basis_vecs: (Vector3d, Vector3d),
    ) {
        let maybe_tri_and_depths = self.translate_tri(triangle);
        if let Some(triangle_on_screen) = maybe_tri_and_depths {
            let adapted = CoordsTranslationAdapter::new(
                color_func,
                triangle,
                triangle_on_screen,
                basis_vecs,
                &self.viewport,
                &self.camera,
            );
            let mut proxied = DepthBufferProxy::new(adapted, &mut self.depth_buffer);
            self.rasterizer.fill_triangle(triangle_on_screen, &mut proxied);
        }
    }

    fn translate_tri(&self, tri: Triangle3d) -> Option<Triangle> {
        let (a, _da) = self.translate_point(tri.a);
        let (b, _db) = self.translate_point(tri.b);
        let (c, _dc) = self.translate_point(tri.c);
        Triangle::try_new(a, b, c)
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

    pub fn fill_triangle(&mut self, tri: Triangle, color_func: &mut impl ColorFunc) {
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

    pub fn fill_glued_triangle(&mut self, glued_tri: GluedTriangle, color_func: &mut impl ColorFunc) {
        let mut min = glued_tri.horizontal_segment.y();
        let mut max = glued_tri.free_point.y;
        if min > max {
            mem::swap(&mut min, &mut max);
        }

        let  left_line = Line::from_points(glued_tri.horizontal_segment.left(),  glued_tri.free_point);
        let right_line = Line::from_points(glued_tri.horizontal_segment.right(), glued_tri.free_point);

        for y in (min.max(0))..=(max.min(self.height as i32 - 1)) {
            let horizontal_line = Line::horizontal(y);
            let  left_isect = horizontal_line.intersect(left_line);
            let right_isect = horizontal_line.intersect(right_line);

            for x in (left_isect.x.max(0))..=(right_isect.x.min(self.width as i32 - 1)) {
                let point = Point {x, y};
                if let Some(color) = color_func.color_at(point) {
                    self.set(x as u32, y as u32, color);
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


pub trait ColorFunc {
    fn color_at(&mut self, point: Point) -> Option<RGB>;
}

impl <T: Fn(Point) -> Option<RGB>> ColorFunc for T {
    fn color_at(&mut self, point: Point) -> Option<RGB> {
        self(point)
    }
}


pub trait OrigColorFunc {
    fn color_at(&mut self, point: BasicPoint<f64>) -> Option<RGB>;
}

impl<T: Fn(BasicPoint<f64>) -> Option<RGB>> OrigColorFunc for T {
    fn color_at(&mut self, point: BasicPoint<f64>) -> Option<RGB> {
        self(point)
    }
}


struct CoordsTranslationAdapter<'a, 'b, F: OrigColorFunc> {
    color_func: F,
    triangle: Triangle3d,
    triangle_on_screen: Triangle,
    basis_vecs: (Vector3d, Vector3d),
    cramer_denom_recip: f64,
    viewport: &'a Viewport,
    camera: &'b Camera,
}

impl<'a, 'b, F: OrigColorFunc> CoordsTranslationAdapter<'a, 'b, F> {
    pub fn new(
        color_func: F,
        triangle: Triangle3d,
        triangle_on_screen: Triangle,
        basis_vecs: (Vector3d, Vector3d),
        viewport: &'a Viewport,
        camera: &'b Camera,
    ) -> Self {
        let (u, v) = basis_vecs;
        let basis_matrix_xy = Matrix2d::from_columns(u.onto_xy().into(), v.onto_xy().into());
        let cramer_denom_recip = basis_matrix_xy.det().recip();
        Self {
            color_func,
            triangle,
            triangle_on_screen,
            basis_vecs,
            cramer_denom_recip,
            viewport,
            camera,
        }
    }

    // Coords on screen -> coords on triangle
    pub fn untranslate(&self, point_on_screen: Point) -> BasicPoint<f64> {
        let projected_3d_point = self.bilinear_untranslate(point_on_screen);

        let s = BasicVector {x: projected_3d_point.x, y: projected_3d_point.y};
        let (p, q) = self.basis_vecs;

        let det_no_u = Matrix2d::from_columns(s.into(), q.onto_xy().into()).det();
        let det_no_v = Matrix2d::from_columns(p.onto_xy().into(), s.into()).det();

        // Find the coordinates of the projected point w.r.t. p and q using Cramer's rule
        let u = det_no_u * self.cramer_denom_recip;
        let v = det_no_v * self.cramer_denom_recip;

        BasicPoint {x: u, y: v}
    }

    fn bilinear_untranslate(&self, point_on_screen: Point) -> Point3d {
        if point_on_screen == self.triangle_on_screen.a {
            return self.triangle.a;
        }
        if point_on_screen == self.triangle_on_screen.b {
            return self.triangle.b;
        }
        if point_on_screen == self.triangle_on_screen.c {
            return self.triangle.c;
        }

        let base_line = Line::from_points(self.triangle_on_screen.b, self.triangle_on_screen.c);
        let pivot_line = Line::from_points(self.triangle_on_screen.a, point_on_screen);
        let pivot_point = base_line.intersect(pivot_line);
        let pivot_point_3d = self.linear_untranslate(
            pivot_point,
            (self.triangle_on_screen.b, self.triangle.b),
            (self.triangle_on_screen.c, self.triangle.c),
        );
        
        if point_on_screen == pivot_point {
            return pivot_point_3d;
        }
        
        let point_3d = self.linear_untranslate(
            point_on_screen,
            (self.triangle_on_screen.a, self.triangle.a),
            (pivot_point, pivot_point_3d),
        );
        point_3d
    }

    fn linear_untranslate(&self, p: Point, a: (Point, Point3d), b: (Point, Point3d)) -> Point3d {
        let (a2, a3) = a;
        let (b2, b3) = b;

        let line = Line3d::from_points(a3, b3);
        let origin = Point3d::from(self.onto_screen_plane(p));
        let vec1 = Vector3d::from(self.onto_screen_plane((b2 - a2).perp()));
        let vec2 = origin.as_vector();  // From the origin in the direction of `p`
        let plane = Plane::from_origin_and_vectors(origin, vec1, vec2);

        plane.intersect(line).unwrap()
    }

    fn onto_screen_plane(&self, p: impl Into<(i32, i32)>) -> (f64, f64, f64) {
        let p = Point::from(p.into());
        let viewport_agnostic_p = self.viewport.untranslate(p);
        let BasicPoint3d {x, y, z} = self.camera.onto_screen_plane(viewport_agnostic_p);
        (x, y, z)
    }
}

impl<F: OrigColorFunc> ColorFunc for CoordsTranslationAdapter<'_, '_, F> {
    fn color_at(&mut self, point: Point) -> Option<RGB> {
        let untranslated = self.untranslate(point);
        self.color_func.color_at(untranslated)
    }
}


pub trait PointCoords {
    fn coords_of(&self, point_on_screen: Point) -> Point3d;
}

impl<F: OrigColorFunc> PointCoords for CoordsTranslationAdapter<'_, '_, F> {
    fn coords_of(&self, point_on_screen: Point) -> Point3d {
        self.bilinear_untranslate(point_on_screen)
    }
}

pub trait PointDepth {
    fn depth_of(&self, point_on_screen: Point) -> f64;
}

impl<T: PointCoords> PointDepth for T {
    fn depth_of(&self, point_on_screen: Point) -> f64 {
        self.coords_of(point_on_screen).as_vector().norm()
    }
}


struct DepthBufferProxy<'a, F: ColorFunc> {
    color_func: F,
    depth_buffer: &'a mut DepthBuffer,
}

impl<'a, F: ColorFunc> DepthBufferProxy<'a, F> {
    pub fn new(color_func: F, depth_buffer: &'a mut DepthBuffer) -> Self {
        Self {color_func, depth_buffer}
    }
}

impl<F: ColorFunc + PointCoords + PointDepth> ColorFunc for DepthBufferProxy<'_, F> {
    fn color_at(&mut self, point: Point) -> Option<RGB> {
        self.depth_buffer.try_coords_from_point(point).and_then(|(x, y)| {
            let depth = self.color_func.depth_of(point);
            if self.depth_buffer.try_update(x, y, depth as f32) {
                self.color_func.color_at(point)
            } else {
                None
            }
        })
    }
}
