use super::*;

/// Abstraction of a vertex in the paths
pub struct Vertex<'a> {
    pub h: Hit<'a>,
    pub gathered: Color,
    pub pdf_fwd: Float,
    pub pdf_bck: Float,
}

impl<'a> Vertex<'a> {
    /// Camera vertex
    pub fn camera(xo: Point, gathered: Color) -> Self {
        let h = Hit::new(
            0.0,
            &Material::Blank,
            Direction::NEG_X,
            xo,
            Vec3::ZERO,
            Normal::X,
            Normal::X,
            Vec2::X,
        ).unwrap();
        Self {
            h,
            gathered,
            pdf_bck: 1.0,
            pdf_fwd: 0.0,
        }
    }

    /// Light vertex
    pub fn light(mut h: Hit<'a>, light: &'a dyn Sampleable, gathered: Color, pdf_bck: Float) -> Self {
        h.light = Some(light);
        Self {
            h,
            gathered,
            pdf_bck,
            // this might cause issues later on (if area is zero, point light?)...
            pdf_fwd: 1.0 / light.area(),
        }
    }

    /// Surface vertex
    pub fn surface(
        h: Hit<'a>,
        gathered: Color,
        pdf_fwd: Float,
        prev: &Vertex,
    ) -> Self {
        let pdf_fwd = if h.material.is_delta() {
            0.0
        } else {
            let xo = prev.h.p;
            let xi = h.p;
            let wi = (xi - xo).normalize();
            let ng = h.ng;

            pdf_fwd * wi.dot(ng).abs() / xi.distance_squared(xo)
        };
        Self {
            h,
            gathered,
            pdf_fwd,
            pdf_bck: 0.0,
        }
    }

    fn material(&self) -> &Material {
        self.h.material
    }

    /// Are we a surface/light vertex?
    pub fn is_surface(&self) -> bool {
        !matches!(self.material(), Material::Blank)
    }

    /// Are we on a light?
    pub fn is_light(&self) -> bool {
        self.h.is_light()
    }

    /// Are we on a surface with delta material?
    pub fn is_delta(&self) -> bool {
        self.material().is_delta()
    }

    /// Helper to get emittance at hit
    pub fn emittance(&self) -> Color {
        self.material().emit(&self.h)
    }

    /// Helper to get shading cosine at hit
    pub fn shading_cosine(&self, wi: Direction, ns: Normal) -> Float {
        self.material().shading_cosine(wi, ns)
    }

    /// Computes BSDF at hit of `self`
    pub fn bsdf(&self, prev: &Vertex, next: &Vertex, mode: Transport) -> Color {
        // TODO (2)
        let wo = (self.h.p - prev.h.p).normalize();
        let wi = (next.h.p - self.h.p).normalize();

        self.material().bsdf_f(wo, wi, mode, &self.h)
    }

    /// Converts solid angle `pdf` to area PDF
    pub fn solid_angle_to_area(&self, pdf: Float, next: &Vertex) -> Float {
        let xo = self.h.p;
        let xi = next.h.p;
        let wi = (xi - xo).normalize();

        if next.is_surface() {
            let ng = next.h.ng;
            pdf * wi.dot(ng).abs() / xi.distance_squared(xo)
        } else {
            pdf / xi.distance_squared(xo)
        }
    }

    /// Geometry term btwn `self` and `v` ... ...
    /// it is symmetric??
    pub fn g(&self, v: &Vertex) -> Float {
        let xo = self.h.p;
        let xi = v.h.p;
        let no = self.h.ns;
        let ni = v.h.ns;

        let wi = (xi - xo).normalize();

        no.dot(wi).abs() * ni.dot(wi).abs() / xo.distance_squared(xi)
    }

    /// PDF to sample direction to `next` from `curr` w.r.t. surface area measure
    pub fn pdf_area(&self, prev: &Vertex, next: &Vertex, mode: Transport) -> Float {
        let ho = &self.h;
        // prev
        let xo = prev.h.p;
        // curr
        let xi = ho.p;
        // prev -> curr
        let wo = xi - xo;
        let ro = Ray::new(xo, wo);
        // next
        let xii = next.h.p;
        // curr -> next
        let wi = xii - xi;
        let ri = Ray::new(xi, wi);
        // normalized
        let wi = ri.dir;
        let angle_pdf = match self.material().bsdf_pdf(ho, &ro) {
            None => 0.0,
            Some(pdf) => pdf.value_for(&ri, matches!(mode, Transport::Importance))
        };
        let ng = next.h.ng;

        // convert solid angle to area at next
        angle_pdf * wi.dot(ng).abs() / xi.distance_squared(xii)
    }

    pub fn pdf_light_origin(&self) -> Float {
        self.h.light.map_or(0.0, |light| 1.0 / light.area())
    }

    pub fn pdf_light_leaving(&self, next: &Vertex) -> Float {
        if let Some(ref light) = self.h.light {
            let xo = self.h.p;
            let xi = next.h.p;
            let wi = xi - xo;
            let ri = Ray::new(xo, wi);
            // normalized
            let wi = ri.dir;
            let ng = self.h.ng;
            let (_, pdf_dir) = light.sample_leaving_pdf(&ri, ng);
            let ng = next.h.ng;
            // convert solid angle to area
            pdf_dir * wi.dot(ng).abs() / xo.distance_squared(xi)
        } else {
            0.0
        }
    }
}
