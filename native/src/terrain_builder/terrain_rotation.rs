use gdnative::prelude::{NativeClass, Reference, methods, Instance, Shared};

const TERAIN_ROTATION_CORNERS: [u8; 4] = [0, 1, 3, 2];
const ERROR_CLASS_INSTANCE_ACCESS: &str = "unable to access NativeClass instance!";

#[derive(NativeClass)]
#[inherit(Reference)]
pub struct TerrainRotation {
    offset: u8,
}

#[methods]
impl TerrainRotation {
    fn new(_base: &Reference) -> Self {
        Self { offset: 0 }
    }

    #[export]
    fn set_rotation(&mut self, _base: &Reference, rotation: i64) {
        self.offset = u8::try_from(rotation).unwrap_or(u8::MAX);
    }
}

pub trait TerrainRotationBehaviour {
    fn get_corner(&self, index: u8) -> u8;

    fn nw(&self) -> usize;
    fn ne(&self) -> usize;
    fn se(&self) -> usize;
    fn sw(&self) -> usize;
}

impl TerrainRotationBehaviour for TerrainRotation {
    fn get_corner(&self, index: u8) -> u8 {
        let shifted_index = ((index + self.offset) % 4) as usize;
        let target_value = TERAIN_ROTATION_CORNERS.get(shifted_index).unwrap_or(&0);

        return target_value.to_owned();
    }

    fn nw(&self) -> usize {
        self.get_corner(0).into()
    }

    fn ne(&self) -> usize {
        return self.get_corner(1).into();
    }

    fn se(&self) -> usize {
        return self.get_corner(2).into();
    }

    fn sw(&self) -> usize {
        return self.get_corner(3).into();
    }
}

impl TerrainRotationBehaviour for Instance<TerrainRotation, Shared> {
    fn get_corner(&self, index: u8) -> u8 {
        let inst_ref = unsafe { self.assume_safe() };

        inst_ref
            .map(|object, _base| object.get_corner(index))
            .expect(ERROR_CLASS_INSTANCE_ACCESS)
    }

    fn nw(&self) -> usize {
        let inst_ref = unsafe { self.assume_safe() };

        inst_ref
            .map(|object, _base| object.nw())
            .expect(ERROR_CLASS_INSTANCE_ACCESS)
    }

    fn ne(&self) -> usize {
        let inst_ref = unsafe { self.assume_safe() };

        inst_ref
            .map(|object, _base| object.ne())
            .expect(ERROR_CLASS_INSTANCE_ACCESS)
    }

    fn se(&self) -> usize {
        let inst_ref = unsafe { self.assume_safe() };

        inst_ref
            .map(|object, _base| object.se())
            .expect(ERROR_CLASS_INSTANCE_ACCESS)
    }

    fn sw(&self) -> usize {
        let inst_ref = unsafe { self.assume_safe() };

        inst_ref
            .map(|object, _base| object.sw())
            .expect(ERROR_CLASS_INSTANCE_ACCESS)
    }
}
