use super::*;

const TEAPOT_URL: &str = "https://graphics.stanford.edu/courses/cs148-10-summer/as3/code/as3/teapot.obj";

const SPHERE_URL: &str = "http://web.mit.edu/djwendel/www/weblogo/shapes/basic-shapes/sphere/sphere.obj";

const NUM_RAYS: usize = 10000;

fn shoot_rays(mesh: Box<dyn Object>) {
    for _ in 0..NUM_RAYS {
        let rand_sq = crate::rand_utils::unit_square();
        let ray_origin = crate::rand_utils::square_to_sphere(rand_sq)
            // move points IN sphere to ON sphere
            .normalize();
        let ray = Ray::new(ray_origin, -ray_origin);
        let hit = mesh.hit(&ray, 0.0, INFINITY);
        // make sure we hit the object
        assert!(hit.is_some());
        let hit = hit.unwrap();
        // make sure we didn't hit the inside
        assert!(hit.norm.dot(ray_origin) > 0.0);
    }
}

#[test]
fn intersect_teapot() {
    let mesh = Mesh::new(
        crate::obj::obj_from_url(TEAPOT_URL).unwrap(),
        Material::Blank,
    )
        .to_unit_size()
        .to_origin()
        .scale(0.8, 0.8, 0.8);

    shoot_rays(mesh);
}

#[test]
fn intersect_sphere() {
    let mesh = Mesh::new(
        crate::obj::obj_from_url(SPHERE_URL).unwrap(),
        Material::Blank,
    )
        .to_unit_size()
        .to_origin()
        .scale(0.5, 0.5, 0.5);

    shoot_rays(mesh);
}
