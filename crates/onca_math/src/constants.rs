/// Trait that defines common math constants
pub trait MathConsts {

    /// Minimum value
    const MIN : Self;
    /// Vaximum value
    const MAX : Self;

    /// pi
    const PI : Self;
    /// 2 * pi
    const TWO_PI : Self;
    /// pi / 2
    const HALF_PI : Self;
    /// 3 * pi / 2
    const THREE_OVER_TWO_PI : Self;
    /// pi / 4
    const QUARTER_PI : Self;
    /// 1 / pi
    const ONE_OVER_PI : Self;
    /// 1 / (2 * pi)
    const ONE_OVER_TWO_PI : Self;
    /// 2 / pi
    const TWO_OVER_PI : Self;
    /// 4 / pi
    const FOUR_OVER_PI : Self;

    /// sqrt(pi)
    const ROOT_PI : Self;
    /// sqrt(pi / 2)
    const ROOT_HALF_PI : Self;
    /// sqrt(2 * pi)
    const ROOT_TWO_PI : Self;
    /// 1 / sqrt(pi)
    const ONE_OVER_ROOT_PI : Self;

    /// sqrt(2)
    const ROOT_TWO : Self;
    /// 1 / sqrt(2)
    const ONE_OVER_ROOT_TWO : Self;
    /// sqrt(3)
    const ROOT_THREE : Self;
    /// sqrt(5)
    const ROOT_FIVE : Self;

    /// Ln(2)
    const LN_TWO : Self;
    /// Ln(10)
    const LN_TEN : Self;

    /// 1 / 3
    const THIRD : Self;
    /// 2 / 3
    const TWO_THIRDS : Self;

    /// e
    const E : Self;
    /// euler
    const EULER : Self;
    /// golden ration
    const GOLDEN_RATIO : Self;

    /// pi / 180
    const DEG_TO_RAD : Self;
    /// 180 / pi
    const RAD_TO_DEG : Self;
}

macro_rules! impl_math_constants {
    {$($ty:ty),*} => {
        $(
            impl MathConsts for $ty {
                const MIN               : $ty = <$ty>::MIN;
                const MAX               : $ty = <$ty>::MAX;

                const PI                : $ty = 3.14159265358979323846264338327950288 as $ty;
                const TWO_PI            : $ty = 6.28318530717958647692528676655900576 as $ty;
                const HALF_PI           : $ty = 1.57079632679489661923132169163975144 as $ty;
                const THREE_OVER_TWO_PI : $ty = 4.71238898038468985769396507491925432 as $ty;
                const QUARTER_PI        : $ty = 0.785398163397448309615660845819875721 as $ty;
                const ONE_OVER_PI       : $ty = 0.318309886183790671537767526745028724 as $ty;
                const ONE_OVER_TWO_PI   : $ty = 0.159154943091895335768883763372514362 as $ty;
                const TWO_OVER_PI       : $ty = 0.636619772367581343075535053490057448 as $ty;
                const FOUR_OVER_PI      : $ty = 1.273239544735162686151070106980114898 as $ty;

                const ROOT_PI           : $ty = 1.7724538509055160272981674833411 as $ty;
                const ROOT_HALF_PI      : $ty = 1.2533141373155002512078826424055 as $ty;
                const ROOT_TWO_PI       : $ty = 2.506628274631000502415765284811 as $ty;
                const ONE_OVER_ROOT_PI  : $ty = 0.56418958354775628694807945156077 as $ty;

                const ROOT_TWO          : $ty = 1.41421356237309504880168872420969808 as $ty;
                const ONE_OVER_ROOT_TWO : $ty = 0.707106781186547524400844362104849039 as $ty;
                const ROOT_THREE        : $ty = 1.73205080756887729352744634150587236 as $ty;
                const ROOT_FIVE         : $ty = 2.23606797749978969640917366873127623 as $ty;

                const LN_TWO            : $ty = 0.693147180559945309417232121458176568 as $ty;
                const LN_TEN            : $ty = 2.30258509299404568401799145468436421 as $ty;

                const THIRD             : $ty = 0.3333333333333333333333333333333333333333 as $ty;
                const TWO_THIRDS        : $ty = 0.666666666666666666666666666666666666667 as $ty;

                const E                 : $ty = 2.71828182845904523536 as $ty;
                const EULER             : $ty = 0.577215664901532860606 as $ty;
                const GOLDEN_RATIO      : $ty = 1.61803398874989484820458683436563811 as $ty;


                const DEG_TO_RAD        : $ty = (<f64 as MathConsts>::PI / 180.0) as $ty;
                const RAD_TO_DEG        : $ty = (180.0 / <f64 as MathConsts>::PI) as $ty;
            }
        )*
    };
}

impl_math_constants!{ i8, i16, i32, i64, u8, u16, u32, u64, f32, f64 }