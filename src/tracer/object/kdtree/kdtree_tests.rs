use super::*;
use std::collections::{VecDeque, HashSet};

const TEAPOT_URL: &str = "https://graphics.stanford.edu/courses/cs148-10-summer/as3/code/as3/teapot.obj";

const SPHERE_URL: &str = "http://web.mit.edu/djwendel/www/weblogo/shapes/basic-shapes/sphere/sphere.obj";

const NUM_RAYS: usize = 10000;

fn shoot_rays(mesh: Box<dyn Object>) {
    for _ in 0..NUM_RAYS {
        let rand_sq = crate::rand_utils::unit_square();
        let ray_dir = crate::rand_utils::square_to_sphere(rand_sq);
        let ray_origin = -ray_dir
            // move points IN sphere to ON sphere..
            .normalize()
            // ..that is bigger than the unit cube. (exact value sqrt(3))
            * 2.0;
        let ray = Ray::new(ray_origin, ray_dir);
        let hit = mesh.hit(&ray, 0.0, INFINITY);
        // make sure we hit the object
        assert!(hit.is_some());
        let hit = hit.unwrap();
        // make sure we didn't hit the inside
        assert!(hit.norm.dot(-ray.dir) > 0.0);
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

#[test]
fn all_objects_in_tree() {
    let mesh = Mesh::new(
        crate::obj::obj_from_url(TEAPOT_URL).unwrap(),
        Material::Blank,
    );

    let mut found = HashSet::new();
    let mut stack = VecDeque::from([mesh.root]);

    while let Some(node) = stack.pop_front() {
        match *node {
            KdNode::Leaf(indices) => {
                indices.iter().for_each(|idx| { found.insert(*idx); })
            }
            KdNode::Split(_, _, left, right) => {
                stack.push_back(left);
                stack.push_back(right);
            }
        }
    }

    assert!(found.len() == mesh.objects.len());
}
