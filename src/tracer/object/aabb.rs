use super::*;

pub struct AxisAlignedBoundingBox {
    ax_min: DVec3,
    ax_max: DVec3,
}

impl AxisAlignedBoundingBox {
    pub fn new(object: &dyn Object) -> Self {
        todo!()
    }
}

impl Object for AxisAlignedBoundingBox {
    fn normal_at(&self, p: DVec3) -> DVec3 { p }
    fn material(&self) -> &Material { &Material::Blank }
    fn area(&self) -> f64 { 0.0 }

    fn hit(&self, r: &Ray) -> Option<Hit> {
        let mut ts = -f64::INFINITY;
        let mut te = f64::INFINITY;
        let ro_min = (self.ax_min - r.origin).to_array();
        let ro_max = (self.ax_max - r.origin).to_array();
        let rd = r.dir.to_array();

        (0..3).for_each(|i: usize| {
            let t1 = ro_min[i] / rd[i];
            let t2 = ro_max[i] / rd[i];
            ts = ts.max(t1);
            te = te.min(t2);
        });

        if ts > te || te < EPSILON {
            None
        } else {
            let t = if ts > EPSILON { ts } else { te };
            Hit::new(t, self, r)
        }
    }

}
