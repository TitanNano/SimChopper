use godot::builtin::{
    Aabb, Basis, Callable, Color, Dictionary, NodePath, PackedByteArray, PackedColorArray,
    PackedFloat32Array, PackedFloat64Array, PackedInt32Array, PackedInt64Array, PackedStringArray,
    PackedVector2Array, PackedVector3Array, PackedVector4Array, Plane, Projection, Quaternion,
    Rect2, Rect2i, Rid, Signal, StringName, Transform2D, Transform3D, Variant, VariantArray,
    VariantType, Vector2, Vector2i, Vector3, Vector3i, Vector4, Vector4i,
};
use godot::classes::{Object, SceneTree, SceneTreeTimer};
use godot::obj::{Gd, NewAlloc};

pub mod async_support;
pub mod logger;

/// Create a new ingame one-shot timer in seconds.
pub fn timer(tree: &mut Gd<SceneTree>, delay: f64) -> Gd<SceneTreeTimer> {
    tree.create_timer_ex(delay)
        .process_always(false)
        .ignore_time_scale(false)
        .process_in_physics(true)
        .done()
        .unwrap()
}

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
        VariantType::DICTIONARY => Variant::from(Dictionary::new()),
        VariantType::ARRAY => Variant::from(VariantArray::new()),
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

        VariantType { ord: MAX.. } | VariantType { ord: i32::MIN..0 } => {
            unreachable!("variant type is out of defined range")
        }
    }
}
