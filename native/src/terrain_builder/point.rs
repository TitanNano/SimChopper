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

/// Type with a Y dimension
pub trait DimensionY {
    fn y(&self) -> f32;
}

impl<V: DimensionY> DimensionY for Arc<V> {
    fn y(&self) -> f32 {
        (**self).y()
    }
}

impl<V: DimensionY> DimensionY for RwLock<V> {
    fn y(&self) -> f32 {
        self.read().expect(MUTEX_LOCK_ERROR).y()
    }
}

/// Type with a mutable Y dimension
pub trait SetDimensionY {
    fn set_y(self, value: f32);
}

impl<V> SetDimensionY for Arc<V>
where
    for<'a> &'a V: SetDimensionY,
{
    fn set_y(self, value: f32) {
        (*self).set_y(value)
    }
}

impl<V> SetDimensionY for &RwLock<V>
where
    for<'a> &'a mut V: SetDimensionY,
{
    fn set_y(self, value: f32) {
        self.write().expect(MUTEX_LOCK_ERROR).set_y(value);
    }
}

impl<V> SetDimensionY for RwLock<V>
where
    for<'a> &'a mut V: SetDimensionY,
{
    fn set_y(self, value: f32) {
        self.write().expect(MUTEX_LOCK_ERROR).set_y(value);
    }
}

/// Type that is a fixed point which should not move
/// from its current location during mesh optimizations.
pub trait FixedPoint {
    fn is_fixed(&self) -> bool;
}

impl<V: FixedPoint> FixedPoint for Arc<V> {
    fn is_fixed(&self) -> bool {
        (**self).is_fixed()
    }
}

impl<V: FixedPoint> FixedPoint for RwLock<V> {
    fn is_fixed(&self) -> bool {
        self.read().expect(MUTEX_LOCK_ERROR).is_fixed()
    }
}
