use std::{ops::*, fmt::Display};
use crate::*;

generic_matrix!{doc = "2x2 matrix (row-major order)"; Mat2, 2, 2}

impl<T: Real> Mat2<T> {
    /// Create a new matrix with the given values
    #[inline]
    #[must_use]
    pub fn new(m00: T, m01: T, m10: T, m11: T) -> Self {
        Self { vals: [m00, m01, 
                      m10, m11] }
    }

    /// Create a new matrix with the given rows
    #[inline]
    #[must_use]
    pub fn from_rows(row0: Vec3<T>, row1: Vec3<T>) -> Self {
        Self { vals: [row0.x, row0.y, 
                      row1.x, row1.y] }
    }

    /// Create a new matrix with the given columsn
    #[inline]
    #[must_use]
    pub fn from_columns(column0: Vec3<T>, column1: Vec3<T>) -> Self {
        Self { vals: [column0.x, column1.x,
                      column0.y, column1.y] }
    }

    /// Get the row at the given index
    #[inline]
    #[must_use]
    pub fn row(self, index: usize) -> Vec2<T> {
        debug_assert!(index < 2);
        let idx = index * 2;
        Vec2::new(self.vals[idx], self.vals[idx + 1])
    }

    /// Set the row at the given index
    #[inline]
    pub fn set_row(&mut self, index: usize, row: Vec2<T>) {
        debug_assert!(index < 2);
        let idx = index * 2;
        self.vals[idx + 0] = row.x;
        self.vals[idx + 1] = row.y;
    }

    /// Get the column at the given index
    #[inline]
    #[must_use]
    pub fn column(self, index: usize) -> Vec2<T> {
        debug_assert!(index < 2);
        Vec2::new(self.vals[index], self.vals[index + 2])
    }

    /// Set the column at the given index
    #[inline]
    pub fn set_column(&mut self, index: usize, column: Vec2<T>) {
        debug_assert!(index < 2);
        self.vals[index +  0] = column.x;
        self.vals[index +  2] = column.y;
    }

    /// Get the diagonal
    #[inline]
    #[must_use]
    pub fn diagonal(self) -> Vec2<T> {
        Vec2 { x: self.vals[0], y: self.vals[3] }
    }

    /// Set the diagonal
    #[inline]
    pub fn set_diagonal(&mut self, diag: Vec2<T>) {
        self.vals[ 0] = diag.x;
        self.vals[ 3] = diag.y;
    }

    /// Get the identity matrix
    #[inline]
    pub fn identity() -> Self {
        let zero = T::zero();
        let one = T::one();

        Self { vals: [one , zero,
                      zero, one ] }
    }

    /// Calculate the determinant
    pub fn determinant(self) -> T {
        self[0] * self[3] - self[1] * self[2]
    }

    /// Calculate the trace
    #[inline]
    pub fn trace(self) -> T {
        self[0] + self[3]
    }

    /// Transpose the matrix
    #[inline]
    pub fn transpose(self) -> Self {
        Self { vals: [self[0], self[2],
                      self[1], self[3]] }
    }

    /// Calculate the adjugate
    pub fn adjugate(self) -> Self {
        Self { vals: [ self[3], -self[1],
                      -self[2],  self[0]] }
    }

    /// Calculate the adjugate (transpose cofactor)
    pub fn cofactor(self) -> Self {
        Self { vals: [ self[3], -self[2],
                      -self[1],  self[0]] }
    }

    /// Calculate the inverse
    pub fn inverse(self) -> Self {
        let det = self.determinant();
        if det.is_zero() {
            Self::zero()
        } else {
            self.adjugate() * det.recip()
        }
    }

    /// Transform a `Vec3`
    pub fn transform(self, vec: Vec2<T>) -> Vec2<T> {
        let row0 = self.row(0);
        let row1 = self.row(1);

        row0 * vec.x + row1 * vec.y
    }

    /// Decompose the matrix into a 2D scale and rotation
    fn decompose(self) -> (Vec2<T>, Radians<T>) where
        Radians<T>: InvTrig<T>
    {
        let scale = Vec2::new(self.column(0).len(), self.column(1).len());
        let angle = Radians::acos(self[0] / scale.x);

        (scale, angle)
    }

    /// Create a 2d scale matrix
    pub fn create_scale(scale: Vec2<T>) -> Self {
        let zero = T::zero();
        Self { vals: [scale.x, zero   ,
                      zero   , scale.y] }
    }

    /// Create a 2d shear matrix
    pub fn create_shear(shear: Vec2<T>) -> Self {
        let one = T::one();
        Self { vals: [one    , shear.y,
                      shear.x, one    ] }
    }

    /// Create a 2d rotation matrix
    pub fn create_rotation(angle: Radians<T>) -> Self where
        Radians<T>: Trig<Output = T>
    {
        let (sin, cos) = angle.sin_cos();

        Self { vals: [cos , -sin,
                      sin ,  cos] }
    }


    /// Create a 2d transformation matrix
    pub fn create_transform(scale: Vec2<T>, angle: Radians<T>) -> Self where
        Radians<T>: Trig<Output = T>
    {
        let (sin, cos) = angle.sin_cos();

        Self { vals: [scale.x * cos, scale.y * -sin,
                      scale.y * sin, scale.x *  cos] }
    }

}

impl<T: Real> Mul for Mat2<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let row0 = self.row(0);
        let row1 = self.row(1);

        let column0 = rhs.column(0);
        let column1 = rhs.column(1);

        Self { vals : [row0.dot(column0), row0.dot(column1),
                       row1.dot(column0), row1.dot(column1)] }
    }
}

impl<T: Real> MulAssign for Mat2<T> {
    fn mul_assign(&mut self, rhs: Self) {
        let row0 = self.row(0);
        let row1 = self.row(1);

        let column0 = rhs.column(0);
        let column1 = rhs.column(1);

        self[0] = row0.dot(column0);
        self[1] = row0.dot(column1);
        self[2] = row1.dot(column0);
        self[3] = row1.dot(column1);
    }
}

impl<T: Real + Display> Display for Mat2<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("[[{}, {}], [{}, {}]",
                    self[0], self[1],
                    self[2], self[3]))
    }
}

#[allow(non_camel_case_types)] type f32m2 = Mat2<f32>;
#[allow(non_camel_case_types)] type f64m2 = Mat2<f64>;