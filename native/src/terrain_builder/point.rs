use std::cell::RefCell;
use std::rc::Rc;

/// Type that has a X dimension
pub trait DimensionX {
    fn x(&self) -> f32;
}

impl<V: DimensionX> DimensionX for Rc<V> {
    fn x(&self) -> f32 {
        (**self).x()
    }
}

impl<V: DimensionX> DimensionX for RefCell<V> {
    fn x(&self) -> f32 {
        self.borrow().x()
    }
}

/// Type that has a Z dimension
pub trait DimensionZ {
    fn z(&self) -> f32;
}

impl<V: DimensionZ> DimensionZ for Rc<V> {
    fn z(&self) -> f32 {
        (**self).z()
    }
}

impl<V: DimensionZ> DimensionZ for RefCell<V> {
    fn z(&self) -> f32 {
        self.borrow().z()
    }
}

/// Type with a Y dimension
pub trait DimensionY {
    fn y(&self) -> f32;
}

impl<V: DimensionY> DimensionY for Rc<V> {
    fn y(&self) -> f32 {
        (**self).y()
    }
}

impl<V: DimensionY> DimensionY for RefCell<V> {
    fn y(&self) -> f32 {
        self.borrow().y()
    }
}

/// Type with a mutable Y dimension
pub trait SetDimensionY {
    fn set_y(self, value: f32);
}

impl<V> SetDimensionY for Rc<V>
where
    for<'a> &'a V: SetDimensionY,
{
    fn set_y(self, value: f32) {
        (*self).set_y(value)
    }
}

impl<V> SetDimensionY for &RefCell<V>
where
    for<'a> &'a mut V: SetDimensionY,
{
    fn set_y(self, value: f32) {
        self.borrow_mut().set_y(value);
    }
}

impl<V> SetDimensionY for RefCell<V>
where
    for<'a> &'a mut V: SetDimensionY,
{
    fn set_y(self, value: f32) {
        self.borrow_mut().set_y(value);
    }
}

/// Type that is a fixed point which should not move
/// from its current location during mesh optimizations.
pub trait FixedPoint {
    fn is_fixed(&self) -> bool;
}

impl<V: FixedPoint> FixedPoint for Rc<V> {
    fn is_fixed(&self) -> bool {
        (**self).is_fixed()
    }
}

impl<V: FixedPoint> FixedPoint for RefCell<V> {
    fn is_fixed(&self) -> bool {
        self.borrow().is_fixed()
    }
}
