use parity_codec::Codec;
use support::{Parameter, StorageMap, StorageValue, decl_storage, decl_module, decl_event, ensure};
use runtime_primitives::traits::{CheckedSub, CheckedAdd, Member, SimpleArithmetic, As};
use system::{self, ensure_signed};

/// The module's configuration trait.
pub trait Trait: system::Trait {
    type TokenBalance: Parameter + Member + SimpleArithmetic + Codec + Default + Copy + As<usize> + As<u64>;
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
