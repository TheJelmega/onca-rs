use std::{ops::{Mul, MulAssign}, fmt::Display};
use crate::*;

impl<T: Numeric> Vec4<T> {
    /// Shrink a `Vec4` to a `Vec3`
    #[inline]
    #[must_use]
    pub fn shrink(self) -> Vec3<T> {
        Vec3 { x: self.x, y: self.y, z: self.z }
    }

    /// Check if all elements are approximately equal, given an epsilon
    pub fn is_uniform(self, epsilon: T) -> bool {
        self.x.abs_diff(self.y) <= epsilon &&
        self.x.abs_diff(self.z) <= epsilon &&
        self.x.abs_diff(self.w) <= epsilon 
    }

    /// Check if the 4d vector represents a 3d vector (z-coord == 0)
    pub fn represents_3d_vector(self) -> bool {
        self.x == T::zero()
    }

    /// Check if the 4d vector represents a 3d point (z-coord == 1)
    pub fn represents_3d_point(self) -> bool {
        self.x == T::one()
    }

    /// Get the minimum component of the vector
    pub fn min_component(self) -> T {
        self.x.min(self.y)
              .min(self.z)
              .min(self.w)
    }

    /// Get the minimum absolute component of the vector
    pub fn min_abs_component(self) -> T {
        self.x.abs().min(self.y.abs())
                    .min(self.z.abs())
                    .min(self.w.abs())
    }

    /// Get the maximum component of the vector
    pub fn max_component(self) -> T {
        self.x.max(self.y)
              .max(self.z)
              .max(self.w)
    }

    /// Get the maximum absolute component of the vector
    pub fn max_abs_component(self) -> T {
        self.x.abs().max(self.y.abs())
                    .max(self.z.abs())
                    .max(self.w.abs())
    }
}

impl<T: Real> Vec4<T> {
    /// Transform the vector by a matrix
    #[inline(always)]
    pub fn transform(self, mat: Mat4<T>) -> Self {
        mat.transform(self)
    }
}

impl<T: Real> Mul<Mat4<T>> for Vec4<T> {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Mat4<T>) -> Self::Output {
        self.transform(rhs)
    }
}

impl<T: Real> MulAssign<Mat4<T>> for Vec4<T> {
    #[inline]
    fn mul_assign(&mut self, rhs: Mat4<T>) {
        *self = self.transform(rhs)
    }
}

impl<T: Numeric + Display> Display for Vec4<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("({}, {}, {}, {})", self.x, self.y, self.z, self.w))
    }
}

// Swizzles
impl<T: Numeric> Vec4<T> {
    const X : u8 = 0;
    const Y : u8 = 1;
    const Z : u8 = 2;
    const W : u8 = 3;

    /// Swizzle the components of the vector into a `Vec2`
    pub fn swizzle2(self, x: u8, y: u8) -> Vec2<T> {
        debug_assert!(x < 4);
        debug_assert!(y < 4);
        Vec2 { x: self[x as usize], y: self[y as usize] }
    }

    /// Swizzle the components of the vector into a `Vec3`
    pub fn swizzle3(self, x: u8, y: u8, z: u8) -> Vec3<T> {
        debug_assert!(x < 4);
        debug_assert!(y < 4);
        debug_assert!(z < 4);
        Vec3 { x: self[x as usize], y: self[y as usize], z: self[z as usize] }
    }

    /// Swizzle the components of the vector
    pub fn swizzle(self, x: u8, y: u8, z: u8, w: u8) -> Self {
        debug_assert!(x < 4);
        debug_assert!(y < 4);
        debug_assert!(z < 4);
        debug_assert!(w < 4);
        Self { x: self[x as usize], y: self[y as usize], z: self[z as usize], w: self[w as usize] }
    }

    create_swizzle!{@2d xx, x, x}
    create_swizzle!{@2d xy, x, y}
    create_swizzle!{@2d xz, x, z}
    create_swizzle!{@2d xw, x, w}
    create_swizzle!{@2d yx, y, x}
    create_swizzle!{@2d yy, y, y}
    create_swizzle!{@2d yz, y, z}
    create_swizzle!{@2d yw, y, w}
    create_swizzle!{@2d zx, z, x}
    create_swizzle!{@2d zy, z, y}
    create_swizzle!{@2d zz, z, z}
    create_swizzle!{@2d zw, z, w}
    create_swizzle!{@2d wx, w, x}
    create_swizzle!{@2d wy, w, y}
    create_swizzle!{@2d wz, w, z}
    create_swizzle!{@2d ww, w, w}

    create_swizzle!{@3d xxx, x, x, x}
    create_swizzle!{@3d xxy, x, x, y}
    create_swizzle!{@3d xxz, x, x, z}
    create_swizzle!{@3d xxw, x, x, w}
    create_swizzle!{@3d xyx, x, y, x}
    create_swizzle!{@3d xyy, x, y, y}
    create_swizzle!{@3d xyz, x, y, z}
    create_swizzle!{@3d xyw, x, y, w}
    create_swizzle!{@3d xzx, x, z, x}
    create_swizzle!{@3d xzy, x, z, y}
    create_swizzle!{@3d xzz, x, z, z}
    create_swizzle!{@3d xzw, x, z, w}
    create_swizzle!{@3d xwx, x, w, x}
    create_swizzle!{@3d xwy, x, w, y}
    create_swizzle!{@3d xwz, x, w, z}
    create_swizzle!{@3d xww, x, w, w}
    create_swizzle!{@3d yxx, y, x, x}
    create_swizzle!{@3d yxy, y, x, y}
    create_swizzle!{@3d yxz, y, x, z}
    create_swizzle!{@3d yxw, y, x, w}
    create_swizzle!{@3d yyx, y, y, x}
    create_swizzle!{@3d yyy, y, y, y}
    create_swizzle!{@3d yyz, y, y, z}
    create_swizzle!{@3d yyw, y, y, w}
    create_swizzle!{@3d yzx, y, z, x}
    create_swizzle!{@3d yzy, y, z, y}
    create_swizzle!{@3d yzz, y, z, z}
    create_swizzle!{@3d yzw, y, z, w}
    create_swizzle!{@3d ywx, y, w, x}
    create_swizzle!{@3d ywy, y, w, y}
    create_swizzle!{@3d ywz, y, w, z}
    create_swizzle!{@3d yww, y, w, w}
    create_swizzle!{@3d zxx, z, x, x}
    create_swizzle!{@3d zxy, z, x, y}
    create_swizzle!{@3d zxz, z, x, z}
    create_swizzle!{@3d zxw, z, x, w}
    create_swizzle!{@3d zyx, z, y, x}
    create_swizzle!{@3d zyy, z, y, y}
    create_swizzle!{@3d zyz, z, y, z}
    create_swizzle!{@3d zyw, z, y, w}
    create_swizzle!{@3d zzx, z, z, x}
    create_swizzle!{@3d zzy, z, z, y}
    create_swizzle!{@3d zzz, z, z, z}
    create_swizzle!{@3d zzw, z, z, w}
    create_swizzle!{@3d zwx, z, w, x}
    create_swizzle!{@3d zwy, z, w, y}
    create_swizzle!{@3d zwz, z, w, z}
    create_swizzle!{@3d zww, z, w, w}
    create_swizzle!{@3d wxx, w, x, x}
    create_swizzle!{@3d wxy, w, x, y}
    create_swizzle!{@3d wxz, w, x, z}
    create_swizzle!{@3d wxw, w, x, w}
    create_swizzle!{@3d wyx, w, y, x}
    create_swizzle!{@3d wyy, w, y, y}
    create_swizzle!{@3d wyz, w, y, z}
    create_swizzle!{@3d wyw, w, y, w}
    create_swizzle!{@3d wzx, w, z, x}
    create_swizzle!{@3d wzy, w, z, y}
    create_swizzle!{@3d wzz, w, z, z}
    create_swizzle!{@3d wzw, w, z, w}
    create_swizzle!{@3d wwx, w, w, x}
    create_swizzle!{@3d wwy, w, w, y}
    create_swizzle!{@3d wwz, w, w, z}
    create_swizzle!{@3d www, w, w, w}

    create_swizzle!{@4d xxxx, x, x, x, x}
    create_swizzle!{@4d xxxy, x, x, x, y}
    create_swizzle!{@4d xxxz, x, x, x, z}
    create_swizzle!{@4d xxxw, x, x, x, w}
    create_swizzle!{@4d xxyx, x, x, y, x}
    create_swizzle!{@4d xxyy, x, x, y, y}
    create_swizzle!{@4d xxyz, x, x, y, z}
    create_swizzle!{@4d xxyw, x, x, y, w}
    create_swizzle!{@4d xxzx, x, x, z, x}
    create_swizzle!{@4d xxzy, x, x, z, y}
    create_swizzle!{@4d xxzz, x, x, z, z}
    create_swizzle!{@4d xxzw, x, x, z, w}
    create_swizzle!{@4d xxwx, x, x, w, x}
    create_swizzle!{@4d xxwy, x, x, w, y}
    create_swizzle!{@4d xxwz, x, x, w, z}
    create_swizzle!{@4d xxww, x, x, w, w}
    create_swizzle!{@4d xyxx, x, y, x, x}
    create_swizzle!{@4d xyxy, x, y, x, y}
    create_swizzle!{@4d xyxz, x, y, x, z}
    create_swizzle!{@4d xyxw, x, y, x, w}
    create_swizzle!{@4d xyyx, x, y, y, x}
    create_swizzle!{@4d xyyy, x, y, y, y}
    create_swizzle!{@4d xyyz, x, y, y, z}
    create_swizzle!{@4d xyyw, x, y, y, w}
    create_swizzle!{@4d xyzx, x, y, z, x}
    create_swizzle!{@4d xyzy, x, y, z, y}
    create_swizzle!{@4d xyzz, x, y, z, z}
    create_swizzle!{@4d xyzw, x, y, z, w}
    create_swizzle!{@4d xywx, x, y, w, x}
    create_swizzle!{@4d xywy, x, y, w, y}
    create_swizzle!{@4d xywz, x, y, w, z}
    create_swizzle!{@4d xyww, x, y, w, w}
    create_swizzle!{@4d xzxx, x, z, x, x}
    create_swizzle!{@4d xzxy, x, z, x, y}
    create_swizzle!{@4d xzxz, x, z, x, z}
    create_swizzle!{@4d xzxw, x, z, x, w}
    create_swizzle!{@4d xzyx, x, z, y, x}
    create_swizzle!{@4d xzyy, x, z, y, y}
    create_swizzle!{@4d xzyz, x, z, y, z}
    create_swizzle!{@4d xzyw, x, z, y, w}
    create_swizzle!{@4d xzzx, x, z, z, x}
    create_swizzle!{@4d xzzy, x, z, z, y}
    create_swizzle!{@4d xzzz, x, z, z, z}
    create_swizzle!{@4d xzzw, x, z, z, w}
    create_swizzle!{@4d xzwx, x, z, w, x}
    create_swizzle!{@4d xzwy, x, z, w, y}
    create_swizzle!{@4d xzwz, x, z, w, z}
    create_swizzle!{@4d xzww, x, z, w, w}
    create_swizzle!{@4d xwxx, x, w, x, x}
    create_swizzle!{@4d xwxy, x, w, x, y}
    create_swizzle!{@4d xwxz, x, w, x, z}
    create_swizzle!{@4d xwxw, x, w, x, w}
    create_swizzle!{@4d xwyx, x, w, y, x}
    create_swizzle!{@4d xwyy, x, w, y, y}
    create_swizzle!{@4d xwyz, x, w, y, z}
    create_swizzle!{@4d xwyw, x, w, y, w}
    create_swizzle!{@4d xwzx, x, w, z, x}
    create_swizzle!{@4d xwzy, x, w, z, y}
    create_swizzle!{@4d xwzz, x, w, z, z}
    create_swizzle!{@4d xwzw, x, w, z, w}
    create_swizzle!{@4d xwwx, x, w, w, x}
    create_swizzle!{@4d xwwy, x, w, w, y}
    create_swizzle!{@4d xwwz, x, w, w, z}
    create_swizzle!{@4d xwww, x, w, w, w}

    create_swizzle!{@4d yxxx, y, x, x, x}
    create_swizzle!{@4d yxxy, y, x, x, y}
    create_swizzle!{@4d yxxz, y, x, x, z}
    create_swizzle!{@4d yxxw, y, x, x, w}
    create_swizzle!{@4d yxyx, y, x, y, x}
    create_swizzle!{@4d yxyy, y, x, y, y}
    create_swizzle!{@4d yxyz, y, x, y, z}
    create_swizzle!{@4d yxyw, y, x, y, w}
    create_swizzle!{@4d yxzx, y, x, z, x}
    create_swizzle!{@4d yxzy, y, x, z, y}
    create_swizzle!{@4d yxzz, y, x, z, z}
    create_swizzle!{@4d yxzw, y, x, z, w}
    create_swizzle!{@4d yxwx, y, x, w, x}
    create_swizzle!{@4d yxwy, y, x, w, y}
    create_swizzle!{@4d yxwz, y, x, w, z}
    create_swizzle!{@4d yxww, y, x, w, w}
    create_swizzle!{@4d yyxx, y, y, x, x}
    create_swizzle!{@4d yyxy, y, y, x, y}
    create_swizzle!{@4d yyxz, y, y, x, z}
    create_swizzle!{@4d yyxw, y, y, x, w}
    create_swizzle!{@4d yyyx, y, y, y, x}
    create_swizzle!{@4d yyyy, y, y, y, y}
    create_swizzle!{@4d yyyz, y, y, y, z}
    create_swizzle!{@4d yyyw, y, y, y, w}
    create_swizzle!{@4d yyzx, y, y, z, x}
    create_swizzle!{@4d yyzy, y, y, z, y}
    create_swizzle!{@4d yyzz, y, y, z, z}
    create_swizzle!{@4d yyzw, y, y, z, w}
    create_swizzle!{@4d yywx, y, y, w, x}
    create_swizzle!{@4d yywy, y, y, w, y}
    create_swizzle!{@4d yywz, y, y, w, z}
    create_swizzle!{@4d yyww, y, y, w, w}
    create_swizzle!{@4d yzxx, y, z, x, x}
    create_swizzle!{@4d yzxy, y, z, x, y}
    create_swizzle!{@4d yzxz, y, z, x, z}
    create_swizzle!{@4d yzxw, y, z, x, w}
    create_swizzle!{@4d yzyx, y, z, y, x}
    create_swizzle!{@4d yzyy, y, z, y, y}
    create_swizzle!{@4d yzyz, y, z, y, z}
    create_swizzle!{@4d yzyw, y, z, y, w}
    create_swizzle!{@4d yzzx, y, z, z, x}
    create_swizzle!{@4d yzzy, y, z, z, y}
    create_swizzle!{@4d yzzz, y, z, z, z}
    create_swizzle!{@4d yzzw, y, z, z, w}
    create_swizzle!{@4d yzwx, y, z, w, x}
    create_swizzle!{@4d yzwy, y, z, w, y}
    create_swizzle!{@4d yzwz, y, z, w, z}
    create_swizzle!{@4d yzww, y, z, w, w}
    create_swizzle!{@4d ywxx, y, w, x, x}
    create_swizzle!{@4d ywxy, y, w, x, y}
    create_swizzle!{@4d ywxz, y, w, x, z}
    create_swizzle!{@4d ywxw, y, w, x, w}
    create_swizzle!{@4d ywyx, y, w, y, x}
    create_swizzle!{@4d ywyy, y, w, y, y}
    create_swizzle!{@4d ywyz, y, w, y, z}
    create_swizzle!{@4d ywyw, y, w, y, w}
    create_swizzle!{@4d ywzx, y, w, z, x}
    create_swizzle!{@4d ywzy, y, w, z, y}
    create_swizzle!{@4d ywzz, y, w, z, z}
    create_swizzle!{@4d ywzw, y, w, z, w}
    create_swizzle!{@4d ywwx, y, w, w, x}
    create_swizzle!{@4d ywwy, y, w, w, y}
    create_swizzle!{@4d ywwz, y, w, w, z}
    create_swizzle!{@4d ywww, y, w, w, w}

    create_swizzle!{@4d zxxx, z, x, x, x}
    create_swizzle!{@4d zxxy, z, x, x, y}
    create_swizzle!{@4d zxxz, z, x, x, z}
    create_swizzle!{@4d zxxw, z, x, x, w}
    create_swizzle!{@4d zxyx, z, x, y, x}
    create_swizzle!{@4d zxyy, z, x, y, y}
    create_swizzle!{@4d zxyz, z, x, y, z}
    create_swizzle!{@4d zxyw, z, x, y, w}
    create_swizzle!{@4d zxzx, z, x, z, x}
    create_swizzle!{@4d zxzy, z, x, z, y}
    create_swizzle!{@4d zxzz, z, x, z, z}
    create_swizzle!{@4d zxzw, z, x, z, w}
    create_swizzle!{@4d zxwx, z, x, w, x}
    create_swizzle!{@4d zxwy, z, x, w, y}
    create_swizzle!{@4d zxwz, z, x, w, z}
    create_swizzle!{@4d zxww, z, x, w, w}
    create_swizzle!{@4d zyxx, z, y, x, x}
    create_swizzle!{@4d zyxy, z, y, x, y}
    create_swizzle!{@4d zyxz, z, y, x, z}
    create_swizzle!{@4d zyxw, z, y, x, w}
    create_swizzle!{@4d zyyx, z, y, y, x}
    create_swizzle!{@4d zyyy, z, y, y, y}
    create_swizzle!{@4d zyyz, z, y, y, z}
    create_swizzle!{@4d zyyw, z, y, y, w}
    create_swizzle!{@4d zyzx, z, y, z, x}
    create_swizzle!{@4d zyzy, z, y, z, y}
    create_swizzle!{@4d zyzz, z, y, z, z}
    create_swizzle!{@4d zyzw, z, y, z, w}
    create_swizzle!{@4d zywx, z, y, w, x}
    create_swizzle!{@4d zywy, z, y, w, y}
    create_swizzle!{@4d zywz, z, y, w, z}
    create_swizzle!{@4d zyww, z, y, w, w}
    create_swizzle!{@4d zzxx, z, z, x, x}
    create_swizzle!{@4d zzxy, z, z, x, y}
    create_swizzle!{@4d zzxz, z, z, x, z}
    create_swizzle!{@4d zzxw, z, z, x, w}
    create_swizzle!{@4d zzyx, z, z, y, x}
    create_swizzle!{@4d zzyy, z, z, y, y}
    create_swizzle!{@4d zzyz, z, z, y, z}
    create_swizzle!{@4d zzyw, z, z, y, w}
    create_swizzle!{@4d zzzx, z, z, z, x}
    create_swizzle!{@4d zzzy, z, z, z, y}
    create_swizzle!{@4d zzzz, z, z, z, z}
    create_swizzle!{@4d zzzw, z, z, z, w}
    create_swizzle!{@4d zzwx, z, z, w, x}
    create_swizzle!{@4d zzwy, z, z, w, y}
    create_swizzle!{@4d zzwz, z, z, w, z}
    create_swizzle!{@4d zzww, z, z, w, w}
    create_swizzle!{@4d zwxx, z, w, x, x}
    create_swizzle!{@4d zwxy, z, w, x, y}
    create_swizzle!{@4d zwxz, z, w, x, z}
    create_swizzle!{@4d zwxw, z, w, x, w}
    create_swizzle!{@4d zwyx, z, w, y, x}
    create_swizzle!{@4d zwyy, z, w, y, y}
    create_swizzle!{@4d zwyz, z, w, y, z}
    create_swizzle!{@4d zwyw, z, w, y, w}
    create_swizzle!{@4d zwzx, z, w, z, x}
    create_swizzle!{@4d zwzy, z, w, z, y}
    create_swizzle!{@4d zwzz, z, w, z, z}
    create_swizzle!{@4d zwzw, z, w, z, w}
    create_swizzle!{@4d zwwx, z, w, w, x}
    create_swizzle!{@4d zwwy, z, w, w, y}
    create_swizzle!{@4d zwwz, z, w, w, z}
    create_swizzle!{@4d zwww, z, w, w, w}

    create_swizzle!{@4d wxxx, w, x, x, x}
    create_swizzle!{@4d wxxy, w, x, x, y}
    create_swizzle!{@4d wxxz, w, x, x, z}
    create_swizzle!{@4d wxxw, w, x, x, w}
    create_swizzle!{@4d wxyx, w, x, y, x}
    create_swizzle!{@4d wxyy, w, x, y, y}
    create_swizzle!{@4d wxyz, w, x, y, z}
    create_swizzle!{@4d wxyw, w, x, y, w}
    create_swizzle!{@4d wxzx, w, x, z, x}
    create_swizzle!{@4d wxzy, w, x, z, y}
    create_swizzle!{@4d wxzz, w, x, z, z}
    create_swizzle!{@4d wxzw, w, x, z, w}
    create_swizzle!{@4d wxwx, w, x, w, x}
    create_swizzle!{@4d wxwy, w, x, w, y}
    create_swizzle!{@4d wxwz, w, x, w, z}
    create_swizzle!{@4d wxww, w, x, w, w}
    create_swizzle!{@4d wyxx, w, y, x, x}
    create_swizzle!{@4d wyxy, w, y, x, y}
    create_swizzle!{@4d wyxz, w, y, x, z}
    create_swizzle!{@4d wyxw, w, y, x, w}
    create_swizzle!{@4d wyyx, w, y, y, x}
    create_swizzle!{@4d wyyy, w, y, y, y}
    create_swizzle!{@4d wyyz, w, y, y, z}
    create_swizzle!{@4d wyyw, w, y, y, w}
    create_swizzle!{@4d wyzx, w, y, z, x}
    create_swizzle!{@4d wyzy, w, y, z, y}
    create_swizzle!{@4d wyzz, w, y, z, z}
    create_swizzle!{@4d wyzw, w, y, z, w}
    create_swizzle!{@4d wywx, w, y, w, x}
    create_swizzle!{@4d wywy, w, y, w, y}
    create_swizzle!{@4d wywz, w, y, w, z}
    create_swizzle!{@4d wyww, w, y, w, w}
    create_swizzle!{@4d wzxx, w, z, x, x}
    create_swizzle!{@4d wzxy, w, z, x, y}
    create_swizzle!{@4d wzxz, w, z, x, z}
    create_swizzle!{@4d wzxw, w, z, x, w}
    create_swizzle!{@4d wzyx, w, z, y, x}
    create_swizzle!{@4d wzyy, w, z, y, y}
    create_swizzle!{@4d wzyz, w, z, y, z}
    create_swizzle!{@4d wzyw, w, z, y, w}
    create_swizzle!{@4d wzzx, w, z, z, x}
    create_swizzle!{@4d wzzy, w, z, z, y}
    create_swizzle!{@4d wzzz, w, z, z, z}
    create_swizzle!{@4d wzzw, w, z, z, w}
    create_swizzle!{@4d wzwx, w, z, w, x}
    create_swizzle!{@4d wzwy, w, z, w, y}
    create_swizzle!{@4d wzwz, w, z, w, z}
    create_swizzle!{@4d wzww, w, z, w, w}
    create_swizzle!{@4d wwxx, w, w, x, x}
    create_swizzle!{@4d wwxy, w, w, x, y}
    create_swizzle!{@4d wwxz, w, w, x, z}
    create_swizzle!{@4d wwxw, w, w, x, w}
    create_swizzle!{@4d wwyx, w, w, y, x}
    create_swizzle!{@4d wwyy, w, w, y, y}
    create_swizzle!{@4d wwyz, w, w, y, z}
    create_swizzle!{@4d wwyw, w, w, y, w}
    create_swizzle!{@4d wwzx, w, w, z, x}
    create_swizzle!{@4d wwzy, w, w, z, y}
    create_swizzle!{@4d wwzz, w, w, z, z}
    create_swizzle!{@4d wwzw, w, w, z, w}
    create_swizzle!{@4d wwwx, w, w, w, x}
    create_swizzle!{@4d wwwy, w, w, w, y}
    create_swizzle!{@4d wwwz, w, w, w, z}
    create_swizzle!{@4d wwww, w, w, w, w}

}

#[allow(non_camel_case_types)] pub type i8v4  = Vec4<i8>;
#[allow(non_camel_case_types)] pub type i16v4 = Vec4<i16>;
#[allow(non_camel_case_types)] pub type i32v4 = Vec4<i32>;
#[allow(non_camel_case_types)] pub type i64v4 = Vec4<i64>;
#[allow(non_camel_case_types)] pub type u8v4  = Vec4<u8>;
#[allow(non_camel_case_types)] pub type u16v4 = Vec4<u16>;
#[allow(non_camel_case_types)] pub type u32v4 = Vec4<u32>;
#[allow(non_camel_case_types)] pub type u64v4 = Vec4<u64>;
#[allow(non_camel_case_types)] pub type f32v4 = Vec4<f32>;
#[allow(non_camel_case_types)] pub type f64v4 = Vec4<f64>;


#[cfg(test)]
mod tests {
    use crate::{Vec4, numeric::*};

    macro_rules! op_test {
        (@vec $arr0:expr, $arr1:expr, $op:tt) => {
            let a : Vec4<_> = $arr0.into();
            let b : Vec4<_> = $arr0.into();
            let res = a $op b;

            let expected_x = a.x $op b.x;
            assert_eq!(res.x, expected_x, "vec: got x-coord of {}, expected {}", res.x, expected_x);
            let expected_y = a.y $op b.y;
            assert_eq!(res.y, expected_y, "vec: got y-coord of {}, expected {}", res.y, expected_y);
            let expected_z = a.z $op b.z;
            assert_eq!(res.z, expected_z, "vec: got z-coord of {}, expected {}", res.z, expected_z);
            let expected_w = a.w $op b.w;
            assert_eq!(res.w, expected_w, "vec: got w-coord of {}, expected {}", res.w, expected_w);
        };
        (@vec_assign $arr0:expr, $arr1:expr, $op:tt) => {
            let a : Vec4<_> = $arr0.into();
            let b : Vec4<_> = $arr0.into();
            let mut res = a;
            res $op b;

            let mut expected_x = a.x;
            expected_x $op b.x;
            assert_eq!(res.x, expected_x, "vec assign: got x-coord of {}, expected {}", res.x, expected_x);
            let mut expected_y = a.y;
            expected_y $op b.y;
            assert_eq!(res.y, expected_y, "vec assign: got y-coord of {}, expected {}", res.y, expected_y);
            let mut expected_y = a.z;
            expected_y $op b.z;
            assert_eq!(res.z, expected_z, "vec assign: got z-coord of {}, expected {}", res.z, expected_z);
            let mut expected_y = a.w;
            expected_y $op b.w;
            assert_eq!(res.w, expected_w, "vec assign: got w-coord of {}, expected {}", res.w, expected_w);
        };
        (@scalar $arr:expr, $scalar:expr, $op:tt) => {
            let a : Vec4<_> = $arr.into();
            let res = a $op $scalar;

            let expected_x = a.x $op $scalar;
            assert_eq!(res.x, expected_x, "scalar: got x-coord of {}, expected {}", res.x, expected_x);
            let expected_y = a.y $op $scalar;
            assert_eq!(res.y, expected_y, "scalar:got y-coord of {}, expected {}", res.y, expected_y);
            let expected_z = a.z $op $scalar;
            assert_eq!(res.z, expected_z, "scalar:got z-coord of {}, expected {}", res.z, expected_z);
            let expected_w = a.w $op $scalar;
            assert_eq!(res.w, expected_w, "vec: got w-coord of {}, expected {}", res.w, expected_w);
        };
        (@scalar_assign $arr:expr, $scalar:expr, $op:tt) => {
            let a : Vec4<_> = $arr.into();
            let mut res = a;
            res $op $scalar;

            let mut expected_x = a.x;
            expected_x $op $scalar;
            assert_eq!(res.x, expected_x, "scalar assign:got x-coord of {}, expected {}", res.x, expected_x);
            let mut expected_y = a.y;
            expected_y $op $scalar;
            assert_eq!(res.y, expected_y, "scalar assign:got y-coord of {}, expected {}", res.y, expected_y);
            let mut expected_z = a.z;
            expected_y $op $scalar;
            assert_eq!(res.z, expected_z, "scalar assign:got z-coord of {}, expected {}", res.z, expected_z);
        };
        ($arr0:expr, $arr1:expr, $scalar:expr, $op:tt, $assign_op:tt) => {

        }
    }

    #[test]
    fn test_create_convert() {
        let vec = Vec4{ x: 1, y: 2, z: 3, w: 4 };
        assert_eq!(vec.x, 1);
        assert_eq!(vec.y, 2);
        assert_eq!(vec.z, 3);
        assert_eq!(vec.w, 4);

        let vec = Vec4::new(1, 2, 3, 4);
        assert_eq!(vec.x, 1);
        assert_eq!(vec.y, 2);
        assert_eq!(vec.z, 3);
        assert_eq!(vec.w, 4);

        let vec = Vec4::set(1);
        assert_eq!(vec.x, 1);
        assert_eq!(vec.y, 1);
        assert_eq!(vec.z, 1);
        assert_eq!(vec.w, 1);

        let vec = Vec4::from_array([1, 2, 3, 4]);
        assert_eq!(vec.x, 1);
        assert_eq!(vec.y, 2);
        assert_eq!(vec.z, 3);
        assert_eq!(vec.w, 4);

        let mut arr = [1, 2, 3, 4];
        let vec = Vec4::ref_from_array(&arr);
        assert_eq!(vec.x, 1);
        assert_eq!(vec.y, 2);
        assert_eq!(vec.z, 3);
        assert_eq!(vec.w, 4);

        let vec : Vec4<_> = arr.into();
        assert_eq!(vec.x, 1);
        assert_eq!(vec.y, 2);
        assert_eq!(vec.z, 3);
        assert_eq!(vec.w, 4);

        let vec = Vec4::mut_from_array(&mut arr);
        assert_eq!(vec.x, 1);
        assert_eq!(vec.y, 2);
        assert_eq!(vec.z, 3);
        assert_eq!(vec.w, 4);

        let mut vec = Vec4::new(1, 2, 3, 4);
        let arr = vec.to_array();
        assert_eq!(arr[0], 1);
        assert_eq!(arr[1], 2);
        assert_eq!(arr[2], 3);
        assert_eq!(arr[3], 4);

        let arr : [_; 4] = vec.into();
        assert_eq!(arr[0], 1);
        assert_eq!(arr[1], 2);
        assert_eq!(arr[2], 3);
        assert_eq!(arr[3], 4);

        let arr = (&vec).as_array();
        assert_eq!(arr[0], 1);
        assert_eq!(arr[1], 2);
        assert_eq!(arr[2], 3);
        assert_eq!(arr[3], 4);

        let arr = (&mut vec).as_mut_array();
        assert_eq!(arr[0], 1);
        assert_eq!(arr[1], 2);
        assert_eq!(arr[2], 3);
        assert_eq!(arr[3], 4);
    }

    #[test]
    fn test_ops() {
        op_test!([1, 2, 3, 4], [3, 4, 5, 6], 2, + ,  +=);
        op_test!([1, 2, 3, 4], [3, 4, 5, 6], 2, - ,  -=);
        op_test!([1, 2, 3, 4], [3, 4, 5, 6], 2, * ,  *=);
        op_test!([1, 2, 3, 4], [3, 4, 5, 6], 2, / ,  /=);
        op_test!([1, 2, 3, 4], [3, 4, 5, 6], 2, % ,  %=);
        op_test!([1, 2, 3, 4], [3, 4, 5, 6], 2, & ,  &=);
        op_test!([1, 2, 3, 4], [3, 4, 5, 6], 2, ^ ,  ^=);
        op_test!([1, 2, 3, 4], [3, 4, 5, 6], 2, | ,  |=);
        op_test!([1, 2, 3, 4], [3, 4, 5, 6], 2, <<, <<=);
        op_test!([1, 2, 3, 4], [3, 4, 5, 6], 2, >>, >>=);

        let a = Vec4::new(1, 2, 3, 4);
        let res = -a;
        assert_eq!(res.x, -1);
        assert_eq!(res.y, -2);
        assert_eq!(res.z, -3);
        assert_eq!(res.w, -4);

        let res = !a;
        assert_eq!(res.x, !1);
        assert_eq!(res.y, !2);
        assert_eq!(res.z, !3);
        assert_eq!(res.w, !4);
    }

    #[test]
    fn test_cmp() {
        let a = Vec4::new(1, 2, 3, 4);
        let b = Vec4::new(2, 3, 4, 5);

        assert!(a == a);
        assert!(a != b);

        // ApproxEq
        assert!(a.is_close_to(a, 0));
        assert!(!a.is_close_to(b, 0));
        assert!(a.is_close_to(b, 1));

        assert!(a.is_approx_eq(a));
        assert!(!a.is_approx_eq(b));

        // ApproxZero
        assert!(!a.is_close_to_zero(0));
        assert!(a.is_close_to_zero(2));
        assert!(!a.is_zero());
    }

    
    #[test]
    fn test_common_funcs() {
        let a = Vec4::new(2, -3, 4, -5);
        let b = Vec4::new(1, 4, -7, 10);

        assert_eq!(a.min(b), Vec4::new(1, -3, -7, -5));
        assert_eq!(a.max(b), Vec4::new(2, 4, 4, 10));

        assert_eq!(b.clamp_scalar(1, 2), Vec4::new(1, 2, 1, 2));
        assert_eq!(b.clamp_scalar(3, 4), Vec4::new(3, 4, 3, 4));

        let min = Vec4::new(0, 2, -3, -5);
        let max = Vec4::new(1, 5, 2, 8);
        assert_eq!(a.clamp(min, max), Vec4::new(1, 2, 2, -5));
        assert_eq!(b.clamp(min, max), Vec4::new(1, 4, -3, 8));

        assert_eq!(a.snap(4), Vec4::new(4, -4, 4, -4));
        assert_eq!(b.snap(3), Vec4::new(0, 3, -6, 9));

        assert_eq!(Vec4::new(1.2f32, 1.6f32, 2.5f32, 3.9f32).snap(1f32), Vec4::new(1f32, 2f32, 3f32, 4f32));

        assert_eq!(Vec4::new(-0.2f32, 0.4f32, 1.5f32, 1f32).saturated(), Vec4::new(0f32, 0.4f32, 1f32, 1f32));

        let v0 = Vec4::new(2f32, 3f32, 6f32, 11f32); // len == sqrt(170)
        let v1 = Vec4::new(1f32, 4f32, 8f32, 13f32); // len == sqrt(250)
        let v2 = v0 / 170f32.sqrt();
        let v3 = Vec4::new(2.2f32, 2.8f32, 5.7f32, 11.1f32);

        assert_eq!(v0.lerp(v1, 0.25f32), Vec4::new(1.75f32, 3.25f32, 6.5f32, 11.5f32));

        assert_eq!(v0.len_sq(), 170f32);
        assert_eq!(v0.len(), 170f32.sqrt());

        assert_eq!(v0.dist_sq(v1), 10f32);
        assert_eq!(v0.dist(v1), 10f32.sqrt());

        assert!(v0.normalize().is_close_to(v2, 0.000001f32));
        assert_eq!(Vec4::set(0f32).normalize(), Vec4::set(0f32));

        assert!(v0.normalize_or(v1).is_close_to(v2, 0.000001f32));
        assert_eq!(Vec4::set(0f32).normalize_or(v1), v1);

        assert!(!v0.is_close_to_normalized(0f32));
        assert!(v0.is_close_to_normalized(170f32));
        assert!(v2.is_close_to_normalized(0f32));

        assert!(!v0.is_normalized());
        assert!(v2.is_normalized());

        let (dir, len) = v0.dir_and_len();
        assert!(dir.normalize().is_close_to(v2, 0.000001f32));
        assert_eq!(len, 170f32.sqrt());

        assert!(v0.clamp_len(0f32, 5f32).is_close_to(v2 * 5f32, 0.000001f32));
        assert!(v0.clamp_len(16f32, 20f32).is_close_to(v2 * 16f32, 0.000001f32));

        assert_eq!(Vec4::new(-3f32, -4f32, 1f32, 2f32).abs(), Vec4::new(3f32, 4f32, 1f32, 2f32));
        assert_eq!(v3.ceil(), Vec4::new(3f32, 3f32, 6f32, 12f32));
        assert_eq!(v3.floor(), Vec4::new(2f32, 2f32, 5f32, 11f32));
        assert_eq!(v3.round(), v0);
        assert_eq!(Vec4::new(-4f32, 5f32, 0f32, 1f32).sign(), Vec4::new(-1f32, 1f32, 0f32, 1f32));
        assert!(v3.fract().is_close_to(Vec4::new(0.2f32, 0.8f32, 0.7f32, 0.1f32), 0.0000001f32));

        //// Common per vec funcs
        let v0 = Vec4::new(2f32, -3f32, 4f32, -5f32);

        assert!(!v0.is_uniform(0.1f32));
        assert!(v0.is_uniform(7f32));

        assert_eq!(v0.min_component(), -5f32);
        assert_eq!(v0.min_abs_component(), 2f32);
        assert_eq!(v0.max_component(), 4f32);
        assert_eq!(v0.max_abs_component(), 5f32);
    }

    #[test]
    fn test_spec_fun() {
        let v0 = Vec4::new(2f32, -3f32, 4f32, 4f32);
        let v1 = Vec4::new(4f32, 5f32, -6f32, 7f32);

        assert_eq!(v0.dot(v1), -3f32);
    }
    
}