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

impl<T: Copy> BasicPoint<T> {
    pub fn as_vector(self) -> BasicVector<T> {
        BasicVector {x: self.x, y: self.y}
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

impl<P: PartialEq> BasicTriangle<P> {
    pub fn new(a: P, b: P, c: P) -> BasicTriangle<P> {
        BasicTriangle::try_new(a, b, c).expect("All three vertices of a triangle must be different")
    }

    pub fn try_new(a: P, b: P, c: P) -> Option<BasicTriangle<P>> {
        if a == b || b == c || a == c {
            None
        } else {
            Some(BasicTriangle {a, b, c, _private: ()})
        }
    }
}

pub type Triangle = BasicTriangle<Point>;

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
    type Output;
    fn norm(&self) -> Self::Output;
}

impl Norm for BasicVector<f64> {
    type Output = f64;

    fn norm(&self) -> f64 {
        self.x.hypot(self.y)
    }
}

impl Norm for BasicVector3d<f64> {
    type Output = f64;

    fn norm(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
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
