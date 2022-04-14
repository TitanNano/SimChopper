use std::cell::RefCell;
use std::rc::Rc;

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

pub trait FixedPoint {
    fn is_fixed(&self) -> bool;
}
