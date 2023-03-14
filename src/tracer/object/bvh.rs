use super::*;

pub struct BoundingVolumeHierarchy {
    //left: Option<dyn Object>,
    //right: Option<dyn Object>,
    boxx: AxisAlignedBoundingBox,
}

// refactor scattered ray away and return tuple with pdf. for light sample too.
