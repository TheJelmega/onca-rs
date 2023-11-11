use crate::*;

#[test]
fn ray_line_2d() {
    let line = Line2D::new(f32p2::new(0.0, 0.0), f32v2::new(1.0, 0.0));
    
    // Parallel
    let ray = BoundedRay2D::new(f32p2::new(0.0, -1.0), f32v2::new(1.0, 0.0), 0.0, 1000.0);
    let t = line.intersect_ray(&ray);
    assert_eq!(t, None);

    // Crossing
    let ray = BoundedRay2D::new(f32p2::new(0.0, -1.0), f32v2::new(0.707, 0.707), 0.0, 1000.0);
    let t = line.intersect_ray(&ray);
    match t {
        Some(val) => assert!(val.is_close_to(1.414, 0.001)),
        None => panic!("None != Some(1.414)"),
    }

    // Behind
    let ray = BoundedRay2D::new(f32p2::new(0.0, -1.0), f32v2::new(0.707, -0.707), 0.0, 1000.0);
    let t = line.intersect_ray(&ray);
    assert_eq!(t, None);

    // before min
    let ray = BoundedRay2D::new(f32p2::new(0.0, -1.0), f32v2::new(0.707, -0.707), 2.0, 1000.0);
    let t = line.intersect_ray(&ray);
    assert_eq!(t, None);

    // after max
    let ray = BoundedRay2D::new(f32p2::new(0.0, -1.0), f32v2::new(0.707, -0.707), 0.0, 1.0);
    let t = line.intersect_ray(&ray);
    assert_eq!(t, None);
}

#[test]
fn ray_line_segment_2d() {
    let segment = LineSegment2D::new(f32p2::new(-2.0, 0.0), f32p2::new(2.0, 0.0));
    
    // Parallel
    let ray = BoundedRay2D::new(f32p2::new(0.0, -1.0), f32v2::new(1.0, 0.0), 0.0, 1000.0);
    let t = segment.intersect_ray(&ray);
    assert_eq!(t, None);

    // Crossing
    let ray = BoundedRay2D::new(f32p2::new(0.0, -1.0), f32v2::new(0.707, 0.707), 0.0, 1000.0);
    let t = segment.intersect_ray(&ray);
    match t {
        Some(val) => assert!(val.is_close_to(1.414, 0.001)),
        None => panic!("None != Some(1.414)"),
    }

    // Behind
    let ray = BoundedRay2D::new(f32p2::new(0.0, -1.0), f32v2::new(0.707, -0.707), 0.0, 1000.0);
    let t = segment.intersect_ray(&ray);
    assert_eq!(t, None);

    // Passing
    let ray = BoundedRay2D::new(f32p2::new(0.0, -1.0), f32v2::new(3.0, 1.0).normalize(), 0.0, 1000.0);
    let t = segment.intersect_ray(&ray);
    assert_eq!(t, None);

    // Before min
    let ray = BoundedRay2D::new(f32p2::new(0.0, -1.0), f32v2::new(0.707, 0.707), 2.0, 1000.0);
    let t = segment.intersect_ray(&ray);
    assert_eq!(t, None);

    // After max
    let ray = BoundedRay2D::new(f32p2::new(0.0, -1.0), f32v2::new(0.707, 0.707), 0.0, 1.0);
    let t = segment.intersect_ray(&ray);
    assert_eq!(t, None);
}

#[test]
fn ray_ray_2d() {
    let other = BoundedRay2D::new(f32p2::new(-2.0, 0.0), f32v2::new(1.0, 0.0), 0.0, 4.0);
    
    // Parallel
    let ray = BoundedRay2D::new(f32p2::new(0.0, -1.0), f32v2::new(1.0, 0.0), 0.0, 1000.0);
    let t = other.intersect_ray(&ray);
    assert_eq!(t, None);

    // Crossing
    let ray = BoundedRay2D::new(f32p2::new(0.0, -1.0), f32v2::new(0.707, 0.707), 0.0, 1000.0);
    let t = other.intersect_ray(&ray);
    match t {
        Some(val) => assert!(val.is_close_to(1.414, 0.001), "Some({}) != Some(1.414)", val),
        None => panic!("None != Some(1.414)"),
    }

    // Behind
    let ray = BoundedRay2D::new(f32p2::new(0.0, -1.0), f32v2::new(0.707, -0.707), 0.0, 1000.0);
    let t = other.intersect_ray(&ray);
    assert_eq!(t, None);

    // Passing
    let ray = BoundedRay2D::new(f32p2::new(0.0, -1.0), f32v2::new(3.0, 1.0).normalize(), 0.0, 1000.0);
    let t = other.intersect_ray(&ray);
    assert_eq!(t, None);

    // Before min
    let ray = BoundedRay2D::new(f32p2::new(0.0, -1.0), f32v2::new(0.707, 0.707), 2.0, 1000.0);
    let t = other.intersect_ray(&ray);
    assert_eq!(t, None);

    // After max
    let ray = BoundedRay2D::new(f32p2::new(0.0, -1.0), f32v2::new(0.707, 0.707), 0.0, 1.0);
    let t = other.intersect_ray(&ray);
    assert_eq!(t, None);
}

#[test]
fn ray_circle_2d() {

    let circle = Circle::new(f32p2::new(0.0, 0.0), 2.0);

    // Miss
    let ray = BoundedRay2D::new(f32p2::new(2.0, -2.0), f32v2::new(0.707, 0.707), 0.0, 1000.0);
    let t = circle.intersect_ray(&ray);
    assert_eq!(t, None);

    // Graze
    let ray = BoundedRay2D::new(f32p2::new(2.0, -2.0), f32v2::new(0.0, 1.0), 0.0, 1000.0);
    let t = circle.intersect_ray(&ray);
    assert_eq!(t, Some(2.0));

    // Through center
    let ray = BoundedRay2D::new(f32p2::new(2.0, -2.0), f32v2::new(-0.707, 0.707), 0.0, 1000.0);
    let t = circle.intersect_ray(&ray);
    match t {
        Some(val) => assert!(val.is_close_to(0.828, 0.001)),
        None => panic!("None != Some(0.828)"),
    }

    // Through circle
    let ray = BoundedRay2D::new(f32p2::new(2.0, -1.0), f32v2::new(-0.707, 0.707), 0.0, 1000.0);
    let t = circle.intersect_ray(&ray);
    match t {
        Some(val) => assert!(val.is_close_to(0.25, 0.001)),
        None => panic!("None != Some(0.828)"),
    }

    // Ray behind
    let ray = BoundedRay2D::new(f32p2::new(2.0, -2.0), f32v2::new(0.707, -0.707), 0.0, 1000.0);
    let t = circle.intersect_ray(&ray);
    assert_eq!(t, None);
    
    // Before min
    let ray = BoundedRay2D::new(f32p2::new(2.0, -2.0), f32v2::new(0.0, 1.0), 10.0, 1000.0);
    let t = circle.intersect_ray(&ray);
    assert_eq!(t, None);

    // After max
    let ray = BoundedRay2D::new(f32p2::new(2.0, -2.0), f32v2::new(0.0, 1.0), 0.0, 1.0);
    let t = circle.intersect_ray(&ray);
    assert_eq!(t, None);
}

#[test]
fn ray_rect_2d() {
    // Quadrants
    //
    //  0 | 1 | 2
    // ---|---|---
    //  3 | 4 | 5
    // ---|---|---
    //  6 | 7 | 8

    let rect = Rect::new(f32p2::new(-1.0, -1.0), f32p2::new(1.0, 1.0));

    // Miss
    let ray = BoundedRay2D::new(f32p2::new(2.0, -2.0), f32v2::new(0.707, 0.707), 0.0, 1000.0);
    let t = rect.intersect_ray(&ray);
    assert_eq!(t, None);

    // Hit from quadrant 0
    let ray = BoundedRay2D::new(f32p2::new(-2.0, 1.5), f32v2::new(0.707, -0.707), 0.0, 1000.0);
    let t = rect.intersect_ray(&ray);
    match t {
        Some(val) => assert!(val.is_close_to(1.414, 0.001)),
        None => panic!("None != Some(1.414)"),
    }

    // Miss from quadrant 0
    let ray = BoundedRay2D::new(f32p2::new(-2.0, 1.5), f32v2::new(1.0, -10.0).normalize(), 0.0, 1000.0);
    let t = rect.intersect_ray(&ray);
    assert_eq!(t, None);

    // Hit from quadrant 1
    let ray = BoundedRay2D::new(f32p2::new(0.0, 2.0), f32v2::new(0.0, -1.0), 0.0, 1000.0);
    let t = rect.intersect_ray(&ray);
    assert_eq!(t, Some(1.0));

    // Hit from quadrant 2
    let ray = BoundedRay2D::new(f32p2::new(2.0, 1.5), f32v2::new(-0.707, -0.707), 0.0, 1000.0);
    let t = rect.intersect_ray(&ray);
    match t {
        Some(val) => assert!(val.is_close_to(1.414, 0.001)),
        None => panic!("None != Some(1.414)"),
    }

    // Hit from quadrant 3
    let ray = BoundedRay2D::new(f32p2::new(-2.0, 0.0), f32v2::new(1.0, 0.0), 0.0, 1000.0);
    let t = rect.intersect_ray(&ray);
    assert_eq!(t, Some(1.0));

    // Hit from quadrant 5
    let ray = BoundedRay2D::new(f32p2::new(2.0, 0.0), f32v2::new(-1.0, 0.0), 0.0, 1000.0);
    let t = rect.intersect_ray(&ray);
    assert_eq!(t, Some(1.0));

    // Hit from quadrant 6
    let ray = BoundedRay2D::new(f32p2::new(-2.0, -1.5), f32v2::new(0.707, 0.707), 0.0, 1000.0);
    let t = rect.intersect_ray(&ray);
    match t {
        Some(val) => assert!(val.is_close_to(1.414, 0.001)),
        None => panic!("None != Some(1.414)"),
    }

    // Hit from quadrant 7
    let ray = BoundedRay2D::new(f32p2::new(0.0, -2.0), f32v2::new(0.0, 1.0), 0.0, 1000.0);
    let t = rect.intersect_ray(&ray);
    assert_eq!(t, Some(1.0));

    // Hit from quadrant 8
    let ray = BoundedRay2D::new(f32p2::new(2.0, -1.5), f32v2::new(-0.707, 0.707), 0.0, 1000.0);
    let t = rect.intersect_ray(&ray);
    match t {
        Some(val) => assert!(val.is_close_to(1.414, 0.001)),
        None => panic!("None != Some(1.414)"),
    }

    //--------------------------------------------------------------

    // Miss inside
    let ray = BoundedRay2D::new(f32p2::new(0.0, 0.0), f32v2::new(-0.707, -0.707).normalize(), 0.0, 1.0);
    let t = rect.intersect_ray(&ray);
    assert_eq!(t, None);

    // Hit from inside to quadrant 0
    let ray = BoundedRay2D::new(f32p2::new(0.0, 0.0), f32v2::new(-0.707, -0.707), 0.0, 1000.0);
    let t = rect.intersect_ray(&ray);
    match t {
        Some(val) => assert!(val.is_close_to(1.414, 0.001)),
        None => panic!("None != Some(1.414)"),
    }

    // Hit from inside to quadrant 1
    let ray = BoundedRay2D::new(f32p2::new(0.0, 0.0), f32v2::new(0.0, 1.0), 0.0, 1000.0);
    let t = rect.intersect_ray(&ray);
    assert_eq!(t, Some(1.0));

    // Hit from inside to quadrant 2
    let ray = BoundedRay2D::new(f32p2::new(0.0, 0.0), f32v2::new(0.707, 0.707), 0.0, 1000.0);
    let t = rect.intersect_ray(&ray);
    match t {
        Some(val) => assert!(val.is_close_to(1.414, 0.001)),
        None => panic!("None != Some(1.414)"),
    }

    // Hit from inside to quadrant 3
    let ray = BoundedRay2D::new(f32p2::new(0.0, 0.0), f32v2::new(-1.0, 0.0), 0.0, 1000.0);
    let t = rect.intersect_ray(&ray);
    assert_eq!(t, Some(1.0));

    // Hit from inside to quadrant 5
    let ray = BoundedRay2D::new(f32p2::new(0.0, 0.0), f32v2::new(1.0, 0.0), 0.0, 1000.0);
    let t = rect.intersect_ray(&ray);
    assert_eq!(t, Some(1.0));

    // Hit from inside to quadrant 6
    let ray = BoundedRay2D::new(f32p2::new(0.0, 0.0), f32v2::new(-0.707, -0.707), 0.0, 1000.0);
    let t = rect.intersect_ray(&ray);
    match t {
        Some(val) => assert!(val.is_close_to(1.414, 0.001)),
        None => panic!("None != Some(1.414)"),
    }

    // Hit from inside to quadrant 7
    let ray = BoundedRay2D::new(f32p2::new(0.0, 0.0), f32v2::new(0.0, -1.0), 0.0, 1000.0);
    let t = rect.intersect_ray(&ray);
    assert_eq!(t, Some(1.0));

    // Hit from inside to quadrant 8
    let ray = BoundedRay2D::new(f32p2::new(0.0, 0.0), f32v2::new(0.707, -0.707), 0.0, 1000.0);
    let t = rect.intersect_ray(&ray);
    match t {
        Some(val) => assert!(val.is_close_to(1.414, 0.001)),
        None => panic!("None != Some(1.414)"),
    }
}
