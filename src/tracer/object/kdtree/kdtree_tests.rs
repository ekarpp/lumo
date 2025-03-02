use super::*;
use std::collections::{VecDeque, HashSet};

const NUM_RAYS: usize = 10_000;

mod util {
    use super::*;

    pub fn _aabb_contains_triangle(aabb: AaBoundingBox, triangle: &Triangle) -> bool {
        let aabb_triangle = triangle.bounding_box();

        [Axis::X, Axis::Y, Axis::Z].iter().all(|axis| {
            // aabbs intersect
            let intersect = aabb_triangle.min(*axis) < aabb.max(*axis)
                && aabb.min(*axis) < aabb_triangle.max(*axis);

            // triangle aabb is planar to aabb and on aabb
            let planar = (aabb_triangle.min(*axis) == aabb_triangle.max(*axis))
                && (aabb_triangle.min(*axis) == aabb.min(*axis)
                    || aabb_triangle.max(*axis) == aabb.max(*axis));

            intersect || planar
        })
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

macro_rules! test_mesh {
    ( $( $name:ident, $mesh:expr, $backface:literal ),* ) => {
        $(
            mod $name {
                use super::*;

                #[test]
                fn intersects() {
                    let mut rng = Xorshift::default();
                    let mesh = $mesh
                        .to_unit_size()
                        .to_origin();

                    for _ in 0..NUM_RAYS {
                        let xo = 5.0 * rng::maps::square_to_sphere(rng.gen_vec2());
                        // shoot towards origin from sphere of radius 5
                        let ray = Ray::new(xo, -xo);
                        println!("{}", xo);
                        let hit = mesh.hit(&ray, 0.0, crate::INF);
                        // make sure we hit the object
                        assert!(hit.is_some());
                        let hit = hit.unwrap();

                        if $backface {
                            // make sure we didn't hit the inside
                            assert!(hit.ng.dot(-ray.dir) > 0.0);
                        }
                    }
                }

                #[test]
                fn splits() {
                    let mesh = $mesh;

                    let mut stack = VecDeque::from([(mesh.root, mesh.boundary)]);
                    while let Some((node, aabb)) = stack.pop_front() {
                        match *node {
                            KdNode::Leaf(indices) => {
                                assert!(indices.iter().all(|idx| {
                                    util::_aabb_contains_triangle(aabb, &mesh.objects[*idx])
                                }));

                                assert!(mesh.objects.iter().enumerate()
                                        .all(|(idx, triangle)| {
                                            !util::_aabb_contains_triangle(aabb, triangle)
                                                || indices.contains(&idx)
                                        }));
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
                fn contains() {
                    let mesh = $mesh;
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
            }
        )*
    }
}

test_mesh!{
    sphere, mesh::sphere(), true,
    teapot, mesh::teapot(), true,
    cube,   mesh::cube(),   false
}

mod mesh {
    use super::*;

    const TEAPOT_URL: &str =
        "https://casual-effects.com/g3d/data10/common/model/teapot/teapot.zip";
    const SPHERE_URL: &str =
        "http://web.mit.edu/djwendel/www/weblogo/shapes/basic-shapes/sphere/sphere.obj";

    pub fn sphere() -> Mesh {
        crate::parser::mesh_from_url(SPHERE_URL, Material::Blank).unwrap()
    }

    pub fn teapot() -> Mesh {
        crate::parser::mesh_from_url(TEAPOT_URL, Material::Blank).unwrap()
    }

    pub fn cube() -> Mesh {
        let vertices = vec!(
            Vec3::new(130.0, 165.0, 65.0),
            Vec3::new(82.0, 165.0, 225.0),
            Vec3::new(240.0, 165.0, 272.0),
            Vec3::new(290.0, 165.0, 114.0),

            Vec3::new(290.0, 0.0, 114.0),
            Vec3::new(290.0, 165.0, 114.0),
            Vec3::new(240.0, 165.0, 272.0),
            Vec3::new(240.0, 0.0, 272.0),

            Vec3::new(130.0, 0.0, 65.0),
            Vec3::new(130.0, 165.0, 65.0),
            Vec3::new(290.0, 165.0, 114.0),
            Vec3::new(290.0, 0.0, 114.0),

            Vec3::new(82.0, 0.0, 225.0),
            Vec3::new(82.0, 165.0, 225.0),
            Vec3::new(130.0, 165.0, 65.0),
            Vec3::new(130.0, 0.0, 65.0),

            Vec3::new(240.0, 0.0, 272.0),
            Vec3::new(240.0, 165.0, 272.0),
            Vec3::new(82.0, 165.0, 225.0),
            Vec3::new(82.0, 0.0, 225.0),
        );

        let mut f = vec!();
        for i in 0..=4 {
            let v0 = i * 4;
            f.push(Face::new(vec!(v0, v0 + 1, v0 + 2), vec!(), vec!()));
            f.push(Face::new(vec!(v0, v0 + 2, v0 + 3), vec!(), vec!()));
        }

        TriangleMesh::new(vertices, f, vec!(), vec!(), Material::Blank)
    }
}
