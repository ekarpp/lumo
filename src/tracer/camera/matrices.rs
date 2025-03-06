use super::*;

pub fn perspective_projection(vfov: Float) -> Transform {
    assert!(vfov > 0.0 && vfov < 180.0);

    let near = 1e-2;
    let far = 1e3;
    let projection = Transform::perspective(near, far);
    let tan_vfov_inv = 1.0 / (vfov.to_radians() / 2.0).tan();

    let scale = Transform::scale(tan_vfov_inv, tan_vfov_inv, 1.0);
    scale * projection
}

pub fn orthographic_projection() -> Transform {
    let near = 0.0;
    let far = 1.0;
    Transform::scale(1.0, 1.0, 1.0 / (far - near))
        * Transform::translation(0.0, 0.0, -near)
}


pub fn world_to_camera(origin: Point, towards: Point, up: Direction) -> Transform {
    assert!(towards.distance_squared(origin) > crate::EPSILON);
    assert!(up.length() != 0.0);
    // x = right, y = up, z = towards
    let forward = (towards - origin).normalize();
    let right = forward.cross(up).normalize();
    let up = right.cross(forward);

    Transform::translation(-origin.dot(right), -origin.dot(up), -origin.dot(forward))
        * Transform::mat3(Mat3::new(right, up, forward))
}

pub fn screen_to_raster(resolution: (u64, u64), zoom: Float) -> Transform {
    assert!(resolution.0 > 0 && resolution.1 > 0);
    assert!(zoom > 0.0);

    let (width, height) = resolution;
    // make aspect ratio of image plane same as it is for film
    let aspect_ratio = width as Float / height as Float;
    let (screen_min, screen_max) = if aspect_ratio > 1.0 {
        (
            Vec2::new(-aspect_ratio, -1.0),
            Vec2::new(aspect_ratio, 1.0),
        )
    } else {
        (
            Vec2::new(-1.0, -1.0 / aspect_ratio),
            Vec2::new(1.0, 1.0 / aspect_ratio),
        )
    };

    let screen_delta = screen_max - screen_min;


    // screen_to_ndc
    // {
    // ndc_to_raster
    Transform::scale(width as Float, -(height as Float), 1.0)
    // screen_to_ndc
        * Transform::scale(1.0 / screen_delta.x, 1.0 / screen_delta.y, 1.0)
        * Transform::translation(-screen_min.x, -screen_max.y, 0.0)
        * Transform::scale(zoom, zoom, zoom)
    // }
}
