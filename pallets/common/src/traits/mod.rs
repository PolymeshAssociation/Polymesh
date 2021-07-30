// This file is part of the Polymesh distribution (https://github.com/PolymathNetwork/Polymesh).
// Copyright (c) 2020 Polymath

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use frame_support::dispatch::DispatchResult;
use polymesh_primitives::{
    secondary_key::api::SecondaryKey, traits::BlockRewardsReserveCurrency, InvestorUid,
};

pub trait CommonConfig: frame_system::Config + permissions::Config {
    type AssetSubTraitTarget: asset::AssetSubTrait;

    type BlockRewardsReserve: BlockRewardsReserveCurrency<NegativeImbalance<Self>>;
}

pub mod imbalances;
pub use imbalances::{NegativeImbalance, PositiveImbalance};

pub mod asset;
pub mod balances;
pub mod checkpoint;
pub mod compliance_manager;
/*
pub mod contracts;
pub use contracts::ContractsFn;
*/
pub mod external_agents;
pub mod governance_group;
pub mod group;
pub mod identity;
pub mod multisig;
pub mod pip;
pub mod portfolio;
pub mod transaction_payment;
pub use transaction_payment::{CddAndFeeDetails, ChargeTxFee};
pub mod permissions;
pub use permissions::{AccountCallPermissionsData, CheckAccountCallPermissions};
pub mod relayer;
pub mod statistics;

pub trait TestUtilsFn<AccountId> {
    /// Creates a new did and attaches a CDD claim to it.
    fn register_did(
        target: AccountId,
        investor: InvestorUid,
        secondary_keys: sp_std::vec::Vec<SecondaryKey<AccountId>>,
    ) -> DispatchResult;
}

pub mod base {
    use frame_support::decl_event;
    use frame_support::dispatch::DispatchError;
    use frame_support::traits::Get;

    decl_event! {
        pub enum Event {
            /// An unexpected error happened that should be investigated.
            UnexpectedError(Option<DispatchError>),
        }
    }

    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type Event: From<Event> + Into<<Self as frame_system::Config>::Event>;

        /// The maximum length governing `TooLong`.
        ///
        /// How lengths are computed to compare against this value is situation based.
        /// For example, you could halve it, double it, compute a sum for some tree of strings, etc.
        type MaxLen: Get<u32>;
    }
}
