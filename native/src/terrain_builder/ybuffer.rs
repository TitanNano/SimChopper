use std::collections::HashMap;

use super::point::{DimensionX, DimensionY, DimensionZ, FixedPoint, SetDimensionY};

pub trait YBuffer<Value: DimensionX + DimensionZ + DimensionY + FixedPoint + SetDimensionY>:
    Sized
{
    fn add(&mut self, value: Value);
    fn new() -> Self;
    fn into_iter_groups(self) -> Box<dyn Iterator<Item = Vec<Value>>>;

    fn reduce(self) {
        for vertex_group in self.into_iter_groups() {
            let count: usize = vertex_group.len();
            let peak_y = vertex_group
                .iter()
                .filter(|v| v.is_fixed())
                .map(|v| v.y())
                .reduce(f32::max)
                .unwrap_or(0.0);

            let average_y = if peak_y > 0.0 {
                peak_y
            } else {
                let total_y: f32 = vertex_group.iter().map(|v| v.y()).sum();

                total_y / (count as f32)
            };

            for vertex in vertex_group {
                vertex.set_y(average_y);
            }
        }
    }
}

pub type HashMapYBuffer<Value> = HashMap<(usize, usize), Vec<Value>>;

impl<V: 'static + DimensionX + DimensionZ + DimensionY + FixedPoint + SetDimensionY> YBuffer<V>
    for HashMapYBuffer<V>
{
    fn add(&mut self, value: V) {
        let xz = (value.x().round() as usize, value.z().round() as usize);

        self.entry(xz).or_insert_with(Vec::new).push(value);
    }

    fn new() -> Self {
        Self::new()
    }

    fn into_iter_groups(self) -> Box<dyn Iterator<Item = Vec<V>>> {
        Box::new(self.into_values())
    }
}
