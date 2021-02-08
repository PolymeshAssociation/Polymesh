#![cfg_attr(not(feature = "std"), no_std)]

// With "testnet" feature.

#[cfg(feature = "testnet")]
mod testnet;

#[cfg(feature = "testnet")]
pub use testnet::{WeightInfo, *};

#[cfg(all(feature = "runtime-benchmarks", feature = "testnet"))]
pub mod benchmarking;

// Without "testnet" feature, the `empty_module` will be used.

#[cfg(not(feature = "testnet"))]
pub use polymesh_common_utilities::empty_module::*;
