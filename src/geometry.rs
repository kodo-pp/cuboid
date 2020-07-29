use std::ops::{Sub, Add, Neg};
use std::mem;
use gcd::Gcd;


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
pub struct Vector {
    pub x: i32,
    pub y: i32,
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


#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Triangle {
    pub a: Point,
    pub b: Point,
    pub c: Point,
}


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
}


#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct GluedTriangle {
    pub horizontal_segment: HorizontalSegment,
    pub free_point: Point,
}

impl GluedTriangle {
    pub fn new(horizontal_segment: HorizontalSegment, free_point: Point) -> GluedTriangle {
        GluedTriangle { horizontal_segment, free_point }
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
