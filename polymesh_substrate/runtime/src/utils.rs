use parity_codec::Codec;
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
    fn as_u128(v: Self::TokenBalance) -> u128;
    fn as_tb(v: u128) -> Self::TokenBalance;
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
