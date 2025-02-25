use super::*;

pub fn perspective_projection(vfov: Float) -> Mat4 {
    assert!(vfov > 0.0 && vfov < 180.0);

    let near = 1e-2;
    let far = 1e3;
    let a = far / (far - near);
    let b = -far * near / (far - near);
    let projection = Mat4::from_cols(
        Vec4::new(1.0, 0.0, 0.0, 0.0),
        Vec4::new(0.0, 1.0, 0.0, 0.0),
        Vec4::new(0.0, 0.0, a,   b),
        Vec4::new(0.0, 0.0, 1.0, 0.0),
    ).transpose();
    let tan_vfov_inv = 1.0 / (vfov.to_radians() / 2.0).tan();

    let scale = Mat4::from_scale(Vec3::new(tan_vfov_inv, tan_vfov_inv, 1.0));
    scale * projection
}

pub fn orthographic_projection() -> Mat4 {
    let near = 0.0;
    let far = 1.0;
    Mat4::from_scale(Vec3::new(1.0, 1.0, 1.0 / (far - near)))
        * Mat4::from_translation(Vec3::new(0.0, 0.0, -near))
}


pub fn camera_to_world(origin: Point, towards: Point, up: Direction) -> Transform {
    assert!(towards.distance_squared(origin) > crate::EPSILON);
    assert!(up.length() != 0.0);
    // x = right, y = up, z = towards
    let forward = (towards - origin).normalize();
    let right = forward.cross(up).normalize();
    let up = right.cross(forward);
    Transform::from_mat3_translation(
        Mat3::from_cols(right, up, forward).transpose(),
        -1.0 * Vec3::new(origin.dot(right), origin.dot(up), origin.dot(forward)),
    ).inverse()
}

pub fn raster_to_screen(resolution: (i32, i32), zoom: Float) -> Transform {
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
    // or clip to raster
    let screen_to_raster =
    // ndc_to_raster
        Transform::from_scale(Vec3::new(width as Float, -height as Float, 1.0))
    // screen_to_ndc
        * Transform::from_scale(Vec3::new(1.0 / screen_delta.x, 1.0 / screen_delta.y, 1.0))
        * Transform::from_translation(Vec3::new(-screen_min.x, -screen_max.y, 0.0))
        * Transform::from_scale(Vec3::splat(zoom));

    // or raster to clip
    screen_to_raster.inverse()
}
