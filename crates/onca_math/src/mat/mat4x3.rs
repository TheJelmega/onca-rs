use std::{ops::*, fmt::Display};
use crate::{*, angle::Radians};

generic_matrix!{doc = "4x3 matrix (row-major order), with an implicit (0, 0, 0, 1) column at the end"; Mat4x3, 4, 3}

impl<T: Real> Mat4x3<T> {
    /// Create a new matrix with the given values
    #[inline]
    #[must_use]
    pub fn new(m00: T, m01: T, m02: T, m10: T, m11: T, m12: T, m20: T, m21: T, m22: T, m30: T, m31: T, m32: T) -> Self {
        Self { vals: [m00, m01, m02, 
                      m10, m11, m12, 
                      m20, m21, m22, 
                      m30, m31, m32] }
    }

    /// Create a new matrix with the given rows
    #[inline]
    #[must_use]
    pub fn from_rows(row0: Vec3<T>, row1: Vec3<T>, row2: Vec3<T>, row3: Vec3<T>) -> Self {
        Self { vals: [row0.x, row0.y, row0.z, 
                      row1.x, row1.y, row1.z, 
                      row2.x, row2.y, row2.z, 
                      row3.x, row3.y, row3.z] }
    }

    /// Create a new matrix with the given columsn
    #[inline]
    #[must_use]
    pub fn from_columns(column0: Vec4<T>, column1: Vec4<T>, column2: Vec4<T>) -> Self {
        Self { vals: [column0.x, column1.x, column2.x,
                      column0.y, column1.y, column2.y,
                      column0.z, column1.z, column2.z,
                      column0.w, column1.w, column2.w] }
    }

    /// Get the row at the given index
    #[inline]
    #[must_use]
    pub fn row(self, index: usize) -> Vec4<T> {
        debug_assert!(index < 4);
        let idx = index * 3;
        Vec4::new(self.vals[idx], self.vals[idx + 1], self.vals[idx + 2], self.vals[idx + 3])
    }

    /// Set the row at the given index
    #[inline]
    pub fn set_row(&mut self, index: usize, row: Vec4<T>) {
        debug_assert!(index < 4);
        let idx = index * 3;
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
        debug_assert!(index < 3);
        self.vals[index + 0] = column.x;
        self.vals[index + 3] = column.y;
        self.vals[index + 6] = column.z;
        self.vals[index + 9] = column.w;
    }

    /// Get the diagonal (implicit w-coord set to 1)
    #[inline]
    #[must_use]
    pub fn diagonal(self) -> Vec4<T> {
        Vec4 { x: self.vals[0], y: self.vals[4], z: self.vals[8], w: T::one() }
    }

    /// Set the diagonal (w-coordinate will be ignored)
    #[inline]
    pub fn set_diagonal(&mut self, diag: Vec4<T>) {
        self.vals[0] = diag.x;
        self.vals[4] = diag.y;
        self.vals[8] = diag.z;
    }

    /// Get the identity matrix
    #[inline]
    pub fn identity() -> Self {
        let zero = T::zero();
        let one = T::one();

        Self { vals: [one , zero, zero,
                      zero, one , zero,
                      zero, zero, one ,
                      zero, zero, zero] }
    }

    /// Get the minor matrix for the given `row` and `column`
    #[inline]
    #[must_use]
    pub fn minor(self, row: usize, column: usize) -> Mat3<T> {
        debug_assert!(row < 4);
        debug_assert!(column < 3);

        let zero = T::zero();
        let one = T::one();

        match row {
            0 => match column {
                0 => Mat3{ vals: [self[4], self[5], zero   , self[7], self[8], zero   , self[10], self[11], one     ]},
                1 => Mat3{ vals: [self[3], self[5], zero   , self[6], self[8], zero   , self[ 9], self[11], one     ]},
                2 => Mat3{ vals: [self[3], self[4], zero   , self[6], self[7], self[8], self[ 9], self[10], one     ]},
                3 => Mat3{ vals: [self[3], self[4], self[5], self[6], self[7], self[8], self[ 9], self[10], self[11]]},
                _ => unreachable!()
            },
            1 => match column {
                0 => Mat3{ vals: [self[1], self[2], zero   , self[7], self[8], zero   , self[10], self[11], one     ]},
                1 => Mat3{ vals: [self[0], self[2], zero   , self[6], self[8], zero   , self[ 9], self[11], one     ]},
                2 => Mat3{ vals: [self[0], self[1], zero   , self[6], self[7], self[8], self[ 9], self[10], one     ]},
                3 => Mat3{ vals: [self[0], self[1], self[2], self[6], self[7], self[8], self[ 9], self[10], self[11]]},
                _ => unreachable!()
            },
            2 => match column {
                0 => Mat3{ vals: [self[1], self[2], zero   , self[4], self[5], zero   , self[10], self[11], one     ]},
                1 => Mat3{ vals: [self[0], self[2], zero   , self[3], self[5], zero   , self[ 9], self[11], one     ]},
                2 => Mat3{ vals: [self[0], self[1], zero   , self[3], self[4], zero   , self[ 9], self[10], one     ]},
                3 => Mat3{ vals: [self[0], self[1], self[2], self[3], self[4], self[5], self[ 9], self[10], self[11]]},
                _ => unreachable!()
            },
            3 => match column {
                0 => Mat3{ vals: [self[1], self[2], zero   , self[4], self[5], zero   , self[7] , self[8], zero   ]},
                1 => Mat3{ vals: [self[0], self[2], zero   , self[3], self[5], zero   , self[6] , self[8], zero   ]},
                2 => Mat3{ vals: [self[0], self[1], zero   , self[3], self[4], zero   , self[6] , self[7], self[8]]},
                3 => Mat3{ vals: [self[0], self[1], self[2], self[3], self[4], self[5], self[6] , self[7], self[8]]},
                _ => unreachable!()
            },
            _ => unreachable!()
        }
    }

    /// Calculate the determinant
    pub fn determinant(self) -> T {
        let minor0_det = self[4] * self[8] - self[5] * self[7];
        let minor1_det = self[3] * self[8] - self[5] * self[6];
        let minor2_det = self[3] * self[7] - self[4] * self[6];

        self[0] * minor0_det - self[1] * minor1_det + self[2] * minor2_det
    }

    /// Calculate the trace
    #[inline]
    pub fn trace(self) -> T {
        self[0] + self[4] + self[8] + T::one()
    }

    /// Transpose the matrix
    #[inline]
    pub fn transpose(self) -> Self {
        let zero = T::zero();

        Self { vals: [self[0], self[3], self[6],
                      self[1], self[4], self[7],
                      self[2], self[5], self[8],
                      zero   , zero   , zero   ] }
    }

    /// Calculate the adjugate
    pub fn adjugate(self) -> Self {
        let tmp00 = self[4] * self[8] - self[5] * self[7];
        let tmp01 = self[3] * self[8] - self[5] * self[6];
        let tmp02 = self[3] * self[7] - self[4] * self[6];
        let tmp03 = self[3] * (self[7] * self[11] - self[8] * self[10]) - self[4] * (self[6] * self[11] - self[8] * self[9]) + self[5] * (self[6] * self[10] - self[7] * self[9]);

        let tmp10 = self[1] * self[8] - self[2] * self[7];
        let tmp11 = self[0] * self[8] - self[2] * self[6];
        let tmp12 = self[0] * self[7] - self[1] * self[6];
        let tmp13 = self[0] * (self[7] * self[11] - self[8] * self[10]) - self[1] * (self[6] * self[11] - self[8] * self[9]) + self[2] * (self[6] * self[10] - self[7] * self[9]);

        let tmp20 = self[1] * self[5] - self[2] * self[4];
        let tmp21 = self[0] * self[5] - self[2] * self[3];
        let tmp22 = self[0] * self[4] - self[1] * self[3];
        let tmp23 = self[0] * (self[4] * self[11] - self[5] * self[10]) - self[1] * (self[3] * self[11] - self[5] * self[9]) + self[2] * (self[3] * self[10] - self[4] * self[9]);

        Self { vals: [ tmp00, -tmp10,  tmp20,
                      -tmp01,  tmp11, -tmp21,
                       tmp02, -tmp12,  tmp22,
                      -tmp03,  tmp13, -tmp23] }
    }

    /// Calculate the adjugate (transpose cofactor)
    pub fn cofactor(self) -> Self {
        let tmp00 = self[4] * self[8] - self[5] * self[7];
        let tmp01 = self[3] * self[8] - self[5] * self[6];
        let tmp02 = self[3] * self[7] - self[4] * self[6];

        let tmp10 = self[1] * self[8] - self[2] * self[7];
        let tmp11 = self[0] * self[8] - self[2] * self[6];
        let tmp12 = self[0] * self[7] - self[1] * self[6];

        let tmp20 = self[1] * self[5] - self[2] * self[4];
        let tmp21 = self[0] * self[5] - self[2] * self[3];
        let tmp22 = self[0] * self[4] - self[1] * self[3];

        let tmp30 = self[1] * self[5];
        let tmp31 = self[0] * self[5];
        let tmp32 = self[0] * self[4] - self[1] * self[3] * self[8];

        Self { vals: [ tmp00, -tmp01,  tmp02,
                      -tmp10,  tmp11, -tmp12,
                       tmp20, -tmp21,  tmp22,
                      -tmp30,  tmp31, -tmp32] }
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

    /// Decompose the matrix into a scale, rotation and translation
    fn decompose(self) -> (Vec3<T>, Quat<T>, Vec3<T>) {
        let scale = Vec3 {
            x: self.column(0).len(),
            y: self.column(1).len(),
            z: self.column(2).len()
        };

        let rot_mat = Mat3 { vals: [
            self[0] / scale.x,
            self[1] / scale.y,
            self[2] / scale.z,
            self[3] / scale.x,
            self[4] / scale.y,
            self[5] / scale.z,
            self[6] / scale.x,
            self[7] / scale.y,
            self[8] / scale.z,
        ]};
        let quat = Quat::from_matrix(rot_mat);

        let trans = Vec3{ x: self[12], y: self[13], z: self[14] };

        (scale, quat, trans)
    }

    /// Decompose the 2D transformation into a scale, rotation and translation
    pub fn decompose_2d(self) -> (Vec2<T>, Radians<T>, Vec2<T>) where
        Radians<T>: InvTrig<T>
    {
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
        Self { vals: [scale.x, zero   , zero,
                      zero   , scale.y, zero,
                      zero   , zero   , one ,
                      zero   , zero   , zero] }
    }

    /// Create a 2d shear matrix
    pub fn create_shear_2d(shear: Vec2<T>) -> Self {
        let zero = T::zero();
        let one = T::one();
        Self { vals: [one    , shear.y, zero,
                      shear.x, one    , zero,
                      zero   , zero   , one ,
                      zero   , zero   , zero] }
    }

    /// Create a 2d rotation matrix
    pub fn create_rotation_2d(angle: Radians<T>) -> Self where
        Radians<T>: Trig<Output = T>
    {
        let zero = T::zero();
        let one = T::one();
        let (sin, cos) = angle.sin_cos();

        Self { vals: [cos , -sin, zero,
                      sin ,  cos, zero,
                      zero, zero, one ,
                      zero, zero, zero] }
    }

    /// Create a 2d translation matrix
    pub fn create_translation_2d(trans: Vec2<T>) -> Self {
        let zero = T::zero();
        let one = T::one();

        Self { vals: [one    , zero   , zero,
                      zero   , one    , zero,
                      zero   , zero   , one ,
                      trans.x, trans.y, zero] }
    }

    /// Create a 2d transformation matrix
    pub fn create_transform_2d(scale: Vec2<T>, angle: Radians<T>, trans: Vec2<T>) -> Self where
        Radians<T>: Trig<Output = T>
    {
        let zero = T::zero();
        let one = T::one();
        let (sin, cos) = angle.sin_cos();

        Self { vals: [scale.x * cos, scale.y * -sin, zero,
                      scale.y * sin, scale.x *  cos, zero,
                      zero         , zero          , one ,
                      trans.x      , trans.y       , zero] }
    }

    /// Create a 3d scale matrix
    pub fn create_scale(scale: Vec3<T>) -> Self {
        let zero = T::zero();

        Self { vals: [scale.x, zero   , zero   ,
                      zero   , scale.y, zero   ,
                      zero   , zero   , scale.z,
                      zero   , zero   , zero   ] }
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
        let two: T = T::from_i32(2);

        Self { vals: [(one - two * (yy + zz)), (      two * (xy - zw)), (      two * (xz + yw)),
                      (      two * (xy + zw)), (one - two * (xx + zz)), (      two * (yz - zw)),
                      (      two * (xz - yw)), (      two * (yz + xw)), (one - two * (xx + yy)),
                      zero                   , zero                   , zero                   ] }
    }

    /// Create a 3d translation matrix
    pub fn create_translation(trans: Vec3<T>) -> Self {
        let zero = T::zero();
        let one = T::one();
        
        Self { vals: [one    , zero   , zero   ,
                      zero   , one    , zero   ,
                      zero   , zero   , one    ,
                      trans.x, trans.y, trans.z] }
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

        let one = T::one();
        let two: T = T::from_i32(2);

        Self { vals: [scale.x * (one - two * (yy + zz)), scale.y * (      two * (xy - zw)), scale.z * (      two * (xz + yw)),
                      scale.x * (      two * (xy + zw)), scale.y * (one - two * (xx + zz)), scale.z * (      two * (yz - zw)),
                      scale.x * (      two * (xz - yw)), scale.y * (      two * (yz + xw)), scale.z * (one - two * (xx + yy)),
                      trans.x                          , trans.y                          , trans.z                          ] }
    }
}

impl<T: Real> Mul for Mat4x3<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let row0 = self.row(0);
        let row1 = self.row(1);
        let row2 = self.row(2);
        let row3 = self.row(3);

        let column0 = rhs.column(0);
        let column1 = rhs.column(1);
        let column2 = rhs.column(2);

        Self { vals : [row0.dot(column0), row0.dot(column1), row0.dot(column2),
                       row1.dot(column0), row1.dot(column1), row1.dot(column2),
                       row2.dot(column0), row2.dot(column1), row2.dot(column2),
                       row3.dot(column0), row3.dot(column1), row3.dot(column2)] }
    }
}

impl<T: Real> MulAssign for Mat4x3<T> {
    fn mul_assign(&mut self, rhs: Self) {
        let row0 = self.row(0);
        let row1 = self.row(1);
        let row2 = self.row(2);
        let row3 = self.row(3);

        let column0 = rhs.column(0);
        let column1 = rhs.column(1);
        let column2 = rhs.column(2);

        self[ 0] = row0.dot(column0); 
        self[ 1] = row0.dot(column1); 
        self[ 2] = row0.dot(column2); 
        self[ 3] = row1.dot(column0); 
        self[ 4] = row1.dot(column1); 
        self[ 5] = row1.dot(column2); 
        self[ 6] = row2.dot(column0); 
        self[ 7] = row2.dot(column1); 
        self[ 8] = row2.dot(column2); 
        self[ 9] = row3.dot(column0); 
        self[10] = row3.dot(column1); 
        self[11] = row3.dot(column2); 
    }
}

impl<T: Real + Display> Display for Mat4x3<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("[[{}, {}, {}], [{}, {}, {}], [{}, {}, {}], [{}, {}, {}]]",
                    self[0], self[ 1], self[ 2],
                    self[3], self[ 4], self[ 5],
                    self[6], self[ 7], self[ 8],
                    self[9], self[10], self[11]))
    }
}

#[allow(non_camel_case_types)] type f32m4x3 = Mat4x3<f32>;
#[allow(non_camel_case_types)] type f64m4x3 = Mat4x3<f64>;