use godot::builtin::math::FloatExt;
use godot::builtin::Vector3;
use godot::classes::Node3D;

pub trait Node3DExt {
    fn align_up(&mut self, normal: Vector3);
}

impl Node3DExt for Node3D {
    fn align_up(&mut self, normal: Vector3) {
        let mut node_transform = self.get_global_transform();
        let node_basis = &mut node_transform.basis;

        node_basis.set_col_b(normal);
        node_basis.set_col_a({
            let z = -node_basis.col_a().cross(normal);
            let x = -node_basis.col_c().cross(normal);
            if z.length() > x.length() {
                z
            } else {
                x
            }
        });
        node_basis.set_col_c(node_basis.col_a().cross(node_basis.col_b()));

        if !node_basis.determinant().is_zero_approx() {
            *node_basis = node_basis.orthonormalized();
        }

        self.set_global_transform(node_transform);
    }
}
