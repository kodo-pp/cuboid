use std::ops::{Add, Sub, Mul, Div, Neg};


#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Matrix2d<T> {
    pub x00: T,
    pub x01: T,
    pub x10: T,
    pub x11: T,
}

impl<
    T: Sub<Output = T>
     + Mul<Output = T>
     + Div<Output = T>
     + Neg<Output = T>
     + Copy
     + PartialEq
     + Into<f64>
> Matrix2d<T> {
    pub fn from_rows(row0: (T, T), row1: (T, T)) -> Matrix2d<T> {
        Matrix2d {
            x00: row0.0,
            x01: row0.1,
            x10: row1.0,
            x11: row1.1,
        }
    }

    pub fn from_columns(col0: (T, T), col1: (T, T)) -> Matrix2d<T> {
        Matrix2d {
            x00: col0.0,
            x01: col1.0,
            x10: col0.1,
            x11: col1.1,
        }
    }

    pub fn det(self) -> T {
        self.x00 * self.x11 - self.x01 * self.x10
    }
}

impl<
    T: Sub<Output = T>
     + Mul<Output = T>
     + Div<Output = T>
     + Neg<Output = T>
     + Copy
     + PartialEq
     + Into<f64>
> Div<T> for Matrix2d<T> {
    type Output = Matrix2d<T>;

    fn div(self, scalar: T) -> Matrix2d<T> {
        Matrix2d::from_rows((self.x00 / scalar, self.x01 / scalar), (self.x10 / scalar, self.x11 / scalar))
    }
}

impl<T: Add<Output = T> + Mul<Output = T> + Copy, V: Into<(T, T)> + From<(T, T)>> Mul<V> for Matrix2d<T> {
    type Output = V;
    
    fn mul(self, vec: V) -> V {
        let (a, b): (T, T) = vec.into();
        let c = self.x00 * a + self.x01 * b;
        let d = self.x10 * a + self.x11 * b;
        V::from((c, d))
    }
}
