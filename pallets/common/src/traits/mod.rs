// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymesh Association

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use polymesh_primitives::traits::BlockRewardsReserveCurrency;

pub trait CommonConfig: frame_system::Config + pallet_permissions::Config {
    type BlockRewardsReserve: BlockRewardsReserveCurrency<NegativeImbalance<Self>>;
}

pub mod imbalances;
pub use imbalances::{NegativeImbalance, PositiveImbalance};

pub mod asset;
pub mod balances;
pub mod checkpoint;
pub mod compliance_manager;
pub mod external_agents;
pub mod governance_group;
pub mod group;
pub mod identity;
pub mod multisig;
pub mod nft;
pub mod portfolio;
pub mod transaction_payment;
pub use transaction_payment::CddAndFeeDetails;
pub mod relayer;
pub mod settlement;
pub mod statistics;
