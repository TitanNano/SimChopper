use godot::{
    meta::{FromGodot, ToGodot},
    prelude::GodotConvert,
};
use godot_rust_script::{GetScriptProperty, GodotScriptExport, SetScriptProperty};

#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Debug)]
pub struct LInt<const LOWER: u32, const UPPER: u32>(u32);

macro_rules! check_bounds {
    ($value: expr, $lower: ident, $upper: ident) => {
        debug_assert!($value >= $lower);

        #[cfg(not(debug_assertions))]
        if $value < $lower {
            return Self::MIN;
        }

        debug_assert!($value <= $upper);

        #[cfg(not(debug_assertions))]
        if $value > $upper {
            return Self::MAX;
        }
    };
}

impl<const LOWER: u32, const UPPER: u32> LInt<LOWER, UPPER> {
    pub const MAX: Self = Self::new(UPPER);

    pub const MIN: Self = Self::new(LOWER);

    pub const fn new(value: u32) -> Self {
        assert!(LOWER < UPPER);

        assert!(value >= LOWER);
        assert!(value <= UPPER);

        Self(value)
    }

    pub const fn add(self, rhs: Self) -> Self {
        let raw = self.0 + rhs.0;

        check_bounds!(raw, LOWER, UPPER);

        Self::new(raw)
    }

    pub const fn mul(self, rhs: Self) -> Self {
        let raw = self.0 * rhs.0;

        check_bounds!(raw, LOWER, UPPER);

        Self::new(raw)
    }

    pub const fn div(self, rhs: Self) -> Self {
        let raw = self.0 / rhs.0;

        check_bounds!(raw, LOWER, UPPER);

        Self::new(raw)
    }

    pub const fn sub(self, rhs: Self) -> Self {
        let raw = self.0 - rhs.0;

        check_bounds!(raw, LOWER, UPPER);

        Self::new(raw)
    }

    pub const fn rem(self, rhs: Self) -> Self {
        let raw = self.0 % rhs.0;

        check_bounds!(raw, LOWER, UPPER);

        Self::new(raw)
    }

    pub const fn into_u32(self) -> u32 {
        self.0
    }

    pub const fn into_u64(self) -> u64 {
        self.0 as u64
    }
}

impl<const UPPER: u32, const LOWER: u32> std::ops::Add for LInt<LOWER, UPPER> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.add(rhs)
    }
}

impl<const UPPER: u32, const LOWER: u32> std::ops::Mul for LInt<LOWER, UPPER> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        self.mul(rhs)
    }
}

impl<const UPPER: u32, const LOWER: u32> std::ops::Div for LInt<LOWER, UPPER> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        self.div(rhs)
    }
}

impl<const UPPER: u32, const LOWER: u32> std::ops::Sub for LInt<LOWER, UPPER> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.sub(rhs)
    }
}

impl<const UPPER: u32, const LOWER: u32> std::ops::Rem for LInt<LOWER, UPPER> {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        self.rem(rhs)
    }
}

impl<const UPPER: u32, const LOWER: u32> Default for LInt<LOWER, UPPER> {
    fn default() -> Self {
        Self::MIN
    }
}

impl<const UPPER: u32, const LOWER: u32> From<LInt<LOWER, UPPER>> for u32 {
    fn from(value: LInt<LOWER, UPPER>) -> Self {
        value.into_u32()
    }
}

impl<const UPPER: u32, const LOWER: u32> From<LInt<LOWER, UPPER>> for u64 {
    fn from(value: LInt<LOWER, UPPER>) -> Self {
        value.into_u64()
    }
}

impl<const UPPER: u32, const LOWER: u32> GodotScriptExport for LInt<LOWER, UPPER> {
    fn hint_string(
        custom_hint: Option<godot::global::PropertyHint>,
        custom_string: Option<String>,
    ) -> String {
        <u32 as GodotScriptExport>::hint_string(custom_hint, custom_string)
    }

    fn hint(custom: Option<godot::global::PropertyHint>) -> godot::global::PropertyHint {
        u32::hint(custom)
    }
}

impl<const UPPER: u32, const LOWER: u32> GodotConvert for LInt<LOWER, UPPER> {
    type Via = u32;
}

impl<const UPPER: u32, const LOWER: u32> SetScriptProperty for LInt<LOWER, UPPER> {
    fn set_property(&mut self, value: Self::Via) {
        *self = Self::new(value);
    }
}

impl<const UPPER: u32, const LOWER: u32> GetScriptProperty for LInt<LOWER, UPPER> {
    fn get_property(&self) -> Self::Via {
        self.into_u32()
    }
}

impl<const UPPER: u32, const LOWER: u32> FromGodot for LInt<LOWER, UPPER> {
    fn try_from_godot(via: Self::Via) -> Result<Self, godot::prelude::ConvertError> {
        Ok(Self::new(via))
    }
}

impl<const UPPER: u32, const LOWER: u32> ToGodot for LInt<LOWER, UPPER> {
    type ToVia<'v>
        = Self::Via
    where
        Self: 'v;

    fn to_godot(&self) -> Self::ToVia<'_> {
        self.into_u32()
    }
}

/// F32 has a mantissa of 23-bits
const F32_MAX: u32 = 2u32.pow(24);

pub type Uf32 = LInt<0, F32_MAX>;

impl Uf32 {
    #[expect(clippy::cast_precision_loss)]
    pub const fn into_f32(self) -> f32 {
        self.0 as f32
    }

    pub const fn into_f64(self) -> f64 {
        self.into_f32() as f64
    }
}

impl From<Uf32> for f32 {
    fn from(value: Uf32) -> Self {
        value.into_f32()
    }
}

impl From<Uf32> for f64 {
    fn from(value: Uf32) -> Self {
        value.into_f64()
    }
}

#[cfg(test)]
mod test {
    use num::ToPrimitive;

    #[test]
    #[expect(clippy::float_cmp)]
    fn create_instances() {
        const VALUE: super::Uf32 = super::Uf32::new(10);
        assert_eq!(VALUE.into_f32(), 10.0);

        assert_eq!(
            super::Uf32::MAX.into_f32().to_u32().unwrap(),
            super::F32_MAX
        );
    }
}
