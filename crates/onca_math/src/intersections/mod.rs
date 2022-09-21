
pub trait Intersect<Rhs = Self> {
    type Output;

    fn intersect(self, rhs: Rhs) -> Self::Output;
}


mod ray_intersections;
mod line_intersections;