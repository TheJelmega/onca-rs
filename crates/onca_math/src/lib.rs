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

pub(crate) mod common;
pub use common::*;

mod numeric;
pub use numeric::*;

mod constants;
pub use constants::*;

mod utils;

mod angle;
pub use angle::*;

mod vec;
pub use vec::*;

mod point;
pub use point::*;

mod normal;
pub use normal::*;

mod mat;
pub use mat::*;

mod quat;
pub use quat::*;

mod ray;
pub use ray::*;

mod bounded_ray;
pub use bounded_ray::*;

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

mod line;
pub use line::*;

mod intersections;
pub use intersections::*;

mod local_system;
pub use local_system::*;

pub mod pixel;
