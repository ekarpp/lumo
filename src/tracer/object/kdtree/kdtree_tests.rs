use super::*;
use std::collections::{VecDeque, HashSet};

const TEAPOT_URL: &str = "https://casual-effects.com/g3d/data10/common/model/teapot/teapot.zip";

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
        let hit = mesh.hit(&ray, 0.0, crate::INF);
        // make sure we hit the object
        assert!(hit.is_some());
        let hit = hit.unwrap();
        // make sure we didn't hit the inside
        assert!(hit.ng.dot(-ray.dir) > 0.0);
    }
}

#[test]
fn intersect_planar() {
    let vertices = vec![
        Point::NEG_X,
        Point::X,
        Point::X + Point::Y,
        Point::NEG_X + Point::Y,
    ];
    let faces = vec![
        Face::new(vec![0, 1, 2], vec![], vec![]),
        Face::new(vec![0, 2, 3], vec![], vec![]),
    ];

    let mesh = TriangleMesh::new(
        vertices,
        faces,
        vec![],
        vec![],
        Material::Blank,
    );

    let r = Ray::new(0.5 * (Point::Y + Point::Z), Point::NEG_Z);
    assert!(mesh.hit(&r, 0.0, crate::INF).is_some());
}

#[test]
fn intersect_teapot() {
    let mesh = crate::parser::mesh_from_url(
        TEAPOT_URL,
        Material::Blank,
    ).unwrap()
        .to_unit_size()
        .to_origin()
        .scale(0.8, 0.8, 0.8);

    shoot_rays(mesh);
}

#[test]
fn intersect_sphere() {
    let mesh = crate::parser::mesh_from_url(
        SPHERE_URL,
        Material::Blank,
    ).unwrap()
        .to_unit_size()
        .to_origin()
        .scale(0.5, 0.5, 0.5);

    shoot_rays(mesh);
}

fn _aabb_contains_triangle(aabb: AaBoundingBox, triangle: &Triangle) -> bool {
    let aabb_triangle = triangle.bounding_box();

    [Axis::X, Axis::Y, Axis::Z].iter().all(|axis| {
        aabb_triangle.min(*axis) < aabb.max(*axis)
            && aabb.min(*axis) < aabb_triangle.max(*axis)
    })
}

#[test]
fn all_objects_correctly_split() {
    let mesh = crate::parser::mesh_from_url(
        TEAPOT_URL,
        Material::Blank,
    ).unwrap();

    let mut stack = VecDeque::from([(mesh.root, mesh.boundary)]);

    while let Some((node, aabb)) = stack.pop_front() {
        match *node {
            KdNode::Leaf(indices) => {
                indices.iter().for_each(|idx| {
                    assert!(_aabb_contains_triangle(aabb, &mesh.objects[*idx]));
                });

                mesh.objects.iter().enumerate()
                    .for_each(|(idx, triangle)| {
                        if _aabb_contains_triangle(aabb, triangle) {
                            assert!(indices.contains(&idx));
                        }
                    });
            }
            KdNode::Split(axis, split, left, right) => {
                let (aabb_left, aabb_right) = aabb.split(axis, split);
                stack.push_front((left, aabb_left));
                stack.push_front((right, aabb_right));
            }
        }
    }
}

#[test]
fn all_objects_in_tree() {
    let mesh = crate::parser::mesh_from_url(
        TEAPOT_URL,
        Material::Blank,
    ).unwrap();

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
