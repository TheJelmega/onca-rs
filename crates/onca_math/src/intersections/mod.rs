use crate::Real;


/// A trait to calculate the intersection with a 2D ray.
pub trait IntersectWithRay<T: Real, R: Copy> {
    /// Calculat the closest intersection on the ray and return the distance on the ray, or `None` if no collision happened.
    fn intersect_ray(&self, ray: &R) -> Option<T>;
}

mod ray_intersections;
//mod line_intersections;

#[cfg(test)]
mod test;