use gdnative::prelude::Vector3;
use lerp::Lerp;

fn bilerp<T: Lerp<F> + Copy, F: Copy>(points: [T; 4], weight_x: F, weight_y: F) -> T {
    let x = points[0].lerp(points[1], weight_x);
    let y = points[2].lerp(points[3], weight_x);

    x.lerp(y, weight_y)
}

pub fn bilerp_xyz(points: &[Vector3; 4], x: f32, y: f32) -> Vector3 {
    let target_x = bilerp(points.map(|v| v.x), x, y);
    let target_y = bilerp(points.map(|v| v.y), x, y);
    let target_z = bilerp(points.map(|v| v.z), x, y);

    Vector3::new(target_x, target_y, target_z)
}
