use crate::*;


//------------------------------------------------------------------------------------------------------------------------------
// 2D
//------------------------------------------------------------------------------------------------------------------------------

//- 2D ray-line intersection ---------------------------------------------------------------------------------------------------

impl<T: Real> IntersectWithRay<T, Ray2D<T>> for Line2D<T> {
    fn intersect_ray(&self, ray: &Ray2D<T>) -> Option<T> {
        let dir_cross = ray.dir.cross(self.dir);
        if dir_cross.is_zero() {
            return None;
        }

        let ab = self.orig - ray.orig;
        let ray_cross = ab.cross(self.dir);
        let t = ray_cross / dir_cross;

        if t >= T::zero() {
            Some(t)
        } else {
            None
        }
    }
}

impl<T: Real> IntersectWithRay<T, BoundedRay2D<T>> for Line2D<T> {
    fn intersect_ray(&self, ray: &BoundedRay2D<T>) -> Option<T> {
        let t = <Self as IntersectWithRay<_, Ray2D<_>>>::intersect_ray(self, &ray.to_ray());
        t.filter(|&val| val >= ray.min && val <= ray.max)
        
    }
}

//- 2D ray-line segnent intersection -------------------------------------------------------------------------------------------

impl<T: Real> IntersectWithRay<T, Ray2D<T>> for LineSegment2D<T> {
    fn intersect_ray(&self, ray: &Ray2D<T>) -> Option<T> {
        let segment_dir = self.end - self.begin;
        let dir_cross = ray.dir.cross(segment_dir);
        if dir_cross.is_zero() {
            return None;
        }

        let ab = self.begin - ray.orig;

        // Value on line segment
        let segment_cross = ab.cross(ray.dir);
        let segment_t = segment_cross / dir_cross;
        if segment_t < T::zero() || segment_t > T::one() {
            return None;
        }

        // Value on ray
        let ray_cross = ab.cross(segment_dir);
        let t = ray_cross / dir_cross;

        if t >= T::zero() {
            Some(t)
        } else {
            None
        }
    }
}

impl<T: Real> IntersectWithRay<T, BoundedRay2D<T>> for LineSegment2D<T> {
    fn intersect_ray(&self, ray: &BoundedRay2D<T>) -> Option<T> {
        let t = <Self as IntersectWithRay<_, Ray2D<_>>>::intersect_ray(self, &ray.to_ray());
        t.filter(|&val| val >= ray.min && val <= ray.max)
        
    }
}

//- 2D ray-ray intersection ----------------------------------------------------------------------------------------------------

impl<T: Real> IntersectWithRay<T, Ray2D<T>> for Ray2D<T> {
    // Keep in mind that this function works from the perspective of the ray that's given to us
    fn intersect_ray(&self, ray: &Ray2D<T>) -> Option<T> {
        // Math explenation, also holds for:
        // - Lines: no need to check if it's on the line
        // - Segments: the same logic, with the exception that the segment's point is calculated between [0; segment.len()] + segment's dir lenght cancels out
        //
        // Given: 
        //     'point on ray0 = a + t * r' and 'point on ray1 = b + u * s ', where the point is the common point on the line,
        //
        // We get:
        //     a + t * r = b + u * s
        //
        // And by using the fact that `v ⨯ v == 0, we can rewrite the equation for:
        // - ray0:
        //          (a + t * r) ⨯ s = (b + u * s) ⨯ s
        //    (a ⨯ s) + t * (r ⨯ s) = (b ⨯ s) + u * (s ⨯ s)
        //    (a ⨯ s) + t * (r ⨯ s) = b ⨯ s
        //              t * (r ⨯ s) = (b ⨯ s) - (a ⨯ s)
        //              t * (r ⨯ s) = (b - a) ⨯ s
        //                        t = ((b - a) ⨯ s) / (r ⨯ s)
        //
        // - ray1: 
        //    using the same logic as above, but crossing with r, we can get: 
        //        u = ((b - a) ⨯ r) / (s ⨯ r)
        //
        //    additionally with property: (s ⨯ r) = - (r ⨯ s), we get:
        //        u = ((b - a) ⨯ r) / -(r ⨯ s)
        //
        // We can use these equation to calculate the point and distance t, as long as 0 <= t <= 1 
        //
        // With the edge case that if `(r ⨯ s) == 0`, we have parallel ray, and so no intersection, as we will not count it as such

        let dir_cross = ray.dir.cross(self.dir);
        if dir_cross.is_zero() {
            return None;
        }
        
        let ab = self.orig - ray.orig;
        
        // Value on line segment
        let self_cross = ab.cross(ray.dir);
        let self_t = self_cross / dir_cross;
        if self_t < T::zero() {
            return None;
        }
        
        // Value on ray
        let ray_cross = ab.cross(self.dir);
        let t = ray_cross / dir_cross;
        
        if t >= T::zero() {
            Some(t)
        } else {
            None
        }
    }
}

impl<T: Real> IntersectWithRay<T, BoundedRay2D<T>> for Ray2D<T> {
    fn intersect_ray(&self, ray: &BoundedRay2D<T>) -> Option<T> {
        let t = <Self as IntersectWithRay<_, Ray2D<_>>>::intersect_ray(self, &ray.to_ray());
        t.filter(|&val| val >= ray.min && val <= ray.max)
        
    }
}

impl<T: Real> IntersectWithRay<T, Ray2D<T>> for BoundedRay2D<T> {
    // Keep in mind that this function works from the perspective of the ray that's given to us
    fn intersect_ray(&self, ray: &Ray2D<T>) -> Option<T> {

        let dir_cross = ray.dir.cross(self.dir);
        if dir_cross.is_zero() {
            return None;
        }
        
        let ab = self.orig - ray.orig;
        
        // Value on line segment
        let self_cross = ab.cross(ray.dir);
        let self_t = self_cross / dir_cross;
        if self_t < self.min || self_t > self.max {
            return None;
        }
        
        // Value on ray
        let ray_cross = ab.cross(self.dir);
        let t = ray_cross / dir_cross;
        
        if t >= T::zero() {
            Some(t)
        } else {
            None
        }
    }
}

impl<T: Real> IntersectWithRay<T, BoundedRay2D<T>> for BoundedRay2D<T> {
    fn intersect_ray(&self, ray: &BoundedRay2D<T>) -> Option<T> {
        let t = <Self as IntersectWithRay<_, Ray2D<_>>>::intersect_ray(self, &ray.to_ray());
        t.filter(|&val| val >= ray.min && val <= ray.max)
        
    }
}

//- 2D ray-circle intersection -------------------------------------------------------------------------------------------------

impl<T: Real> IntersectWithRay<T, Ray2D<T>> for Circle<T> {
    fn intersect_ray(&self, ray: &Ray2D<T>) -> Option<T> {
        let ray_to_center = self.center - ray.orig;
        let closest_dist_to_center = ray_to_center.dot(ray.dir);

        // Ray is pointing in opposite direction
        if closest_dist_to_center < T::zero() {
            return None;
        }

        let mid = ray.orig + ray.dir * closest_dist_to_center;

        let mid_to_center_len_sq = self.center.dist_sq(mid);
        let radius_2 = self.radius * self.radius;

        let t = if mid_to_center_len_sq > radius_2 {
            return None
        } else if mid_to_center_len_sq.is_approx_eq(radius_2) {
            closest_dist_to_center
        } else {
            let t_diff = (radius_2 - mid_to_center_len_sq).sqrt();
            closest_dist_to_center - t_diff
        };

        if t > T::zero() {
            Some(t)
        } else {
            None
        }
    }
}

impl<T: Real> IntersectWithRay<T, BoundedRay2D<T>> for Circle<T> {
    fn intersect_ray(&self, ray: &BoundedRay2D<T>) -> Option<T> {
        let t = <Self as IntersectWithRay<_, Ray2D<_>>>::intersect_ray(self, &ray.to_ray());
        t.filter(|&val| val >= ray.min && val <= ray.max)
        
    }
}

//- 2D ray-rectangle intersection ----------------------------------------------------------------------------------------------

impl<T: Real> IntersectWithRay<T, Ray2D<T>> for Rect<T> {
    // Based on https://en.wikipedia.org/wiki/Cohen%E2%80%93Sutherland_algorithm
    fn intersect_ray(&self, ray: &Ray2D<T>) -> Option<T> {
        let to_min = self.min - ray.orig;
        let to_max = self.max - ray.orig;
        let max_dist = to_min.len_sq().max(to_max.len());

        let begin = ray.orig;
        let end = ray.orig + ray.dir * max_dist;

        let begin_quadrant = self.quadrant(begin);
        let end_quadrant = self.quadrant(end);

        // If they have matching quadrants, we start outside the rect and move away from it or parrallel to a side, so we can just ignore it
        if (begin_quadrant == RectQuadrant::Inside && end_quadrant == RectQuadrant::Inside) || begin_quadrant.intersects(end_quadrant) {
            return None;
        }

        // We only care about the firs intersection we encounter, so no need to use a second pass, but we do use a slighly modifed version because of it
        let mut t = None;

        // If the origin starts outside of the center, we can just rely on knowing that we are going towards the center.
        // The correct point will always be the point furthest away, since to get to the intersection with that side, we first need to pass the other edge
        if begin_quadrant != RectQuadrant::Inside {
            if begin_quadrant.contains(RectQuadrant::Left) {
                let dist = to_min.x / ray.dir.x;
                t = Some(dist);
            } else if begin_quadrant.contains(RectQuadrant::Right) {
                let dist = to_max.x / ray.dir.x;
                t = Some(dist);
            }
            if begin_quadrant.contains(RectQuadrant::Bottom) {
                let dist = to_min.y / ray.dir.y;
                t = t.map_or(Some(dist), |t| Some(t.max(dist)));
            } else if begin_quadrant.contains(RectQuadrant::Top) {
                let dist = to_max.y / ray.dir.y;
                t = t.map_or(Some(dist), |t| Some(t.max(dist)));
            }
        } else {
            if end_quadrant.contains(RectQuadrant::Left) {
                let dist = to_min.x / ray.dir.x;
                t = Some(dist);
            } else if end_quadrant.contains(RectQuadrant::Right) {
                let dist = to_max.x / ray.dir.x;
                t = Some(dist);
            }
            if end_quadrant.contains(RectQuadrant::Bottom) {
                let dist = to_min.y / ray.dir.y;
                t = t.map_or(Some(dist), |t| Some(t.max(dist)));
            } else if end_quadrant.contains(RectQuadrant::Top) {
                let dist = to_max.y / ray.dir.y;
                t = t.map_or(Some(dist), |t| Some(t.max(dist)));
            }
        }

        // Make sure that the point end up on the rect, so either inside or
        if let Some(dist) = t {
            let quadrant = self.quadrant(ray.orig + ray.dir * (dist - T::EPSILON));
            if quadrant.intersects(end_quadrant) {
                t = None;
            }
        }
        
        t.filter(|&val| val >= T::zero())
    }
}

impl<T: Real> IntersectWithRay<T, BoundedRay2D<T>> for Rect<T> {
    fn intersect_ray(&self, ray: &BoundedRay2D<T>) -> Option<T> {
        let t = <Self as IntersectWithRay<_, Ray2D<_>>>::intersect_ray(self, &ray.to_ray());
        t.filter(|&val| val >= ray.min && val <= ray.max)
        
    }
}

//------------------------------------------------------------------------------------------------------------------------------
// 3D
//------------------------------------------------------------------------------------------------------------------------------

//- 3d ray-plane intersection --------------------------------------------------------------------------------------------------

//impl IntersectWithRay<T, Ray<T>> for Plane<T> {
//    fn intersect_ray(&self, ray: &Ray<T>) -> Option<T> {
//        todo!()
//    }
//}
