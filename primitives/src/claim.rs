#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};

use codec::{ Encode, Decode };
use sp_std::{ convert::{ Into, From, TryInto},  vec::Vec };

#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum DataTypes {
}

impl Default for DataTypes {
    fn default() -> Self {
        DataTypes::VecU8
    }
}


#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub enum ClaimValue {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    Bool(bool),
    Data(Vec<u8>),
}

impl Default for ClaimValue {
    fn default() -> Self {
        ClaimValue::from(0u8)
    }
}

impl From<u8> for ClaimValue {
    fn from( v: u8) -> ClaimValue {
        ClaimValue::U8(v)
    }
}

impl ClaimValue {
    pub fn as_u8() -> Result<u8, ParseIntError>{
        match self {
            ClaimValue::U8(v) => Ok(v),
            _ =>
        }
    }
}

impl TryInto<u8> for ClaimValue {
    type Error = &'static str;

    fn try_into(self) -> Result<u8, Self::Error> {
        match self {
            ClaimValue::U8(v) => Ok(v),
            _ => Err("unsupported"),
        }
    }
}
