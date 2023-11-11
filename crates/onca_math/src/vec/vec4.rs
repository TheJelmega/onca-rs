use std::{
    ops::*,
    fmt::Display
};
use crate::*;

generic_vec!{ doc = "4D Vector (row-major order)"; Vec4, 4, (T, T, T, T), x => 0, y => 1, z => 2, w => 3;
    i8v4  => i8
    i16v4 => i16
    i32v4 => i32
    i64v4 => i64
    u8v4  => u8
    u16v4 => u16
    u32v4 => u32
    u64v4 => u64
    f32v4 => f32
    f64v4 => f64
}

impl<T: Numeric> Vec4<T> {
    /// Shrink a `Vec4` to a `Vec3`
    #[inline]
    #[must_use]
    pub fn shrink(self) -> Vec3<T> {
        Vec3 { x: self.x, y: self.y, z: self.z }
    }

    /// Check if the 4d vector represents a 3d vector (z-coord == 0)
    pub fn represents_3d_vector(self) -> bool where
        T: ApproxZero
    {
        self.w.is_zero()
    }

    /// Check if the 4d vector represents a 3d point (z-coord == 1)
    pub fn represents_3d_point(self) -> bool where
        T: One + ApproxEq
    {
        self.w.is_approx_eq(T::one())
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
        write!(f, "({}, {}, {}, {})", self.x, self.y, self.z, self.w)
    }
}

// Swizzles
impl<T: Numeric> Vec4<T> {
    /// Swizzle the components of the vector into a `Vec2`
    pub fn swizzle2(self, x: Swizzle, y: Swizzle) -> Vec2<T> {
        debug_assert!(x <= Swizzle::W);
        debug_assert!(y <= Swizzle::W);
        Vec2 { x: self[x as usize], y: self[y as usize] }
    }

    /// Swizzle the components of the vector into a `Vec3`
    pub fn swizzle3(self, x: Swizzle, y: Swizzle, z: Swizzle) -> Vec3<T> {
        debug_assert!(x <= Swizzle::W);
        debug_assert!(y <= Swizzle::W);
        debug_assert!(z <= Swizzle::W);
        Vec3 { x: self[x as usize], y: self[y as usize], z: self[z as usize] }
    }

    /// Swizzle the components of the vector
    pub fn swizzle(self, x: Swizzle, y: Swizzle, z: Swizzle, w: Swizzle) -> Self {
        debug_assert!(x <= Swizzle::W);
        debug_assert!(y <= Swizzle::W);
        debug_assert!(z <= Swizzle::W);
        debug_assert!(w <= Swizzle::W);
        Self { x: self[x as usize], y: self[y as usize], z: self[z as usize], w: self[w as usize] }
    }

    create_swizzle!{@2d
        xx => x, x  xy => x, y  xz => x, z  xw => x, w
        yx => y, x  yy => y, y  yz => y, z  yw => y, w
        zx => z, x  zy => z, y  zz => z, z  zw => z, w
        wx => w, x  wy => w, y  wz => w, z  ww => w, w
    }

    create_swizzle!{@3d
        xxx => x, x, x  xxy => x, x, y  xxz => x, x, z  xxw => x, x, w
        xyx => x, y, x  xyy => x, y, y  xyz => x, y, z  xyw => x, y, w
        xzx => x, z, x  xzy => x, z, y  xzz => x, z, z  xzw => x, z, w
        xwx => x, w, x  xwy => x, w, y  xwz => x, w, z  xww => x, w, w
        yxx => y, x, x  yxy => y, x, y  yxz => y, x, z  yxw => y, x, w
        yyx => y, y, x  yyy => y, y, y  yyz => y, y, z  yyw => y, y, w
        yzx => y, z, x  yzy => y, z, y  yzz => y, z, z  yzw => y, z, w
        ywx => y, w, x  ywy => y, w, y  ywz => y, w, z  yww => y, w, w
        zxx => z, x, x  zxy => z, x, y  zxz => z, x, z  zxw => z, x, w
        zyx => z, y, x  zyy => z, y, y  zyz => z, y, z  zyw => z, y, w
        zzx => z, z, x  zzy => z, z, y  zzz => z, z, z  zzw => z, z, w
        zwx => z, w, x  zwy => z, w, y  zwz => z, w, z  zww => z, w, w
        wxx => w, x, x  wxy => w, x, y  wxz => w, x, z  wxw => w, x, w
        wyx => w, y, x  wyy => w, y, y  wyz => w, y, z  wyw => w, y, w
        wzx => w, z, x  wzy => w, z, y  wzz => w, z, z  wzw => w, z, w
        wwx => w, w, x  wwy => w, w, y  wwz => w, w, z  www => w, w, w
    }

    create_swizzle!{@4d
        xxxx => x, x, x, x  xxxy => x, x, x, y  xxxz => x, x, x, z  xxxw => x, x, x, w
        xxyx => x, x, y, x  xxyy => x, x, y, y  xxyz => x, x, y, z  xxyw => x, x, y, w
        xxzx => x, x, z, x  xxzy => x, x, z, y  xxzz => x, x, z, z  xxzw => x, x, z, w
        xxwx => x, x, w, x  xxwy => x, x, w, y  xxwz => x, x, w, z  xxww => x, x, w, w
        xyxx => x, y, x, x  xyxy => x, y, x, y  xyxz => x, y, x, z  xyxw => x, y, x, w
        xyyx => x, y, y, x  xyyy => x, y, y, y  xyyz => x, y, y, z  xyyw => x, y, y, w
        xyzx => x, y, z, x  xyzy => x, y, z, y  xyzz => x, y, z, z  xyzw => x, y, z, w
        xywx => x, y, w, x  xywy => x, y, w, y  xywz => x, y, w, z  xyww => x, y, w, w
        xzxx => x, z, x, x  xzxy => x, z, x, y  xzxz => x, z, x, z  xzxw => x, z, x, w
        xzyx => x, z, y, x  xzyy => x, z, y, y  xzyz => x, z, y, z  xzyw => x, z, y, w
        xzzx => x, z, z, x  xzzy => x, z, z, y  xzzz => x, z, z, z  xzzw => x, z, z, w
        xzwx => x, z, w, x  xzwy => x, z, w, y  xzwz => x, z, w, z  xzww => x, z, w, w
        xwxx => x, w, x, x  xwxy => x, w, x, y  xwxz => x, w, x, z  xwxw => x, w, x, w
        xwyx => x, w, y, x  xwyy => x, w, y, y  xwyz => x, w, y, z  xwyw => x, w, y, w
        xwzx => x, w, z, x  xwzy => x, w, z, y  xwzz => x, w, z, z  xwzw => x, w, z, w
        xwwx => x, w, w, x  xwwy => x, w, w, y  xwwz => x, w, w, z  xwww => x, w, w, w

        yxxx => y, x, x, x  yxxy => y, x, x, y  yxxz => y, x, x, z  yxxw => y, x, x, w
        yxyx => y, x, y, x  yxyy => y, x, y, y  yxyz => y, x, y, z  yxyw => y, x, y, w
        yxzx => y, x, z, x  yxzy => y, x, z, y  yxzz => y, x, z, z  yxzw => y, x, z, w
        yxwx => y, x, w, x  yxwy => y, x, w, y  yxwz => y, x, w, z  yxww => y, x, w, w
        yyxx => y, y, x, x  yyxy => y, y, x, y  yyxz => y, y, x, z  yyxw => y, y, x, w
        yyyx => y, y, y, x  yyyy => y, y, y, y  yyyz => y, y, y, z  yyyw => y, y, y, w
        yyzx => y, y, z, x  yyzy => y, y, z, y  yyzz => y, y, z, z  yyzw => y, y, z, w
        yywx => y, y, w, x  yywy => y, y, w, y  yywz => y, y, w, z  yyww => y, y, w, w
        yzxx => y, z, x, x  yzxy => y, z, x, y  yzxz => y, z, x, z  yzxw => y, z, x, w
        yzyx => y, z, y, x  yzyy => y, z, y, y  yzyz => y, z, y, z  yzyw => y, z, y, w
        yzzx => y, z, z, x  yzzy => y, z, z, y  yzzz => y, z, z, z  yzzw => y, z, z, w
        yzwx => y, z, w, x  yzwy => y, z, w, y  yzwz => y, z, w, z  yzww => y, z, w, w
        ywxx => y, w, x, x  ywxy => y, w, x, y  ywxz => y, w, x, z  ywxw => y, w, x, w
        ywyx => y, w, y, x  ywyy => y, w, y, y  ywyz => y, w, y, z  ywyw => y, w, y, w
        ywzx => y, w, z, x  ywzy => y, w, z, y  ywzz => y, w, z, z  ywzw => y, w, z, w
        ywwx => y, w, w, x  ywwy => y, w, w, y  ywwz => y, w, w, z  ywww => y, w, w, w

        zxxx => z, x, x, x  zxxy => z, x, x, y  zxxz => z, x, x, z  zxxw => y, x, x, w
        zxyx => z, x, y, x  zxyy => z, x, y, y  zxyz => z, x, y, z  zxyw => y, x, y, w
        zxzx => z, x, z, x  zxzy => z, x, z, y  zxzz => z, x, z, z  zxzw => y, x, z, w
        zxwx => z, x, w, x  zxwy => z, x, w, y  zxwz => z, x, w, z  zxww => y, x, w, w
        zyxx => z, y, x, x  zyxy => z, y, x, y  zyxz => z, y, x, z  zyxw => y, y, x, w
        zyyx => z, y, y, x  zyyy => z, y, y, y  zyyz => z, y, y, z  zyyw => y, y, y, w
        zyzx => z, y, z, x  zyzy => z, y, z, y  zyzz => z, y, z, z  zyzw => y, y, z, w
        zywx => z, y, w, x  zywy => z, y, w, y  zywz => z, y, w, z  zyww => y, y, w, w
        zzxx => z, z, x, x  zzxy => z, z, x, y  zzxz => z, z, x, z  zzxw => y, z, x, w
        zzyx => z, z, y, x  zzyy => z, z, y, y  zzyz => z, z, y, z  zzyw => y, z, y, w
        zzzx => z, z, z, x  zzzy => z, z, z, y  zzzz => z, z, z, z  zzzw => y, z, z, w
        zzwx => z, z, w, x  zzwy => z, z, w, y  zzwz => z, z, w, z  zzww => y, z, w, w
        zwxx => z, w, x, x  zwxy => z, w, x, y  zwxz => z, w, x, z  zwxw => y, w, x, w
        zwyx => z, w, y, x  zwyy => z, w, y, y  zwyz => z, w, y, z  zwyw => y, w, y, w
        zwzx => z, w, z, x  zwzy => z, w, z, y  zwzz => z, w, z, z  zwzw => y, w, z, w
        zwwx => z, w, w, x  zwwy => z, w, w, y  zwwz => z, w, w, z  zwww => y, w, w, w

        wxxx => w, x, x, x  wxxy => w, x, x, y  wxxz => w, x, x, z  wxxw => w, x, x, w
        wxyx => w, x, y, x  wxyy => w, x, y, y  wxyz => w, x, y, z  wxyw => w, x, y, w
        wxzx => w, x, z, x  wxzy => w, x, z, y  wxzz => w, x, z, z  wxzw => w, x, z, w
        wxwx => w, x, w, x  wxwy => w, x, w, y  wxwz => w, x, w, z  wxww => w, x, w, w
        wyxx => w, y, x, x  wyxy => w, y, x, y  wyxz => w, y, x, z  wyxw => w, y, x, w
        wyyx => w, y, y, x  wyyy => w, y, y, y  wyyz => w, y, y, z  wyyw => w, y, y, w
        wyzx => w, y, z, x  wyzy => w, y, z, y  wyzz => w, y, z, z  wyzw => w, y, z, w
        wywx => w, y, w, x  wywy => w, y, w, y  wywz => w, y, w, z  wyww => w, y, w, w
        wzxx => w, z, x, x  wzxy => w, z, x, y  wzxz => w, z, x, z  wzxw => w, z, x, w
        wzyx => w, z, y, x  wzyy => w, z, y, y  wzyz => w, z, y, z  wzyw => w, z, y, w
        wzzx => w, z, z, x  wzzy => w, z, z, y  wzzz => w, z, z, z  wzzw => w, z, z, w
        wzwx => w, z, w, x  wzwy => w, z, w, y  wzwz => w, z, w, z  wzww => w, z, w, w
        wwxx => w, w, x, x  wwxy => w, w, x, y  wwxz => w, w, x, z  wwxw => w, w, x, w
        wwyx => w, w, y, x  wwyy => w, w, y, y  wwyz => w, w, y, z  wwyw => w, w, y, w
        wwzx => w, w, z, x  wwzy => w, w, z, y  wwzz => w, w, z, z  wwzw => w, w, z, w
        wwwx => w, w, w, x  wwwy => w, w, w, y  wwwz => w, w, w, z  wwww => w, w, w, w
    }

}
