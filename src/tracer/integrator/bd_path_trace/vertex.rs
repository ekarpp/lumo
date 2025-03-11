use super::*;

/// Abstraction of a vertex in the paths
#[derive(Clone)]
pub struct Vertex<'a> {
    pub h: Hit<'a>,
    /// `gathered` radiance/importance when arriving at `self`
    pub gathered: Color,
    /// PDF to arrive at `self` when starting from beginning of a path (in area measure)
    pub pdf_fwd: Float,
    /// PDF to arrive at `self` when starting from the end of a path (in area measure)
    pub pdf_bck: Float,
    /// Direction from previous vertex (if exists)
    pub wo: Direction,
    /// Light index, if we are a light vertex
    pub light: Option<usize>,
}

impl<'a> Vertex<'a> {
    /// Camera vertex
    pub fn camera(xo: Point, pdf_fwd: Float, gathered: Color) -> Self {
        let Some(h) = Hit::new(
            0.0,
            &Material::Blank,
            -Direction::X,
            xo,
            Vec3::ZERO,
            Normal::X,
            Normal::X,
            Vec2::X,
        ) else { unreachable!() };

        Self {
            h,
            gathered,
            pdf_bck: 0.0,
            pdf_fwd,
            light: None,
            wo: Direction::ZERO,
        }
    }

    /// Light vertex
    pub fn light(h: Hit<'a>, light: usize, gathered: Color, pdf_fwd: Float) -> Self {
        Self {
            h,
            gathered,
            light: Some(light),
            pdf_bck: 0.0,
            pdf_fwd,
            wo: Direction::ZERO,
        }
    }

    /// Surface vertex
    pub fn surface(
        wo: Direction,
        h: Hit<'a>,
        gathered: Color,
        pdf_sa: Float,
        prev: &Vertex,
    ) -> Self {
        let ho = &h;
        let hp = &prev.h;
        let xp = hp.p;
        let xo = ho.p;
        let pdf_fwd = if ho.material.is_delta() {
            0.0
        } else {
            let ng = if matches!(ho.material, Material::Volumetric(..)) {
                -wo
            } else {
                ho.ng
            };

            // at this
            measure::sa_to_area(pdf_sa, xp, xo, -wo, ng)
        };
        Self {
            h,
            gathered,
            pdf_fwd,
            light: None,
            pdf_bck: 0.0,
            wo,
        }
    }

    fn material(&self) -> &Material {
        self.h.material
    }

    /// Are we a surface/light vertex?
    pub fn is_surface(&self) -> bool {
        !matches!(self.material(), Material::Blank | Material::Volumetric(..))
    }

    /// Are we on a light?
    pub fn is_light(&self) -> bool {
        self.light.is_some()
    }

    /// Are we on a surface with delta material?
    pub fn is_delta(&self) -> bool {
        self.material().is_delta()
    }

    /// Helper to get emittance at hit
    pub fn emittance(&self, lambda: &ColorWavelength) -> Color {
        self.material().emit(lambda, &self.h)
    }

    /// Helper to get shading cosine at hit
    pub fn shading_cosine(&self, wi: Direction) -> Float {
        self.material().shading_cosine(wi, self.h.ns)
    }

    /// Correction term for non-symmetry of shading normals when transporting improtance
    pub fn shading_correction(&self, wi: Direction) -> Float {
        let m = self.material();
        let wo = self.wo;
        let ng = self.h.ng;
        let ns = self.h.ns;

        m.shading_cosine(wi, ng) * m.shading_cosine(wo, ns)
            / (m.shading_cosine(wo, ng) * m.shading_cosine(wi, ns))
    }

    /// Computes BSDF at hit of `self`
    pub fn f(&self, next: &Vertex, lambda: &ColorWavelength, mode: Transport) -> Color {
        let wi = (next.h.p - self.h.p).normalize();

        self.material().bsdf_f(self.wo, wi, lambda, mode, &self.h)
    }

    /// PDF w.r.t SA from BSDF at self with `wi` sampled
    pub fn bsdf_pdf(&self, wi: Direction, swap_dir: bool) -> Float {
        self.material().bsdf_pdf(self.wo, wi, &self.h, swap_dir)
    }

    /// PDF from `self` to `prev` with respect to surface area
    pub fn pdf_prev(&self, prev: &Vertex, wi: Direction) -> Float {
        if self.is_delta() || prev.is_delta() {
            0.0
        } else {
            let pdf_sa = self.bsdf_pdf(wi, true);
            // v.dot(v) = 1, cancels out the dot product for mediums
            let ngp = if !prev.is_surface() { -self.wo }  else { prev.h.ng };

            let xp = prev.h.p;
            let xo = self.h.p;
            // at prev
            measure::sa_to_area(pdf_sa, xo, xp, -self.wo, ngp)
        }
    }
}
