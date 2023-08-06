use super::*;

/// Abstraction of a vertex in the paths
pub struct Vertex<'a> {
    pub h: Hit<'a>,
    pub gathered: DVec3,
    pub pdf_fwd: f64,
    pub pdf_bck: f64,
}

impl<'a> Vertex<'a> {
    /// Camera vertex
    pub fn camera(xo: DVec3, gathered: DVec3) -> Self {
        let h = Hit::new(
            0.0,
            &Material::Blank,
            DVec3::NEG_X,
            xo,
            DVec3::ZERO,
            DVec3::X,
            DVec3::X,
            DVec2::X,
        ).unwrap();
        Self {
            h,
            gathered,
            pdf_bck: 1.0,
            pdf_fwd: 0.0,
        }
    }

    /// Light vertex
    pub fn light(mut h: Hit<'a>, light: &'a dyn Sampleable, gathered: DVec3, pdf_bck: f64) -> Self {
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
        gathered: DVec3,
        pdf_fwd: f64,
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
    pub fn emittance(&self) -> DVec3 {
        self.material().emit(&self.h)
    }

    /// Helper to get shading cosine at hit
    pub fn shading_cosine(&self, wi: DVec3, ns: DVec3) -> f64 {
        self.material().shading_cosine(wi, ns)
    }

    /// Computes BSDF at hit of `self`
    pub fn bsdf(&self, prev: &Vertex, next: &Vertex, mode: Transport) -> DVec3 {
        // TODO (2)
        let wo = (self.h.p - prev.h.p).normalize();
        let wi = (next.h.p - self.h.p).normalize();

        self.material().bsdf_f(wo, wi, mode, &self.h)
    }

    /// Converts solid angle `pdf` to area PDF
    pub fn solid_angle_to_area(&self, pdf: f64, next: &Vertex) -> f64 {
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
    pub fn g(&self, v: &Vertex) -> f64 {
        let xo = self.h.p;
        let xi = v.h.p;
        let no = self.h.ns;
        let ni = v.h.ns;

        let wi = (xi - xo).normalize();

        no.dot(wi).abs() * ni.dot(wi).abs() / xo.distance_squared(xi)
    }

    /// PDF to sample direction to `next` from `curr` w.r.t. surface area measure
    pub fn pdf_area(&self, prev: &Vertex, next: &Vertex, mode: Transport) -> f64 {
        if self.is_delta() {
            return 0.0;
        }

        let ho = &self.h;
        let xo = prev.h.p;
        let xi = ho.p;
        let wo = xi - xo;
        let ro = Ray::new(xo, wo);
        let xii = next.h.p;
        let wi = xii - xi;
        let ri = Ray::new(xi, wi);
        // normalized
        let wi = ri.dir;
        let sa = match self.material().bsdf_pdf(ho, &ro) {
            None => 0.0,
            Some(pdf) => pdf.value_for(&ri, matches!(mode, Transport::Importance)),
        };
        let ng = next.h.ng;

        sa * wi.dot(ng).abs() / xi.distance_squared(xii)
    }
}
