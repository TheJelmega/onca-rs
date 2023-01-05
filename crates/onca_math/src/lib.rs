//! A generic math library, but built with the purpose to fit the needs of the Onca game engine
//! 
//! Future plans:
//! - Fixed point numbers
//! - Gillbert Algebra or PGA (Projected Geometric Algebra)
//!     - Should be a more general version that includes most standard 
//!     - Should not have negative performance impact
//!     - More info: [`Siggraph 2019: Geometric algebra`]
//! 
//! [`Siggraph 2019: Geometric algebra`]: <https://www.youtube.com/watch?v=tX4H_ctggYo>

#![allow(dead_code)]

mod numeric;
pub use numeric::*;

mod constants;
pub use constants::*;

mod angle;
pub use angle::*;

mod vec;
pub use vec::*;

mod mat;
pub use mat::*;

mod quat;
pub use quat::*;

mod plane;
pub use plane::*;

mod rect;
pub use rect::*;

mod circle;
pub use circle::*;

mod aabb;
pub use aabb::*;

mod sphere;
pub use sphere::*;

mod ray;
pub use ray::*;

mod line;
pub use line::*;

mod intersections;
pub use intersections::*;

pub mod pixel;