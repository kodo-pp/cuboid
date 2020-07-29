use sdl2::rect::Rect;
use std::ops::{Sub, Add, Neg};


#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Add<Vector> for Point {
    type Output = Point;

    fn add(self, other: Vector) -> Point {
        Point { x: self.x + other.x, y: self.y + other.y }
    }
}

impl Sub<Point> for Point {
    type Output = Vector;

    fn sub(self, other: Point) -> Vector {
        Vector { x: self.x - other.x, y: self.y - other.y }
    }
}

impl Sub<Vector> for Point {
    type Output = Point;

    fn sub(self, other: Vector) -> Point {
        Point { x: self.x - other.x, y: self.y - other.y }
    }
}


#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Orientation {
    Positive,
    Negative,
    Indeterminate,
}


#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Vector {
    pub x: i32,
    pub y: i32,
}

impl Vector {
    fn orientation(self, other: Vector) -> Orientation {
        let value = self.x * other.y - self.y * other.x;
        if value < 0 {
            Orientation::Negative
        } else if value > 0 {
            Orientation::Positive
        } else {
            Orientation::Indeterminate
        }
    }
}

impl Add for Vector {
    type Output = Vector;

    fn add(self, other: Vector) -> Vector {
        Vector { x: self.x + other.x, y: self.y + other.y }
    }
}

impl Sub for Vector {
    type Output = Vector;

    fn sub(self, other: Vector) -> Vector {
        Vector { x: self.x - other.x, y: self.y - other.y }
    }
}

impl Neg for Vector {
    type Output = Vector;

    fn neg(self) -> Vector {
        Vector { x: -self.x, y: -self.y }
    }
}


#[inline]
fn equal3<T: PartialEq + Copy>(a: T, b: T, c: T) -> bool {
    return a == b && a == c;
}


#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Triangle {
    pub a: Point,
    pub b: Point,
    pub c: Point,
}

impl Triangle {
    pub fn bounding_rect(&self) -> Rect {
        let left   = self.a.x.min(self.b.x).min(self.c.x);
        let top    = self.a.y.min(self.b.y).min(self.c.y);
        let right  = self.a.x.max(self.b.x).max(self.c.x);
        let bottom = self.a.y.max(self.b.y).max(self.c.y);
        let width = right - left;
        let height = bottom - top;
        Rect::new(left, top, width as u32, height as u32)
    }

    pub fn contains(&self, point: Point) -> bool {
        let vec_ap = point - self.a;
        let vec_bp = point - self.b;
        let vec_cp = point - self.c;
        let vec_ab = self.b - self.a;
        let vec_bc = self.c - self.b;
        let vec_ca = self.a - self.c;
        equal3(vec_ab.orientation(vec_ap), vec_bc.orientation(vec_bp), vec_ca.orientation(vec_cp))
    }
}
