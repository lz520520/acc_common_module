use crate::ParamMeta;

#[derive(Clone)]
pub enum ParamValue {
    ParamList(Vec<ParamMeta>),
    Params(ParamMeta),
    Str(String),
    Bytes(Vec<u8>),
    Bool(bool),

    Uint64(u64),
    Uint32(u32),
    Uint16(u16),
    Uint8(u8),
    // Uint(usize),
    Int64(i64),
    Int32(i32),
    Int16(i16),
    Int8(i8),
    // Int(isize),

    Float64(f64),
    Float32(f32)
}

macro_rules! impl_from_param_value {
    ($variant:ident, $t:ty) => {
        impl From<$t> for ParamValue {
            fn from(v: $t) -> Self {
                ParamValue::$variant(v)
            }
        }
    };
}

macro_rules! impl_param_value_try_into {
    ($target:ty, $variant:ident) => {
        impl TryInto<$target> for ParamValue {
            type Error = String;

            fn try_into(self) -> Result<$target, Self::Error> {
                if let ParamValue::$variant(v) = self {
                    Ok(v)
                } else {
                    Err(obfstr::obfstring!(concat!(
                        "type error: expected ",
                        stringify!($variant)
                    )))
                }
            }
        }
    };
}

impl_from_param_value!(ParamList, Vec<ParamMeta>);
impl_from_param_value!(Params, ParamMeta);
impl_from_param_value!(Str, String);
impl_from_param_value!(Bytes, Vec<u8>);
impl_from_param_value!(Bool, bool);

impl_from_param_value!(Uint64, u64);
impl_from_param_value!(Uint32, u32);
impl_from_param_value!(Uint16, u16);
impl_from_param_value!(Uint8, u8);

impl_from_param_value!(Int64, i64);
impl_from_param_value!(Int32, i32);
impl_from_param_value!(Int16, i16);
impl_from_param_value!(Int8, i8);

impl_from_param_value!(Float64, f64);
impl_from_param_value!(Float32, f32);
impl From<&str> for ParamValue {
    fn from(v: &str) -> Self {
        ParamValue::Str(v.to_string())
    }
}

impl_param_value_try_into!(bool, Bool);
impl_param_value_try_into!(String, Str);
impl_param_value_try_into!(Vec<u8>, Bytes);

impl_param_value_try_into!(u64, Uint64);
impl_param_value_try_into!(u32, Uint32);
impl_param_value_try_into!(u16, Uint16);
impl_param_value_try_into!(u8, Uint8);

impl_param_value_try_into!(i64, Int64);
impl_param_value_try_into!(i32, Int32);
impl_param_value_try_into!(i16, Int16);
impl_param_value_try_into!(i8, Int8);

impl_param_value_try_into!(f64, Float64);
impl_param_value_try_into!(f32, Float32);

impl_param_value_try_into!(ParamMeta, Params);
impl_param_value_try_into!(Vec<ParamMeta>, ParamList);
