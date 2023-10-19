use std::{ops::{Mul, MulAssign}, fmt::Display};
use crate::{*, angle::Radians};

impl<T: Real> Mat4<T> {
    /// Create a new matrix with the given values
    #[inline]
    #[must_use]
    pub fn new(m00: T, m01: T, m02: T, m03: T, m10: T, m11: T, m12: T, m13: T, m20: T, m21: T, m22: T, m23: T, m30: T, m31: T, m32: T, m33: T) -> Self {
        Self { vals: [m00, m01, m02, m03, 
                      m10, m11, m12, m13, 
                      m20, m21, m22, m23, 
                      m30, m31, m32, m33] }
    }

    /// Create a new matrix with the given rows
    #[inline]
    #[must_use]
    pub fn from_rows(row0: Vec4<T>, row1: Vec4<T>, row2: Vec4<T>, row3: Vec4<T>) -> Self {
        Self { vals: [row0.x, row0.y, row0.z, row0.w, 
                      row1.x, row1.y, row1.z, row1.w, 
                      row2.x, row2.y, row2.z, row2.w, 
                      row3.x, row3.y, row3.z, row3.w] }
    }

    /// Create a new matrix with the given columsn
    #[inline]
    #[must_use]
    pub fn from_columns(column0: Vec4<T>, column1: Vec4<T>, column2: Vec4<T>, column3: Vec4<T>) -> Self {
        Self { vals: [column0.x, column1.x, column2.x, column3.x,
                      column0.y, column1.y, column2.y, column3.y,
                      column0.z, column1.z, column2.z, column3.z,
                      column0.w, column1.w, column2.w, column3.w] }
    }

    /// Get the row at the given index
    #[inline]
    #[must_use]
    pub fn row(self, index: usize) -> Vec4<T> {
        debug_assert!(index < 4);
        let idx = index * 4;
        Vec4::new(self.vals[idx], self.vals[idx + 1], self.vals[idx + 2], self.vals[idx + 3])
    }

    /// Set the row at the given index
    #[inline]
    pub fn set_row(&mut self, index: usize, row: Vec4<T>) {
        debug_assert!(index < 4);
        let idx = index * 4;
        self.vals[idx + 0] = row.x;
        self.vals[idx + 1] = row.y;
        self.vals[idx + 2] = row.z;
        self.vals[idx + 3] = row.w;
    }

    /// Get the column at the given index
    #[inline]
    #[must_use]
    pub fn column(self, index: usize) -> Vec4<T> {
        debug_assert!(index < 4);
        Vec4::new(self.vals[index], self.vals[index + 4], self.vals[index + 8], self.vals[index + 12])
    }

    /// Set the column at the given index
    #[inline]
    pub fn set_column(&mut self, index: usize, column: Vec4<T>) {
        debug_assert!(index < 4);
        self.vals[index +  0] = column.x;
        self.vals[index +  4] = column.y;
        self.vals[index +  8] = column.z;
        self.vals[index + 12] = column.w;
    }

    /// Get the diagonal
    #[inline]
    #[must_use]
    pub fn diagonal(self) -> Vec4<T> {
        Vec4 { x: self.vals[0], y: self.vals[5], z: self.vals[10], w: self.vals[15] }
    }

    /// Set the diagonal
    #[inline]
    pub fn set_diagonal(&mut self, diag: Vec4<T>) {
        self.vals[ 0] = diag.x;
        self.vals[ 5] = diag.y;
        self.vals[10] = diag.z;
        self.vals[15] = diag.w;
    }

    /// Get the identity matrix
    #[inline]
    pub fn identity() -> Self {
        let zero = T::zero();
        let one = T::one();

        Self { vals: [one , zero, zero, zero,
                      zero, one , zero, zero,
                      zero, zero, one , zero,
                      zero, zero, zero, one ] }
    }

    /// Get the minor matrix for the given `row` and `column`
    #[inline]
    #[must_use]
    pub fn minor(self, row: usize, column: usize) -> Mat3<T> {
        debug_assert!(row < 4);
        debug_assert!(column < 4);

        match row {
            0 => match column {
                0 => Mat3{ vals: [self[5], self[6], self[7], self[9], self[10], self[11], self[13], self[14], self[15]]},
                1 => Mat3{ vals: [self[4], self[6], self[7], self[8], self[10], self[11], self[12], self[14], self[15]]},
                2 => Mat3{ vals: [self[4], self[5], self[7], self[8], self[ 9], self[10], self[12], self[13], self[15]]},
                3 => Mat3{ vals: [self[4], self[5], self[6], self[8], self[ 9], self[10], self[12], self[13], self[14]]},
                _ => unreachable!()
            },
            1 => match column {
                0 => Mat3{ vals: [self[1], self[2], self[3], self[9], self[10], self[11], self[13], self[14], self[15]]},
                1 => Mat3{ vals: [self[0], self[2], self[3], self[8], self[10], self[11], self[12], self[14], self[15]]},
                2 => Mat3{ vals: [self[0], self[1], self[3], self[8], self[ 9], self[10], self[12], self[13], self[15]]},
                3 => Mat3{ vals: [self[0], self[1], self[2], self[8], self[ 9], self[10], self[12], self[13], self[14]]},
                _ => unreachable!()
            },
            2 => match column {
                0 => Mat3{ vals: [self[1], self[2], self[3], self[5], self[ 6], self[ 7], self[13], self[14], self[15]]},
                1 => Mat3{ vals: [self[0], self[2], self[3], self[4], self[ 6], self[ 7], self[12], self[14], self[15]]},
                2 => Mat3{ vals: [self[0], self[1], self[3], self[4], self[ 5], self[ 7], self[12], self[13], self[15]]},
                3 => Mat3{ vals: [self[0], self[1], self[2], self[4], self[ 5], self[ 6], self[12], self[13], self[14]]},
                _ => unreachable!()
            },
            3 => match column {
                0 => Mat3{ vals: [self[1], self[2], self[3], self[5], self[ 6], self[ 7], self[ 9], self[10], self[11]]},
                1 => Mat3{ vals: [self[0], self[2], self[3], self[4], self[ 6], self[ 7], self[ 8], self[10], self[11]]},
                2 => Mat3{ vals: [self[0], self[1], self[3], self[4], self[ 5], self[ 7], self[ 8], self[ 9], self[10]]},
                3 => Mat3{ vals: [self[0], self[1], self[2], self[4], self[ 5], self[ 6], self[ 8], self[ 9], self[10]]},
                _ => unreachable!()
            },
            _ => unreachable!()
        }
    }

    /// Calculate the determinant
    pub fn determinant(self) -> T {
        let minor0_det = self[5] * (self[10] * self[15] - self[11] * self[14]) - self[6] * (self[9] * self[15] - self[11] * self[13]) + self[7] * (self[9] * self[14] - self[10] * self[13]);
        let minor1_det = self[4] * (self[10] * self[15] - self[11] * self[14]) - self[6] * (self[8] * self[15] - self[11] * self[12]) + self[7] * (self[8] * self[14] - self[10] * self[12]);
        let minor2_det = self[4] * (self[ 9] * self[15] - self[11] * self[13]) - self[5] * (self[8] * self[15] - self[11] * self[12]) + self[7] * (self[8] * self[13] - self[ 9] * self[12]);
        let minor3_det = self[4] * (self[ 9] * self[14] - self[10] * self[13]) - self[5] * (self[8] * self[14] - self[10] * self[12]) + self[6] * (self[8] * self[13] - self[ 9] * self[12]);

        self[0] * minor0_det - self[1] * minor1_det + self[2] * minor2_det - self[3] * minor3_det
    }

    /// Calculate the trace
    #[inline]
    pub fn trace(self) -> T {
        self[0] + self[5] + self[10] + self[15]
    }

    /// Transpose the matrix
    #[inline]
    pub fn transpose(self) -> Self {
        Self { vals: [self[0], self[4], self[ 8], self[12],
                      self[1], self[5], self[ 9], self[13],
                      self[2], self[6], self[10], self[14],
                      self[3], self[7], self[11], self[15]] }
    }

    /// Calculate the adjugate
    pub fn adjugate(self) -> Self {
        let tmp00 = self[5] * (self[10] * self[15] - self[11] * self[14]) - self[6] * (self[9] * self[15] - self[11] * self[13]) + self[7] * (self[9] * self[14] - self[10] * self[13]);
        let tmp01 = self[4] * (self[10] * self[15] - self[11] * self[14]) - self[6] * (self[8] * self[15] - self[11] * self[12]) + self[7] * (self[8] * self[14] - self[10] * self[12]);
        let tmp02 = self[4] * (self[ 9] * self[15] - self[11] * self[13]) - self[5] * (self[8] * self[15] - self[11] * self[12]) + self[7] * (self[8] * self[13] - self[ 9] * self[12]);
        let tmp03 = self[4] * (self[ 9] * self[14] - self[10] * self[13]) - self[5] * (self[8] * self[14] - self[10] * self[12]) + self[6] * (self[8] * self[13] - self[ 9] * self[12]);

        let tmp10 = self[1] * (self[10] * self[15] - self[11] * self[14]) - self[2] * (self[9] * self[15] - self[11] * self[13]) + self[3] * (self[9] * self[14] - self[10] * self[13]);
        let tmp11 = self[0] * (self[10] * self[15] - self[11] * self[14]) - self[2] * (self[8] * self[15] - self[11] * self[12]) + self[3] * (self[8] * self[14] - self[10] * self[12]);
        let tmp12 = self[0] * (self[ 9] * self[15] - self[11] * self[13]) - self[1] * (self[8] * self[15] - self[11] * self[12]) + self[3] * (self[8] * self[13] - self[ 9] * self[12]);
        let tmp13 = self[0] * (self[ 9] * self[14] - self[10] * self[13]) - self[1] * (self[8] * self[14] - self[10] * self[12]) + self[2] * (self[8] * self[13] - self[ 9] * self[12]);

        let tmp20 = self[1] * (self[ 6] * self[15] - self[ 7] * self[14]) - self[2] * (self[5] * self[15] - self[ 7] * self[13]) + self[3] * (self[5] * self[14] - self[ 6] * self[13]);
        let tmp21 = self[0] * (self[ 6] * self[15] - self[ 7] * self[14]) - self[2] * (self[4] * self[15] - self[ 7] * self[12]) + self[3] * (self[4] * self[14] - self[ 6] * self[12]);
        let tmp22 = self[0] * (self[ 5] * self[15] - self[ 7] * self[13]) - self[1] * (self[4] * self[15] - self[ 7] * self[12]) + self[3] * (self[4] * self[13] - self[ 5] * self[12]);
        let tmp23 = self[0] * (self[ 5] * self[14] - self[ 6] * self[13]) - self[1] * (self[4] * self[14] - self[ 6] * self[12]) + self[2] * (self[4] * self[13] - self[ 5] * self[12]);

        let tmp30 = self[1] * (self[ 6] * self[11] - self[ 7] * self[10]) - self[2] * (self[5] * self[11] - self[ 7] * self[ 9]) + self[3] * (self[5] * self[10] - self[ 6] * self[ 9]);
        let tmp31 = self[0] * (self[ 6] * self[11] - self[ 7] * self[10]) - self[2] * (self[4] * self[11] - self[ 7] * self[ 8]) + self[3] * (self[4] * self[10] - self[ 6] * self[ 8]);
        let tmp32 = self[0] * (self[ 5] * self[11] - self[ 7] * self[ 9]) - self[1] * (self[4] * self[10] - self[ 7] * self[ 8]) + self[3] * (self[4] * self[ 9] - self[ 5] * self[ 8]);
        let tmp33 = self[0] * (self[ 5] * self[10] - self[ 6] * self[ 9]) - self[1] * (self[4] * self[10] - self[ 6] * self[ 8]) + self[2] * (self[4] * self[ 9] - self[ 5] * self[ 8]);

        Self { vals: [ tmp00, -tmp10,  tmp20, -tmp30,
                      -tmp01,  tmp11, -tmp21,  tmp31,
                       tmp02, -tmp12,  tmp22, -tmp32,
                      -tmp03,  tmp13, -tmp23,  tmp33] }
    }

    /// Calculate the adjugate (transpose cofactor)
    pub fn cofactor(self) -> Self {
        let tmp00 = self[5] * (self[10] * self[15] - self[11] * self[14]) - self[6] * (self[9] * self[15] - self[11] * self[13]) + self[7] * (self[9] * self[14] - self[10] * self[13]);
        let tmp01 = self[4] * (self[10] * self[15] - self[11] * self[14]) - self[6] * (self[8] * self[15] - self[11] * self[12]) + self[7] * (self[8] * self[14] - self[10] * self[12]);
        let tmp02 = self[4] * (self[ 9] * self[15] - self[11] * self[13]) - self[5] * (self[8] * self[15] - self[11] * self[12]) + self[7] * (self[8] * self[13] - self[ 9] * self[12]);
        let tmp03 = self[4] * (self[ 9] * self[14] - self[10] * self[13]) - self[5] * (self[8] * self[14] - self[10] * self[12]) + self[6] * (self[8] * self[13] - self[ 9] * self[12]);

        let tmp10 = self[1] * (self[10] * self[15] - self[11] * self[14]) - self[2] * (self[9] * self[15] - self[11] * self[13]) + self[3] * (self[9] * self[14] - self[10] * self[13]);
        let tmp11 = self[0] * (self[10] * self[15] - self[11] * self[14]) - self[2] * (self[8] * self[15] - self[11] * self[12]) + self[3] * (self[8] * self[14] - self[10] * self[12]);
        let tmp12 = self[0] * (self[ 9] * self[15] - self[11] * self[13]) - self[1] * (self[8] * self[15] - self[11] * self[12]) + self[3] * (self[8] * self[13] - self[ 9] * self[12]);
        let tmp13 = self[0] * (self[ 9] * self[14] - self[10] * self[13]) - self[1] * (self[8] * self[14] - self[10] * self[12]) + self[2] * (self[8] * self[13] - self[ 9] * self[12]);

        let tmp20 = self[1] * (self[ 6] * self[15] - self[ 7] * self[14]) - self[2] * (self[5] * self[15] - self[ 7] * self[13]) + self[3] * (self[5] * self[14] - self[ 6] * self[13]);
        let tmp21 = self[0] * (self[ 6] * self[15] - self[ 7] * self[14]) - self[2] * (self[4] * self[15] - self[ 7] * self[12]) + self[3] * (self[4] * self[14] - self[ 6] * self[12]);
        let tmp22 = self[0] * (self[ 5] * self[15] - self[ 7] * self[13]) - self[1] * (self[4] * self[15] - self[ 7] * self[12]) + self[3] * (self[4] * self[13] - self[ 5] * self[12]);
        let tmp23 = self[0] * (self[ 5] * self[14] - self[ 6] * self[13]) - self[1] * (self[4] * self[14] - self[ 6] * self[12]) + self[2] * (self[4] * self[13] - self[ 5] * self[12]);

        let tmp30 = self[1] * (self[ 6] * self[11] - self[ 7] * self[10]) - self[2] * (self[5] * self[11] - self[ 7] * self[ 9]) + self[3] * (self[5] * self[10] - self[ 6] * self[ 9]);
        let tmp31 = self[0] * (self[ 6] * self[11] - self[ 7] * self[10]) - self[2] * (self[4] * self[11] - self[ 7] * self[ 8]) + self[3] * (self[4] * self[10] - self[ 6] * self[ 8]);
        let tmp32 = self[0] * (self[ 5] * self[11] - self[ 7] * self[ 9]) - self[1] * (self[4] * self[10] - self[ 7] * self[ 8]) + self[3] * (self[4] * self[ 9] - self[ 5] * self[ 8]);
        let tmp33 = self[0] * (self[ 5] * self[10] - self[ 6] * self[ 9]) - self[1] * (self[4] * self[10] - self[ 6] * self[ 8]) + self[2] * (self[4] * self[ 9] - self[ 5] * self[ 8]);

        Self { vals: [ tmp00, -tmp01,  tmp02, -tmp03,
                      -tmp10,  tmp11, -tmp12,  tmp13,
                       tmp20, -tmp21,  tmp22, -tmp23,
                      -tmp30,  tmp31, -tmp32,  tmp33] }
    }

    /// Calculate the inverse
    pub fn inverse(self) -> Self {
        let det = self.determinant();
        if det.is_zero() {
            Self::zero()
        } else {
            self.adjugate() * det.rcp()
        }
    }

    /// Transform a `Vec4`
    pub fn transform(self, vec: Vec4<T>) -> Vec4<T> {
        let row0 = self.row(0);
        let row1 = self.row(1);
        let row2 = self.row(2);
        let row3 = self.row(3);

        row0 * vec.x + row1 * vec.y + row2 * vec.z + row3 * vec.w
    }

    /// Transform a `Vec4`
    pub fn transform_3(self, vec: Vec3<T>) -> Vec3<T> {
        let row0 = self.row(0).shrink();
        let row1 = self.row(1).shrink();
        let row2 = self.row(2).shrink();

        row0 * vec.x + row1 * vec.y + row2 * vec.z
    }

    /// Transform a `Vec4` as a point (w-coord is forced to 1)
    pub fn transform_point(self, vec: Vec4<T>) -> Vec4<T> {
        let row0 = self.row(0);
        let row1 = self.row(1);
        let row2 = self.row(2);
        let row3 = self.row(3);

        row0 * vec.x + row1 * vec.y + row2 * vec.z + row3
    }

    /// Transform a `Vec3` as a point (implicit w-coord with a value of 1)
    pub fn transform_point_3(self, vec: Vec3<T>) -> Vec3<T> {
        let row0 = self.row(0).shrink();
        let row1 = self.row(1).shrink();
        let row2 = self.row(2).shrink();
        let row3 = self.row(3).shrink();

        row0 * vec.x + row1 * vec.y + row2 * vec.z + row3
    }

    // TODO
    /// Decompose the matrix into a scale, rotation and translation
    fn decompose(self) -> (Vec3<T>, Quat<T>, Vec3<T>) {
        let scale = Vec3 {
            x: self.column(0).len(),
            y: self.column(1).len(),
            z: self.column(2).len()
        };

        let rot_mat = Mat3 { vals: [
            self[0]  / scale.x,
            self[1]  / scale.y,
            self[2]  / scale.z,
            self[4]  / scale.x,
            self[5]  / scale.y,
            self[6]  / scale.z,
            self[8]  / scale.x,
            self[9]  / scale.y,
            self[10] / scale.z,
        ]};
        let quat = Quat::from_matrix(rot_mat);

        let trans = Vec3{ x: self[12], y: self[13], z: self[14] };

        (scale, quat, trans)
    }

    /// Decompose the 2D transformation into a scale, rotation and translation
    pub fn decompose_2d(self) -> (Vec2<T>, Radians<T>, Vec2<T>) {
        debug_assert!(self[3].is_zero(), "matrix does not represent a 2d tranfromation");
        debug_assert!(self[7].is_zero(), "matrix does not represent a 2d tranfromation");
        debug_assert!(self[14].is_zero(), "matrix does not represent a 2d tranfromation");
        debug_assert!(self[15] == T::one(), "matrix does not represent a 2d tranfromation");
        debug_assert!(self.row(2) == Vec4::new(T::zero(), T::zero(), T::one(), T::zero()), "matrix does not represent a 2d tranfromation");

        let scale = Vec2::new(self.column(0).len(), self.column(1).len());
        let angle = Radians::acos(self[0] / scale.x);
        let trans = Vec2::new(self[12], self[13]);

        (scale, angle, trans)
    }

    /// Extract the 3D scale and rotation matrix
    pub fn extract_scale_rotation(self) -> Mat3<T> {
        Mat3 { vals: [self[0], self[1], self[ 2],
                      self[4], self[5], self[ 6],
                      self[8], self[9], self[10]] }
    }

    /// Extract the 2D scale and rotation matrix
    pub fn extract_scale_rotation_2d(self) -> Mat2<T> {
        debug_assert!(self[3].is_zero(), "matrix does not represent a 2d tranfromation");
        debug_assert!(self[7].is_zero(), "matrix does not represent a 2d tranfromation");
        debug_assert!(self[14].is_zero(), "matrix does not represent a 2d tranfromation");
        debug_assert!(self[15] == T::one(), "matrix does not represent a 2d tranfromation");
        debug_assert!(self.row(2) == Vec4::new(T::zero(), T::zero(), T::one(), T::zero()), "matrix does not represent a 2d tranfromation");

        Mat2 { vals: [self[0], self[1],
                      self[4], self[5]] }
    }

    /// Create a 2d scale matrix
    pub fn create_scale_2d(scale: Vec2<T>) -> Self {
        let zero = T::zero();
        let one = T::one();
        Self { vals: [scale.x, zero   , zero, zero,
                      zero   , scale.y, zero, zero,
                      zero   , zero   , one , zero,
                      zero   , zero   , zero, one ] }
    }

    /// Create a 2d shear matrix
    pub fn create_shear_2d(shear: Vec2<T>) -> Self {
        let zero = T::zero();
        let one = T::one();
        Self { vals: [one    , shear.y, zero, zero,
                      shear.x, one    , zero, zero,
                      zero   , zero   , one , zero,
                      zero   , zero   , zero, one ] }
    }

    /// Create a 2d rotation matrix
    pub fn create_rotation_2d(angle: Radians<T>) -> Self {
        let zero = T::zero();
        let one = T::one();
        let (sin, cos) = angle.sin_cos();

        Self { vals: [cos , -sin, zero, zero,
                      sin ,  cos, zero, zero,
                      zero, zero, one , zero,
                      zero, zero, zero, one ] }
    }

    /// Create a 2d translation matrix
    pub fn create_translation_2d(trans: Vec2<T>) -> Self {
        let zero = T::zero();
        let one = T::one();

        Self { vals: [one    , zero   , zero, zero,
                      zero   , one    , zero, zero,
                      zero   , zero   , one , zero,
                      trans.x, trans.y, zero, one ] }
    }

    /// Create a 2d transformation matrix
    pub fn create_transform_2d(scale: Vec2<T>, angle: Radians<T>, trans: Vec2<T>) -> Self {
        let zero = T::zero();
        let one = T::one();
        let (sin, cos) = angle.sin_cos();

        Self { vals: [scale.x * cos, scale.y * -sin, zero, zero,
                      scale.y * sin, scale.x *  cos, zero, zero,
                      zero         , zero          , one , zero,
                      trans.x      , trans.y       , zero, one ] }
    }

    /// Create a 3d scale matrix
    pub fn create_scale(scale: Vec3<T>) -> Self {
        let zero = T::zero();
        let one = T::one();

        Self { vals: [scale.x, zero   , zero   , zero,
                      zero   , scale.y, zero   , zero,
                      zero   , zero   , scale.z, zero,
                      zero   , zero   , zero   , one ] }
    }

    // TODO
    /// Create a 3d rotation matrix
    pub fn create_rotation(rot: Quat<T>) -> Self {
        debug_assert!(rot.is_normalized());

        let xx = rot.x * rot.x;
        let yy = rot.y * rot.y;
        let zz = rot.z * rot.z;

        let xw = rot.x * rot.w;
        let yw = rot.y * rot.w;
        let zw = rot.z * rot.w;

        let xy = rot.x * rot.y;
        let xz = rot.x * rot.z;
        let yz = rot.y * rot.z;

        let zero = T::zero();
        let one = T::one();
        let two = T::from_i32(2);

        Self { vals: [one - two * (yy + zz),       two * (xy - zw),       two * (xz + yw), zero,
                            two * (xy * zw), one - two * (xx + zz),       two * (yz - zw), zero,
                            two * (xz - yw),       two * (yz + xw), one - two * (xx + yy), zero,
                      zero                 , zero                 , zero                 , one ] }
    }

    /// Create a 3d translation matrix
    pub fn create_translation(trans: Vec3<T>) -> Self {
        let zero = T::zero();
        let one = T::one();
        
        Self { vals: [one    , zero   , zero   , zero,
                      zero   , one    , zero   , zero,
                      zero   , zero   , one    , zero,
                      trans.x, trans.y, trans.z, one ] }
    }

    // TODO
    /// Create a 3d transformation matrix
    pub fn create_transform(scale: Vec3<T>, rot: Quat<T>, trans: Vec3<T>) -> Self {
        debug_assert!(rot.is_normalized());

        let xx = rot.x * rot.x;
        let yy = rot.y * rot.y;
        let zz = rot.z * rot.z;

        let xw = rot.x * rot.w;
        let yw = rot.y * rot.w;
        let zw = rot.z * rot.w;

        let xy = rot.x * rot.y;
        let xz = rot.x * rot.z;
        let yz = rot.y * rot.z;

        let zero = T::zero();
        let one = T::one();
        let two = T::from_i32(2);

        Self { vals: [scale.x * (one - two * (yy + zz)), scale.y * (      two * (xy - zw)), scale.z * (      two * (xz + yw)), zero,
                      scale.x * (      two * (xy + zw)), scale.y * (one - two * (xx + zz)), scale.z * (      two * (yz - zw)), zero,
                      scale.x * (      two * (xz - yw)), scale.y * (      two * (yz + xw)), scale.z * (one - two * (xx + yy)), zero,
                      trans.x                          , trans.y                          , trans.z                          , one ] }
    }

    // RH would need to be `eye - focus` for the z-axis
    /// Create a left-handed look-at matrix, looking from `eye` towards `focus` and a given `up` vector
    /// 
    /// This version assumes a LH coordinate system
    pub fn create_lookat(eye: Vec3<T>, focus: Vec3<T>, up: Vec3<T>) -> Self {
        let z_axis = (focus - eye).normalize();
        let x_axis = up.cross(z_axis).normalize();
        let y_axis = z_axis.cross(x_axis);
        let zero = T::zero();

        Self { vals: [ x_axis.x       ,  y_axis.x       ,  z_axis.x       , zero    ,
                       x_axis.y       ,  y_axis.y       ,  z_axis.y       , zero    ,
                       x_axis.z       ,  y_axis.z       ,  z_axis.z       , zero    ,
                      -x_axis.dot(eye), -y_axis.dot(eye), -z_axis.dot(eye), T::one()] }
    }

    // RH would need to be `z_axis.cross(up)` for the x-axis
    /// Create a left-handed look-to matrix, looking from `eye` in `look_dir` and a given `up` vector
    /// 
    /// This version assumes a LH coordinate system
    pub fn create_lookto(eye: Vec3<T>, look_dir: Vec3<T>, up: Vec3<T>) -> Self {
        let z_axis = look_dir.normalize();
        let x_axis = up.cross(z_axis).normalize();
        let y_axis = z_axis.cross(x_axis);
        let zero = T::zero();

        Self { vals: [ x_axis.x       ,  x_axis.y       ,  x_axis.z       , zero    ,
                       y_axis.x       ,  y_axis.y       ,  y_axis.z       , zero    ,
                       z_axis.x       ,  z_axis.y       ,  z_axis.z       , zero    ,
                      -x_axis.dot(eye), -y_axis.dot(eye), -z_axis.dot(eye), T::one()] }
    }

    // RH would need to be `near - far` for f_range, and `near * f_range` for m32
    /// Create an othrographic projection matrix (isometric)
    /// 
    /// This version assumes a LH coordinate system with a z depth in the range (0; 1)
    pub fn create_ortho(width: T, height: T, near: T, far: T) -> Self {
        debug_assert!(width > T::zero());
        debug_assert!(height > T::zero());
        debug_assert!(near < far);

        let zero = T::zero();
        let one = T::one();
        let two = T::from_f32(2f32);

        let f_range = one / (far - near);

        Self { vals: [two / width, zero        ,  zero          , zero,
                      zero       , two / height,  zero          , zero,
                      zero       , zero        ,  f_range       , zero,
                      zero       , zero        , -near * f_range, one ] }
    }

    // RH would need to be `near - far` for f_range, and `near * f_range` for m32
    /// Create an othrographic projection matrix (isometric), which can be offset from the center of the screen
    /// 
    /// This version assumes a LH coordinate system with a z depth in the range (0; 1)
    pub fn create_ortho_offset(left: T, right: T, top: T, bottom: T, near: T, far: T) -> Self {
        debug_assert!(left < right);
        debug_assert!(bottom < top);
        debug_assert!(far < near);

        let zero = T::zero();
        let one = T::one();
        let two = T::from_f32(2f32);

        let rcp_width = (right - left).rcp();
        let rcp_height = (top - bottom).rcp();
        let f_range = one / (far - near);

        Self { vals: [ two * rcp_width           ,  zero                       ,  zero          , zero,
                       zero                      ,  two * rcp_height           ,  zero          , zero,
                       zero                      ,  zero                       ,  f_range       , zero,
                      -(right + left) * rcp_width, -(top + bottom) * rcp_height, -near * f_range, one ] }
    }

    // RH would need to be `near - far` for f_range, `-1` for m23, and `near * f_range` for m32
    /// Create a perspective matrix, with the `width` and `height` given as the size of the frustum at the `near` plane
    /// 
    /// This version assumes a LH coordinate system with a z depth in the range (0; 1)
    pub fn create_perspective(width: T, height: T, near: T, far: T) -> Self {
        debug_assert!(width > T::zero());
        debug_assert!(height > T::zero());
        debug_assert!(near < far);
        debug_assert!(near > T::zero());

        let zero = T::zero();
        let one = T::one();
        let two_near = T::from_f32(2f32) * near;
        let f_range = far / (far - near);

        Self { vals: [two_near / width, zero             , zero           , zero,
                      zero            , two_near / height, zero           , zero,
                      zero            , zero             , f_range        , one ,
                      zero            , zero             , -near * f_range, zero] }
    }

    // RH would need to be `near - far` for m22
    /// Create a perspective matrix, with the `left`, `right`, `top` and `bottom` defining the size of the frustum at the `near` plane
    /// 
    /// This version assumes a LH coordinate system with a z depth in the range (0; 1)
    pub fn create_perspective_offset(left: T, right: T, top: T, bottom: T, near: T, far: T) -> Self {
        debug_assert!(left < right);
        debug_assert!(bottom < top);
        debug_assert!(near < far);
        debug_assert!(near > T::zero());

        let zero = T::zero();
        let one = T::one();

        let two_near = T::from_f32(2f32) * near;
        let f_range = far / (far - near);
        let rcp_width = (right - left).rcp();
        let rcp_height = (top - bottom).rcp();

        Self { vals: [ two_near * rcp_width      ,  zero                       , zero           , zero,
                       zero                      ,  two_near * rcp_height      , zero           , zero,
                      -(left - right) * rcp_width, -(top - bottom) * rcp_height, f_range        , one ,
                       zero                      ,  zero                       , -near * f_range, zero] }
    }

    // RH would need to be `near - far` for f_range, `-1` for m23, and `near * f_range` for m32
    /// Create a perspective matrix, with a given vertical `fov` and an `aspect` ratio defined as `height / width`
    /// 
    /// This version assumes a LH coordinate system with a z depth in the range (0; 1)
    pub fn create_perspective_fov(fov: Radians<T>, aspect: T, near: T, far: T) -> Self {
        debug_assert!(fov > Radians::zero());
        debug_assert!(aspect > T::zero());
        debug_assert!(near < far);
        debug_assert!(near > T::zero());

        let zero = T::zero();
        let one = T::one();

        let height = (fov / T::from_f32(2f32)).tan().rcp();
        let width = height * aspect;
        let f_range = far / (far - near);

        Self { vals: [width, zero  , zero           , zero,
                      zero , height, zero           , zero,
                      zero , zero  , f_range        , one ,
                      zero , zero  , -near * f_range, zero] }
    }
}

impl<T: Real> Mul for Mat4<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let row0 = self.row(0);
        let row1 = self.row(1);
        let row2 = self.row(2);
        let row3 = self.row(3);

        let column0 = rhs.column(0);
        let column1 = rhs.column(1);
        let column2 = rhs.column(2);
        let column3 = rhs.column(3);

        Self { vals : [row0.dot(column0), row0.dot(column1), row0.dot(column2), row0.dot(column3),
                       row1.dot(column0), row1.dot(column1), row1.dot(column2), row1.dot(column3),
                       row2.dot(column0), row2.dot(column1), row2.dot(column2), row2.dot(column3),
                       row3.dot(column0), row3.dot(column1), row3.dot(column2), row3.dot(column3)] }
    }
}

impl<T: Real> MulAssign for Mat4<T> {
    fn mul_assign(&mut self, rhs: Self) {
        let row0 = self.row(0);
        let row1 = self.row(1);
        let row2 = self.row(2);
        let row3 = self.row(3);

        let column0 = rhs.column(0);
        let column1 = rhs.column(1);
        let column2 = rhs.column(2);
        let column3 = rhs.column(3);

        self[ 0] = row0.dot(column0); 
        self[ 1] = row0.dot(column1); 
        self[ 2] = row0.dot(column2); 
        self[ 3] = row0.dot(column3);
        self[ 4] = row1.dot(column0); 
        self[ 5] = row1.dot(column1); 
        self[ 6] = row1.dot(column2); 
        self[ 7] = row1.dot(column3);
        self[ 8] = row2.dot(column0); 
        self[ 9] = row2.dot(column1); 
        self[10] = row2.dot(column2); 
        self[11] = row2.dot(column3);
        self[12] = row3.dot(column0); 
        self[13] = row3.dot(column1); 
        self[14] = row3.dot(column2); 
        self[15] = row3.dot(column3);
    }
}

impl<T: Real + Display> Display for Mat4<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("[[{}, {}, {}, {}], [{}, {}, {}, {}], [{}, {}, {}, {}], [{}, {}, {}, {}]]",
                    self[ 0], self[ 1], self[ 2], self[ 3],
                    self[ 4], self[ 5], self[ 6], self[ 7],
                    self[ 8], self[ 9], self[10], self[11],
                    self[12], self[13], self[14], self[15]))
    }
}

#[allow(non_camel_case_types)] type f32m4 = Mat4<f32>;
#[allow(non_camel_case_types)] type f64m4 = Mat4<f64>;