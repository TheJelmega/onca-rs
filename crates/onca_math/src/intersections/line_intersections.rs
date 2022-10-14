use crate::*;

//-- 2D: line-line intersection ------------------------------------------------------------------------------------------------

impl<T: Real> Intersect<Line2D<T>> for Line2D<T> {
    type Output = T;

    fn intersect(self, rhs: Line2D<T>) -> Self::Output {
        let ab = rhs.orig - self.orig;
        let ab_cross = ab.cross(rhs.dir);
        let dir_cross = self.dir.cross(rhs.dir);

        ab_cross / dir_cross
    }
}

//-- 2D: line-ray intersection -------------------------------------------------------------------------------------------------

impl<T: Real> Intersect<Ray2D<T>> for Line2D<T> {
    type Output = <Ray2D<T> as Intersect<Line2D<T>>>::Output;

    /// Calculate the itnersection point between a 2D ray and a circle
    /// 
    /// the result is given as the ray parameter of the closest intersection, INF if no intersection happened
    fn intersect(self, rhs: Ray2D<T>) -> Self::Output {
        <Ray2D<T> as Intersect<Line2D<T>>>::intersect(rhs, self)
    }
}

//-- 2D: ray-line segment intersection -----------------------------------------------------------------------------------------

impl<T: Real> Intersect<LineSegment2D<T>> for Line2D<T> {
    type Output = T;

    fn intersect(self, rhs: LineSegment2D<T>) -> Self::Output {
        let segment_dir = (rhs.end - rhs.begin).normalize();
        let ab = rhs.begin - self.orig;
        let dir_cross = self.dir.cross(segment_dir);
        
        // If we are not on the segment, no intersection happened
        let seg_cross = ab.cross(self.dir);

        // `seg_cross / dir_cross` is in range [0; 1], so just compare with [0; dir_cross]
        if seg_cross < T::zero() || seg_cross > dir_cross {
            return T::INF;
        }
        
        let ab_cross = ab.cross(segment_dir);
        ab_cross / dir_cross
    }
}

impl<T: Real> Intersect<Line2D<T>> for LineSegment2D<T> {
    type Output = <Line2D<T> as Intersect<LineSegment2D<T>>>::Output;

    /// Calculate the itnersection point between a 2D ray and a circle
    /// 
    /// the result is given as the ray parameter of the closest intersection, INF if no intersection happened
    fn intersect(self, rhs: Line2D<T>) -> Self::Output {
        <Line2D<T> as Intersect<LineSegment2D<T>>>::intersect(rhs, self)
    }
}

//-- 2D: line-circle intersection ----------------------------------------------------------------------------------------------

impl<T: Real> Intersect<Circle<T>> for Line2D<T> {
    type Output = (T, T);

    /// Calculate the intersection points between a 2D ray and a circle
    /// 
    /// the result is given as a tuple of the distances to the intersections, INF if no intersection happened
    fn intersect(self, rhs: Circle<T>) -> Self::Output {

        let dist_to_mid = self.dist(rhs.center);
        let mid = self.orig + self.dir * dist_to_mid;
        let dist_from_center_sq = (mid - rhs.center).len_sq();

        if dist_from_center_sq > rhs.radius {
            (T::INF, T::INF)
        } else {
            let radius2 = rhs.radius * rhs.radius;
            if dist_from_center_sq.is_approx_eq(radius2) {
                (dist_to_mid, T::INF)
            } else {
                let offset = (radius2 - dist_from_center_sq).sqrt();
                (dist_to_mid - offset, dist_to_mid + offset)
            }
        }
    }
}

impl<T: Real> Intersect<Line2D<T>> for Circle<T> {
    type Output = <Line2D<T> as Intersect<Circle<T>>>::Output;

    /// Calculate the intersection points between a 2D ray and a circle
    /// 
    /// the result is given as a tuple of the distances to the intersections, INF if no intersection happened
    fn intersect(self, rhs: Line2D<T>) -> Self::Output {
        <Line2D<T> as Intersect<Circle<T>>>::intersect(rhs, self)
    }
}

//-- 2D: line-rect intersection ------------------------------------------------------------------------------------------------

impl<T: Real> Intersect<Rect<T>> for Line2D<T> {
    type Output = (T, T);

    /// Calculate the intersection points between a 2D ray and a rect
    /// 
    /// the result is given as a tuple of the distances to the intersections, INF if no intersection happened
    fn intersect(self, rhs: Rect<T>) -> Self::Output {
        // When dir.? == 0, division will result in -/+INF

        let t0x = (rhs.min.x - self.orig.x) / self.dir.x;
        let t1x = (rhs.max.x - self.orig.x) / self.dir.x;        
        let t_min = t0x.min(t1x);
        let t_max = t0x.max(t1x);
        
        let t0y = (rhs.min.y - self.orig.y) / self.dir.y;
        let t1y = (rhs.max.y - self.orig.y) / self.dir.y;
        let t_min = t_min.max(t0y.min(t1y));
        let t_max = t_max.min(t0y.max(t1y));

        (t_min, t_max)
    }
}

impl<T: Real> Intersect<Line2D<T>> for Rect<T> {
    type Output = <Line2D<T> as Intersect<Rect<T>>>::Output;

    /// Calculate the intersection points between a 2D ray and a rect
    /// 
    /// the result is given as a tuple of the distances to the intersections, INF if no intersection happened
    fn intersect(self, rhs: Line2D<T>) -> Self::Output {
        <Line2D<T> as Intersect<Rect<T>>>::intersect(rhs, self)
    }
}

//-- 3D: line-plane intersection -----------------------------------------------------------------------------------------------

impl<T: Real> Intersect<Plane<T>> for Line<T> {
    type Output = T;

    fn intersect(self, rhs: Plane<T>) -> Self::Output {
        let plane_point = rhs.normal * rhs.dist;
        let ray_to_plane = plane_point - self.orig;
        let ray_to_plane_dot = ray_to_plane.dot(rhs.normal);
        let dir_dot = self.dir.dot(rhs.normal);

        ray_to_plane_dot / dir_dot
    }
}

impl<T: Real> Intersect<Line<T>> for Plane<T> {
    type Output = <Line<T> as Intersect<Plane<T>>>::Output;

    /// Calculate the intersection points between a 2D ray and a rect
    /// 
    /// the result is given as a tuple of the distances to the intersections, INF if no intersection happened
    fn intersect(self, rhs: Line<T>) -> Self::Output {
        <Line<T> as Intersect<Plane<T>>>::intersect(rhs, self)
    }
}

//-- 3D: line-sphere intersection ----------------------------------------------------------------------------------------------

impl<T: Real> Intersect<Sphere<T>> for Line<T> {
    type Output = (T, T);

    /// Calculate the intersection points between a 2D ray and a circle
    /// 
    /// the result is given as a tuple of the distances to the intersections, INF if no intersection happened
    fn intersect(self, rhs: Sphere<T>) -> Self::Output {

        let dist_to_mid = self.dist(rhs.center);
        let mid = self.orig + self.dir * dist_to_mid;
        let dist_from_center_sq = (mid - rhs.center).len_sq();

        if dist_from_center_sq > rhs.radius {
            (T::INF, T::INF)
        } else {
            let radius2 = rhs.radius * rhs.radius;
            if dist_from_center_sq.is_approx_eq(radius2) {
                (dist_to_mid, T::INF)
            } else {
                let offset = (radius2 - dist_from_center_sq).sqrt();
                (dist_to_mid - offset, dist_to_mid + offset)
            }
        }
    }
}

impl<T: Real> Intersect<Line<T>> for Sphere<T> {
    type Output = <Line<T> as Intersect<Sphere<T>>>::Output;

    /// Calculate the intersection points between a 2D ray and a circle
    /// 
    /// the result is given as a tuple of the distances to the intersections, INF if no intersection happened
    fn intersect(self, rhs: Line<T>) -> Self::Output {
        <Line<T> as Intersect<Sphere<T>>>::intersect(rhs, self)
    }
}

//-- 3D: line-aabb intersection ------------------------------------------------------------------------------------------------

impl<T: Real> Intersect<AABB<T>> for Line<T> {
    type Output = (T, T);

    /// Calculate the intersection points between a 2D ray and a rect
    /// 
    /// the result is given as a tuple of the distances to the intersections, INF if no intersection happened
    fn intersect(self, rhs: AABB<T>) -> Self::Output {
        // When dir.? == 0, division will result in -/+INF

        let t0x = (rhs.min.x - self.orig.x) / self.dir.x;
        let t1x = (rhs.max.x - self.orig.x) / self.dir.x;        
        let t_min = t0x.min(t1x);
        let t_max = t0x.max(t1x);
        
        let t0y = (rhs.min.y - self.orig.y) / self.dir.y;
        let t1y = (rhs.max.y - self.orig.y) / self.dir.y;
        let t_min = t_min.max(t0y.min(t1y));
        let t_max = t_max.min(t0y.max(t1y));

        let t0z = (rhs.min.z - self.orig.z) / self.dir.z;
        let t1z = (rhs.max.z - self.orig.z) / self.dir.z;
        let t_min = t_min.max(t0z.min(t1z));
        let t_max = t_max.min(t0z.max(t1z));

        (t_min, t_max)
    }
}

impl<T: Real> Intersect<Line<T>> for AABB<T> {
    type Output = <Line<T> as Intersect<AABB<T>>>::Output;

    /// Calculate the intersection points between a 2D ray and a rect
    /// 
    /// the result is given as a tuple of the distances to the intersections, INF if no intersection happened
    fn intersect(self, rhs: Line<T>) -> Self::Output {
        <Line<T> as Intersect<AABB<T>>>::intersect(rhs, self)
    }
}
