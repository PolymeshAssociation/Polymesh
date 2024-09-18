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

//! Runtime API definition for the Statistics module.

use frame_support::dispatch::DispatchError;
use sp_std::vec::Vec;

use polymesh_primitives::asset::AssetID;
use polymesh_primitives::transfer_compliance::TransferCondition;
use polymesh_primitives::{Balance, IdentityId};

sp_api::decl_runtime_apis! {
    #[api_version(0)]
    pub trait StatisticsApi {
        /// Returns a vector containing all [`TransferCondition`] that are not being respected for the transfer. An empty vec means there's no error.
        fn transfer_restrictions_report(
            asset_id: AssetID,
            sender_did: &IdentityId,
            receiver_did: &IdentityId,
            transfer_amount: Balance,
        ) -> Result<Vec<TransferCondition>, DispatchError>;
    }
}
