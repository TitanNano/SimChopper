pub mod async_support;
pub mod logger;
mod numbers;

#[cfg(debug_assertions)]
use godot::builtin::{
    Aabb, Callable, Color, NodePath, PackedByteArray, PackedColorArray, PackedFloat32Array,
    PackedFloat64Array, PackedInt32Array, PackedInt64Array, PackedStringArray, PackedVector2Array,
    PackedVector3Array, PackedVector4Array, Plane, Projection, Quaternion, Rect2, Rect2i, Rid,
    Signal, StringName, Transform2D, Transform3D, VarArray, VarDictionary, Variant, VariantType,
    Vector2, Vector2i, Vector3i, Vector4, Vector4i,
};
use godot::builtin::{Basis, Vector3};
#[cfg(debug_assertions)]
use godot::classes::Object;
use godot::classes::{SceneTree, SceneTreeTimer};
use godot::obj::Gd;
#[cfg(debug_assertions)]
use godot::obj::NewAlloc;

pub use numbers::*;

/// Create a new ingame one-shot timer in seconds.
#[inline]
pub fn timer(tree: &mut Gd<SceneTree>, delay: f64) -> Gd<SceneTreeTimer> {
    tree.create_timer_ex(delay)
        .process_always(false)
        .ignore_time_scale(false)
        .process_in_physics(true)
        .done()
        .unwrap()
}

#[cfg(debug_assertions)]
#[inline]
pub fn variant_type_default_value(ty: VariantType) -> Variant {
    const MAX: i32 = VariantType::MAX.ord;

    match ty {
        VariantType::NIL => Variant::nil(),
        VariantType::BOOL => Variant::from(false),
        VariantType::INT => Variant::from(0),
        VariantType::FLOAT => Variant::from(0.0),
        VariantType::STRING => Variant::from(""),
        VariantType::VECTOR2 => Variant::from(Vector2::ZERO),
        VariantType::VECTOR2I => Variant::from(Vector2i::ZERO),
        VariantType::RECT2 => Variant::from(Rect2::default()),
        VariantType::RECT2I => Variant::from(Rect2i::default()),
        VariantType::VECTOR3 => Variant::from(Vector3::ZERO),
        VariantType::VECTOR3I => Variant::from(Vector3i::ZERO),
        VariantType::VECTOR4 => Variant::from(Vector4::ZERO),
        VariantType::VECTOR4I => Variant::from(Vector4i::ZERO),
        VariantType::TRANSFORM2D => Variant::from(Transform2D::default()),
        VariantType::PLANE => Variant::from(Plane::invalid()),
        VariantType::QUATERNION => Variant::from(Quaternion::default()),
        VariantType::AABB => Variant::from(Aabb::default()),
        VariantType::BASIS => Variant::from(Basis::default()),
        VariantType::TRANSFORM3D => Variant::from(Transform3D::IDENTITY),
        VariantType::PROJECTION => Variant::from(Projection::IDENTITY),
        VariantType::COLOR => Variant::from(Color::default()),
        VariantType::STRING_NAME => Variant::from(StringName::default()),
        VariantType::NODE_PATH => Variant::from(NodePath::default()),
        VariantType::RID => Variant::from(Rid::Invalid),
        VariantType::OBJECT => Variant::from(Object::new_alloc()),
        VariantType::CALLABLE => Variant::from(Callable::invalid()),
        VariantType::SIGNAL => Variant::from(Signal::invalid()),
        VariantType::DICTIONARY => Variant::from(VarDictionary::new()),
        VariantType::ARRAY => Variant::from(VarArray::new()),
        VariantType::PACKED_BYTE_ARRAY => Variant::from(PackedByteArray::new()),
        VariantType::PACKED_INT32_ARRAY => Variant::from(PackedInt32Array::new()),
        VariantType::PACKED_INT64_ARRAY => Variant::from(PackedInt64Array::new()),
        VariantType::PACKED_FLOAT32_ARRAY => Variant::from(PackedFloat32Array::new()),
        VariantType::PACKED_FLOAT64_ARRAY => Variant::from(PackedFloat64Array::new()),
        VariantType::PACKED_STRING_ARRAY => Variant::from(PackedStringArray::new()),
        VariantType::PACKED_VECTOR2_ARRAY => Variant::from(PackedVector2Array::new()),
        VariantType::PACKED_VECTOR3_ARRAY => Variant::from(PackedVector3Array::new()),
        VariantType::PACKED_COLOR_ARRAY => Variant::from(PackedColorArray::new()),
        VariantType::PACKED_VECTOR4_ARRAY => Variant::from(PackedVector4Array::new()),

        VariantType {
            ord: MAX..=i32::MAX | i32::MIN..0,
        } => {
            unreachable!("variant type is out of defined range")
        }
    }
}

pub(crate) mod vector3 {
    use godot::builtin::Vector3;

    #[expect(unused)]
    pub const XY_PLANE: Vector3 = Vector3 {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    };

    pub const XZ_PLANE: Vector3 = Vector3 {
        x: 1.0,
        y: 0.0,
        z: 1.0,
    };

    #[expect(unused)]
    pub const YZ_PLANE: Vector3 = Vector3 {
        x: 0.0,
        y: 1.0,
        z: 1.0,
    };
}

#[inline]
pub(crate) fn basis_from_normal(normal: Vector3) -> Basis {
    Basis::from_cols(
        normal.cross(Basis::IDENTITY.col_c()),
        normal,
        Basis::IDENTITY.col_a().cross(normal),
    )
}

#[macro_export]
macro_rules! debug_3d {
    ($debugger: expr => $($variable: tt),+) => {
        #[cfg(debug_assertions)]
        if let Some(ref mut debugger) = $debugger {
            use $crate::scripts::objects::debugger_3_d::IDebugger3D;

            $(
                $crate::debug_3d!(inner debugger, $variable);
            )+
        }
    };

    (inner $debugger: ident, (float $variable: ident)) => {
        $debugger.debug_data().set(stringify!($variable), ($variable * 100.0).round() / 100.0);
    };

    (inner $debugger: ident, (as_deg $variable: ident)) => {
        $debugger.debug_data().set(stringify!($variable), $variable.to_degrees());
    };

    (inner $debugger: ident, $variable: ident) => {
        $debugger.debug_data().set(stringify!($variable), $variable);
    };
}
