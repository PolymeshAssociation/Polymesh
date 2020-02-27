use polymesh_runtime_common::balances::Trait as BalancesTrait;

use codec::{Decode, Encode};
use frame_system;
use pallet_session;
use sp_runtime::traits::{IdentifyAccount, Member, Verify};
use sp_std::prelude::*;

/// The module's configuration trait.
pub trait Trait: frame_system::Trait + BalancesTrait + pallet_session::Trait {
    type Public: IdentifyAccount<AccountId = Self::AccountId>;
    type OffChainSignature: Verify<Signer = Self::Public> + Member + Decode + Encode;
    fn validator_id_to_account_id(
        v: <Self as pallet_session::Trait>::ValidatorId,
    ) -> Self::AccountId;
}
