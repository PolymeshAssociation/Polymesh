use parity_codec::Codec;
use rstd::prelude::*;
use runtime_primitives::traits::{As, CheckedAdd, CheckedSub, Member, SimpleArithmetic};
use support::{decl_event, decl_module, decl_storage, ensure, Parameter, StorageMap, StorageValue};
use system::{self, ensure_signed};

/// The module's configuration trait.
pub trait Trait: system::Trait + balances::Trait {
    type TokenBalance: Parameter
        + Member
        + SimpleArithmetic
        + Codec
        + Default
        + Copy
        + As<usize>
        + As<u64>
        + As<<Self as balances::Trait>::Balance>;
}

decl_storage! {
    trait Store for Module<T: Trait> as Utils {

    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

    }
}

// Other utility functions
#[inline]
/// Convert all letter characters of a slice to their upper case counterparts.
pub fn bytes_to_upper(v: &[u8]) -> Vec<u8> {
    v.iter()
        .map(|chr| match chr {
            97..=122 => chr - 32,
            other => *other,
        })
        .collect()
}
