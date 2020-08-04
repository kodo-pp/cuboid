use crate::linalg::Matrix2d;

use gcd::Gcd;
use std::ops::{Sub, Add, Mul, Div, Neg};
use std::mem;
use std::fmt::Debug;
use core::f64::consts::{PI, FRAC_PI_2};


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BasicPoint<T> {
    pub x: T,
    pub y: T,
}

pub type Point = BasicPoint<i32>;

impl<T> From<(T, T)> for BasicPoint<T> {
    fn from(tuple: (T, T)) -> BasicPoint<T> {
        BasicPoint {x: tuple.0, y: tuple.1}
    }
}

impl<T> Into<(T, T)> for BasicPoint<T> {
    fn into(self) -> (T, T) {
        (self.x, self.y)
    }
}

impl<O, B, A: Add<B, Output = O>> Add<BasicVector<B>> for BasicPoint<A> {
    type Output = BasicPoint<O>;

    fn add(self, other: BasicVector<B>) -> BasicPoint<O> {
        BasicPoint { x: self.x + other.x, y: self.y + other.y }
    }
}

impl<O, B, A: Sub<B, Output = O>> Sub<BasicPoint<B>> for BasicPoint<A> {
    type Output = BasicVector<O>;

    fn sub(self, other: BasicPoint<B>) -> BasicVector<O> {
        BasicVector { x: self.x - other.x, y: self.y - other.y }
    }
}

impl<O, B, A: Sub<B, Output = O>> Sub<BasicVector<B>> for BasicPoint<A> {
    type Output = BasicPoint<O>;

    fn sub(self, other: BasicVector<B>) -> BasicPoint<O> {
        BasicPoint { x: self.x - other.x, y: self.y - other.y }
    }
}


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BasicVector<T> {
    pub x: T,
    pub y: T,
}

impl<T> BasicVector<T> {
    pub fn map<P>(self, func: &impl Fn(T) -> P) -> BasicVector<P> {
        BasicVector {
            x: func(self.x),
            y: func(self.y),
        }
    }
}

impl<T> Into<(T, T)> for BasicVector<T> {
    fn into(self) -> (T, T) {
        (self.x, self.y)
    }
}

impl<T> From<(T, T)> for BasicVector<T> {
    fn from(tuple: (T, T)) -> BasicVector<T> {
        BasicVector {x: tuple.0, y: tuple.1}
    }
}

impl<O, B, A: Add<B, Output = O>> Add<BasicVector<B>> for BasicVector<A> {
    type Output = BasicVector<O>;

    fn add(self, other: BasicVector<B>) -> BasicVector<O> {
        BasicVector { x: self.x + other.x, y: self.y + other.y }
    }
}

impl<O, B, A: Sub<B, Output = O>> Sub<BasicVector<B>> for BasicVector<A> {
    type Output = BasicVector<O>;

    fn sub(self, other: BasicVector<B>) -> BasicVector<O> {
        BasicVector { x: self.x - other.x, y: self.y - other.y }
    }
}

impl<O, T: Neg<Output = O>> Neg for BasicVector<T> {
    type Output = BasicVector<O>;

    fn neg(self) -> BasicVector<O> {
        BasicVector { x: -self.x, y: -self.y }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BasicTriangle<P> {
    pub a: P,
    pub b: P,
    pub c: P,
    
    _private: (),  // See https://stackoverflow.com/questions/53588819#53589431
}

impl<P: PartialEq + LiesOn> BasicTriangle<P> {
    pub fn new(a: P, b: P, c: P) -> BasicTriangle<P> {
        BasicTriangle::try_new(a, b, c).expect("The triangle is invalid")
    }

    pub fn try_new(a: P, b: P, c: P) -> Option<BasicTriangle<P>> {
        if a == b || b == c || a == c {
            None
        } else if a.lies_on(&b, &c) || b.lies_on(&a, &c) || c.lies_on(&a, &b) {
            None
        } else {
            Some(BasicTriangle {a, b, c, _private: ()})
        }
    }
}

pub type Triangle = BasicTriangle<Point>;
pub type Triangle3d = BasicTriangle<Point3d>;

#[derive(Debug, Clone, Copy)]
pub struct Line {
    a: i32,
    b: i32,
    c: i32,
}

impl Line {
    pub fn from_points(p: Point, q: Point) -> Line {
        if p == q {
            panic!("More than one line passes through two coinciding points");
        }

        if p.x == q.x {
            Line { a: 1, b: 0, c: -p.x }
        } else {
            Line { a: p.y - q.y, b: q.x - p.x, c: p.x * q.y - p.y * q.x }
        }
    }

    pub fn horizontal(y: i32) -> Line {
        Line { a: 0, b: 1, c: -y }
    }

    pub fn intersect(self, other: Line) -> Point {
        let x_numerator = self.c * other.b - self.b * other.c;
        let y_numerator = self.a * other.c - self.c * other.a;
        let denominator = self.b * other.a - self.a * other.b;
        Point { x: x_numerator / denominator, y: y_numerator / denominator }
    }

    pub fn contains_point(self, point: Point) -> bool {
        self.a * point.x + self.b * point.y + self.c == 0
    }
}

impl PartialEq for Line {
    fn eq(&self, other: &Line) -> bool {
        let ga = (self.a.abs() as u32).gcd(other.a.abs() as u32) as i32;
        let p = (self.a / ga) as i64;
        let q = (other.a / ga) as i64;
        
        let  first = self.a as i64 * q == other.a as i64 * p;
        let second = self.b as i64 * q == other.b as i64 * p;
        let  third = self.c as i64 * q == other.c as i64 * p;
        first && second && third
    }
}

impl Eq for Line {}


#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct HorizontalSegment {
    left: Point,
    width: u32,
}

impl HorizontalSegment {
    pub fn from_points(mut left: Point, mut right: Point) -> HorizontalSegment {
        assert_eq!(left.y, right.y, "Y coordinates of points of a horizontal segment must coincide");
        if left.x > right.x {
            mem::swap(&mut left, &mut right);
        }
        HorizontalSegment { left, width: (right.x - left.x) as u32 }
    }

    pub fn left(self) -> Point {
        self.left
    }

    pub fn right(self) -> Point {
        Point { x: self.left.x + (self.width as i32), y: self.left.y }
    }

    pub fn y(self) -> i32 {
        self.left.y
    }

    pub fn as_line(self) -> Line {
        Line::horizontal(self.y())
    }
}


#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct GluedTriangle {
    pub horizontal_segment: HorizontalSegment,
    pub free_point: Point,
}

impl GluedTriangle {
    #[allow(dead_code)]
    pub fn new(horizontal_segment: HorizontalSegment, free_point: Point) -> GluedTriangle {
        GluedTriangle::try_new(horizontal_segment, free_point)
            .expect("GluedTriangle's free point cannot lie on the same line as its horizontal segment")
    }

    pub fn try_new(horizontal_segment: HorizontalSegment, free_point: Point) -> Option<GluedTriangle> {
        if horizontal_segment.as_line().contains_point(free_point) {
            None
        } else {
            Some(GluedTriangle {horizontal_segment, free_point})
        }
    }
}


pub trait Triangular {
    fn points(&self) -> (Point, Point, Point);

    fn sort_points<K: Ord>(&self, key: impl Fn(Point) -> K) -> (Point, Point, Point) {
        let (mut a, mut b, mut c) = self.points();
        if key(a) > key(b) {
            mem::swap(&mut a, &mut b);
        }
        if key(b) > key(c) {
            mem::swap(&mut b, &mut c);
        }
        if key(a) > key(b) {
            mem::swap(&mut a, &mut b);
        }
        (a, b, c)
    }

    fn xsort(&self) -> (Point, Point, Point) {
        self.sort_points(|p| p.x)
    }

    fn ysort(&self) -> (Point, Point, Point) {
        self.sort_points(|p| p.y)
    }
}

impl Triangular for Triangle {
    fn points(&self) -> (Point, Point, Point) {
        (self.a, self.b, self.c)
    }
}

impl Triangular for GluedTriangle {
    fn points(&self) -> (Point, Point, Point) {
        (self.free_point, self.horizontal_segment.left(), self.horizontal_segment.right())
    }
}


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BasicPoint3d<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

pub type Point3d = BasicPoint3d<f64>;

impl<T> BasicPoint3d<T> {
    pub fn as_vector(self) -> BasicVector3d<T> {
        BasicVector3d {x: self.x, y: self.y, z: self.z}
    }
}

impl<O, B, A: Add<B, Output = O>> Add<BasicVector3d<B>> for BasicPoint3d<A> {
    type Output = BasicPoint3d<O>;

    fn add(self, other: BasicVector3d<B>) -> BasicPoint3d<O> {
        BasicPoint3d { x: self.x + other.x, y: self.y + other.y, z: self.z + other.z }
    }
}

impl<O, B, A: Sub<B, Output = O>> Sub<BasicPoint3d<B>> for BasicPoint3d<A> {
    type Output = BasicVector3d<O>;

    fn sub(self, other: BasicPoint3d<B>) -> BasicVector3d<O> {
        BasicVector3d { x: self.x - other.x, y: self.y - other.y, z: self.z - other.z }
    }
}

impl<O, B, A: Sub<B, Output = O>> Sub<BasicVector3d<B>> for BasicPoint3d<A> {
    type Output = BasicPoint3d<O>;

    fn sub(self, other: BasicVector3d<B>) -> BasicPoint3d<O> {
        BasicPoint3d { x: self.x - other.x, y: self.y - other.y, z: self.z - other.z }
    }
}


#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct BasicVector3d<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

pub type Vector3d = BasicVector3d<f64>;

impl Vector3d {
    pub fn approx(self, other: Vector3d) -> bool {
        (self - other).norm_sq() < 1e-10
    }
}

impl<T> BasicVector3d<T> {
    pub fn as_point(self) -> BasicPoint3d<T> {
        BasicPoint3d {x: self.x, y: self.y, z: self.z}
    }
}

impl<O, B, A: Add<B, Output = O>> Add<BasicVector3d<B>> for BasicVector3d<A> {
    type Output = BasicVector3d<O>;

    fn add(self, other: BasicVector3d<B>) -> BasicVector3d<O> {
        BasicVector3d { x: self.x + other.x, y: self.y + other.y, z: self.z + other.z }
    }
}

impl<O, B, A: Sub<B, Output = O>> Sub<BasicVector3d<B>> for BasicVector3d<A> {
    type Output = BasicVector3d<O>;

    fn sub(self, other: BasicVector3d<B>) -> BasicVector3d<O> {
        BasicVector3d { x: self.x - other.x, y: self.y - other.y, z: self.z - other.z }
    }
}

impl<O, T: Neg<Output = O>> Neg for BasicVector3d<T> {
    type Output = BasicVector3d<O>;

    fn neg(self) -> BasicVector3d<O> {
        BasicVector3d { x: -self.x, y: -self.y, z: -self.z }
    }
}


#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Angle(f64);

impl Angle {
    pub fn zero() -> Angle {
        Angle(0.0)
    }

    pub fn from_radians(radians: f64) -> Angle {
        Angle(radians)
    }

    pub fn from_degrees(degrees: f64) -> Angle {
        Angle(degrees.to_radians())
    }

    pub fn quarter_circle() -> Angle {
        Angle(FRAC_PI_2)
    }

    pub fn half_circle() -> Angle {
        Angle(PI)
    }

    pub fn circle() -> Angle {
        Angle(2.0 * PI)
    }

    pub fn into_plus_minus_pi_interval(self) -> Angle {
        let zero_2pi_angle = self.into_zero_2pi_interval();
        if zero_2pi_angle > Angle::half_circle() {
            zero_2pi_angle - Angle::circle()
        } else {
            zero_2pi_angle
        }
    }

    pub fn into_zero_2pi_interval(self) -> Angle {
        Angle(self.0.rem_euclid(2.0 * PI))
    }

    #[allow(dead_code)]
    pub fn as_radians(self) -> f64 {
        self.0
    }

    #[allow(dead_code)]
    pub fn as_degrees(self) -> f64 {
        self.0.to_degrees()
    }

    pub fn rotation_matrix(self) -> Matrix2d<f64> {
        let (sin, cos) = self.0.sin_cos();
        Matrix2d::from_rows((cos, -sin), (sin, cos))
    }
}

impl Sub for Angle {
    type Output = Angle;

    fn sub(self, other: Angle) -> Angle {
        Angle(self.0 - other.0)
    }
}

impl Mul<f64> for Angle {
    type Output = Angle;

    fn mul(self, scalar: f64) -> Angle {
        Angle(self.0 * scalar)
    }
}

impl Div for Angle {
    type Output = f64;

    fn div(self, other: Angle) -> f64 {
        self.0 / other.0
    }
}

impl Div<f64> for Angle {
    type Output = Angle;

    fn div(self, scalar: f64) -> Angle {
        Angle(self.0 / scalar)
    }
}


pub trait Dot<T> {
    type Output;
    fn dot(&self, other: &T) -> Self::Output;
}

impl<AO, MO: Add<Output = AO>, B: Copy, A: Mul<B, Output=MO> + Copy> Dot<BasicVector<B>> for BasicVector<A> {
    type Output = AO;

    // BasVec<A> · BasVec<B> = (A * B) + (A * B) = MO + MO = AO
    fn dot(&self, other: &BasicVector<B>) -> AO {
        self.x * other.x + self.y * other.y
    }
}

impl<
    AO2,
    AO1: Add<MO, Output = AO2>,
    MO: Add<Output = AO1>,
    B: Copy,
    A: Mul<B, Output = MO> + Copy
> Dot<BasicVector3d<B>> for BasicVector3d<A> {
    type Output = AO2;

    // BasVec3<A> · BasVec3<B> = ((A * B) + (A * B)) + (A * B) = (MO + MO) + MO = AO1 + MO = AO2
    fn dot(&self, other: &BasicVector3d<B>) -> AO2 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}


pub trait Norm {
    type Output: Mul<Output = Self::Output>;

    fn norm(&self) -> Self::Output;
    fn norm_sq(&self) -> Self::Output {
        let x = self.norm();
        x * x
    }
}

impl Norm for BasicVector<f64> {
    type Output = f64;

    fn norm(&self) -> f64 {
        self.x.hypot(self.y)
    }

    fn norm_sq(&self) -> f64 {
        self.x.powi(2) + self.y.powi(2)
    }
}

impl Norm for BasicVector3d<f64> {
    type Output = f64;

    fn norm(&self) -> f64 {
        self.norm_sq().sqrt()
    }

    fn norm_sq(&self) -> f64 {
        self.x.powi(2) + self.y.powi(2) + self.z.powi(2)
    }
}


pub trait AngleWith<T> {
    fn angle_with(&self, other: &T) -> Angle
    where
        Self: Dot<T, Output = f64> + Norm<Output = f64>,
        T: Norm<Output = f64> {
        let dot = self.dot(other);
        let norms = self.norm() * other.norm();
        Angle::from_radians((dot / norms).acos())
    }
}

impl<T, V: Dot<T, Output = f64> + Norm<Output = f64>> AngleWith<T> for V {}


pub trait Azimuth {
    fn azimuth(&self) -> Angle;
}

impl Azimuth for BasicVector<f64> {
    fn azimuth(&self) -> Angle {
        Angle::from_radians(self.y.atan2(self.x))
    }
}

impl Azimuth for Vector3d {
    fn azimuth(&self) -> Angle {
        (BasicVector {x: self.x, y: self.z}).azimuth()
    }
}


pub trait Vangle {
    fn vangle(&self) -> Angle;
}

impl Vangle for Vector3d {
    fn vangle(&self) -> Angle {
        let xz_projection = self.onto_xz();
        let angle_abs = self.angle_with(xz_projection);
        let angle_sign = self.y.signum();
        angle_abs * angle_sign
    }
}


pub trait LiesOn {
    fn lies_on(&self, a: &Self, b: &Self) -> bool;
}

impl LiesOn for Point {
    fn lies_on(&self, a: &Point, b: &Point) -> bool {
        Line::from_points(*a, *b).contains_point(*self)
    }
}

impl LiesOn for Point3d {
    fn lies_on(&self, a: &Point3d, b: &Point3d) -> bool {
        (*a - *self).angle_with(&(*b - *self)).as_radians().abs() < 1e-10
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Par3d {
    origin: Point3d,
    vec1: Vector3d,
    vec2: Vector3d,
}

impl Par3d {
    pub fn new(origin: Point3d, vec1: Vector3d, vec2: Vector3d) -> Par3d {
        Self::try_new(origin, vec1, vec2).expect("Vectors in a 3D parallelogram must not be collinear")
    }

    pub fn try_new(origin: Point3d, vec1: Vector3d, vec2: Vector3d) -> Option<Par3d> {
        let angle = vec1.angle_with(&vec2);
        if angle.as_radians().abs() < 1e-12 || (angle - Angle::half_circle()).as_radians().abs() < 1e-12 { 
            // Vectors are collinear
            None
        } else {
            Some(Par3d {origin, vec1, vec2})
        }
    }

    pub fn to_triangles(&self) -> (Triangle3d, Triangle3d) {
        let a1 = self.origin;
        let b1 = a1 + self.vec1;
        let c1 = a1 + self.vec2;
        let tri1 = Triangle3d::new(a1, b1, c1);
        let a2 = self.origin + self.vec1 + self.vec2;
        let b2 = c1;
        let c2 = b1;
        let tri2 = Triangle3d::new(a2, b2, c2);
        (tri1, tri2)
    }
}


pub trait Rotate3d {
    fn rotate_3d(self, delta_azimuth: Angle) -> Self;
}

impl Rotate3d for Vector3d {
    fn rotate_3d(self, delta_azimuth: Angle, delta_vangle: Angle) -> Self {
        let rotate_xz = |vec, ang| {
            let vec2d = BasicVector::<f64> {x: vec.x, y: vec.z};  // `y: vec.z` is not a typo
            let matrix = ang.rotation_matrix();
            let (x, z) = (matrix * vec2d)::into();
            Vector3d {x, y: vec.y, z}
        };

        let rotate_xy = |vec, ang| {
            let vec2d = BasicVector::<f64> {x: vec.x, y: vec.y};
            let matrix = ang.rotation_matrix();
            let (x, y) = (matrix * vec2d)::into();
            Vector3d {x, y, z: vec.z}
        };

        let old_azimuth = self.azimuth();
        let self_north = rotate_xz(self, -old_azimuth);
        let result_north = rotate_xy(self_north, delta_vangle); // TODO: cap vangle at ±90°
        let new_azimuth = old_azimuth + delta_azimuth;
        rotate_xz(result_north, new_azimuth)
    }
}


#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Segment<P> {
    a: P,
    b: P,
}

impl<P: Copy> Segment<P> {
    pub fn try_from_points(a: P, b: P) -> Option<Self> {
        if a == b {
            None
        } else {
            Some(Segment {a, b})
        }
    }

    pub fn from_points(a: P, b: P) -> Self {
        Self::try_from_points(a, b).expect("Ends of segment cannot coincide")
    }

    pub fn a(self) -> P {
        self.a
    }

    pub fn b(self) -> P {
        self.b
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plane {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
}

impl Plane {
    pub fn coefficients(self) -> (f64, f64, f64, f64) {
        (self.a, self.b, self.c, self.d)
    }

    pub fn basis_vectors(self) -> (Vector3d, Vector3d) {
        let vec_a = Vector3d {x: self.b * self.c,  y:              -1,  z:              -1};
        let vec_b = Vector3d {x:              -1,  y: self.a * self.c,  z:              -1};
        let vec_c = Vector3d {x:              -1,  y:              -1,  z: self.a * self.b};

        // The vectors obviously are collinear to the plane, since they are orthogonal to
        // its normal vector (a, b, c). In addition, they are constructed in such a way that
        // if u, v \in {vec_a, vec_b, vec_c}, then (u is collinear to v) \iff (u == v).
        
        // Since calculations with floating point numbers are performed, the relation "=="
        // (exactly equals) is replaced with "≈" a.k.a. `.approx()` (approximately equals).
    
        if !vec_a.approx(vec_b) {
            (vec_a, vec_b)
        } else if !vec_b.approx(vec_c) {
            (vec_b, vec_c)
        } else {
            (vec_a, vec_c)
        }
    }
}

impl From<Triangle3d> for Plane {
    fn from(tri: Triangle3d) -> Plane {
        let (p1, p2, p3) = tri.points();
        // d = -Matrix3d::from_columns(p1.into(), p2.into(), p3.into()).det()
        // But I'm too lazy to write `Matrix3d` and `Into<(T, T, T)> for BasicPoint3d`
        let d = p3.x * p2.y * p1.z
              + p1.x * p3.y * p2.z
              + p2.x * p1.y * p3.z
              - p2.x * p3.y * p1.z
              - p3.x * p1.y * p2.z
              - p1.x * p2.y * p3.z;
        
        let a = (p2.y - p3.y) * p1.z - (p1.y - p3.y) * p2.z + (p1.y - p2.y) * p3.z;
        let b = (p2.z - p3.z) * p1.x - (p1.z - p3.z) * p2.x + (p1.z - p2.z) * p3.x;
        let c = (p2.x - p3.x) * p1.y - (p1.x - p3.x) * p2.y + (p1.x - p2.x) * p3.y;
        Plane {a, b, c, d}
    }
}


pub trait OntoWithBasis<T, B> {
    type Output;

    fn onto_with_basis(self, object: T, basis: B) -> Self::Output;
}

impl OntoWithBasis<Plane, (Vector3d, Vector3d)> for Point3d {
    type Output = BasicPoint<f64>;

    fn onto_with_basis(self, plane: Plane, basis: (Vector3d, Vector3d)) -> Self::Output {
        let (a, b, c, d) = plane.coefficients();
        let t = -(a * self.x + b * self.y + c * self.z + d) / (a.powi(2) + b.powi(2) + c.powi(2));

        let x = self.x + a * t;
        let y = self.y + b * t;
        let (p, q) = basis;

        // Z coordinates are unneeded since we guarantee that the point lies on the plane
        let p = p.onto_xy();
        let q = q.onto_xy();
        let s = BasicVector {x, y};

        let matrix_full = Matrix2d::from_columns(p.into(), q.into());
        let matrix_no_p = Matrix2d::from_columns(s.into(), q.into());
        let matrix_no_q = Matrix2d::from_columns(p.into(), s.into());
        let common_factor = matrix_full.det().recip();

        // Find the coordinates of the projected point w.r.t. p and q using Cramer's rule
        let u = matrix_no_p.det() * common_factor;
        let v = matrix_no_q.det() * common_factor;

        BasicPoint {x: u, y: v}
    }
}
