#![cfg_attr(not(feature = "std"), no_std)]

pub mod traits {
    pub mod checkpoint;
    pub mod identity;
}
pub use traits::*;

pub mod protocol_fee;
