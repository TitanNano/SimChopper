use godot::{
    builtin::{math::FloatExt, Basis, Transform3D, Vector3},
    engine::Node3D,
};

pub trait Node3DExt {
    fn align_up(&mut self, normal: Vector3);
}

pub trait Vector3Ext {
    fn align_up(self, normal: Vector3) -> Vector3;
}

impl Vector3Ext for Vector3 {
    fn align_up(self, normal: Vector3) -> Vector3 {
        let mut basis = Basis::default();

        basis.set_col_b(normal);
        basis.set_col_a({
            let z = -basis.col_a().cross(normal);
            let x = -basis.col_c().cross(normal);
            if z.length() > x.length() {
                z
            } else {
                x
            }
        });
        basis.set_col_c(basis.col_a().cross(basis.col_b()));

        if !basis.determinant().is_zero_approx() {
            basis = basis.orthonormalized();
        }

        let transform = Transform3D::new(basis, Vector3::ZERO);

        transform * self
    }
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
