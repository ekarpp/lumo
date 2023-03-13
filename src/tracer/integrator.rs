#![allow(dead_code)]
use crate::DVec3;
use crate::rand_utils;
use crate::consts::{PATH_TRACE_RR, PATH_TRACE_MAX_DEPTH};
use crate::pdfs::{Pdf, ObjectPdf, CosPdf};
use crate::tracer::hit::Hit;
use crate::tracer::material::Material;
use crate::tracer::ray::{Ray, ScatterRay};
use crate::tracer::scene::Scene;

mod path_trace;
mod direct_light;

pub enum Integrator {
    PathTrace,
    DirectLight,
}

impl Integrator {
    pub fn integrate(&self, s: &Scene, r: &Ray) -> DVec3 {
        match self {
            Integrator::PathTrace => path_trace::integrate(s, r, 0, true),
            Integrator::DirectLight => direct_light::integrate(s, r, 0),
        }
    }
}
