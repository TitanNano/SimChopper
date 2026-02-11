use std::sync::{Arc, RwLock};

const MUTEX_LOCK_ERROR: &str = "mutex apears to be poisoned";

/// Type that has a X dimension
pub trait DimensionX {
    fn x(&self) -> f32;
}

impl<V: DimensionX> DimensionX for Arc<V> {
    fn x(&self) -> f32 {
        (**self).x()
    }
}

impl<V: DimensionX> DimensionX for RwLock<V> {
    fn x(&self) -> f32 {
        self.read().expect(MUTEX_LOCK_ERROR).x()
    }
}

/// Type that has a Z dimension
pub trait DimensionZ {
    fn z(&self) -> f32;
}

impl<V: DimensionZ> DimensionZ for Arc<V> {
    fn z(&self) -> f32 {
        (**self).z()
    }
}

impl<V: DimensionZ> DimensionZ for RwLock<V> {
    fn z(&self) -> f32 {
        self.read().expect(MUTEX_LOCK_ERROR).z()
    }
}
