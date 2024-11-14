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

//! # Trait Interface to the Multisig Module
//!
//! The interface allows to process addition of a multisig signer from modules other than the
//! multisig module itself.

use frame_support::pallet_prelude::Weight;

pub trait WeightInfo {
    fn create_multisig(signers: u32) -> Weight;
    fn create_proposal() -> Weight;
    fn approve() -> Weight;
    fn execute_proposal() -> Weight;
    fn reject() -> Weight;
    fn accept_multisig_signer() -> Weight;
    fn add_multisig_signers(signers: u32) -> Weight;
    fn remove_multisig_signers(signers: u32) -> Weight;
    fn add_multisig_signers_via_admin(signers: u32) -> Weight;
    fn remove_multisig_signers_via_admin(signers: u32) -> Weight;
    fn change_sigs_required() -> Weight;
    fn change_sigs_required_via_admin() -> Weight;
    fn add_admin() -> Weight;
    fn remove_admin_via_admin() -> Weight;
    fn remove_payer() -> Weight;
    fn remove_payer_via_payer() -> Weight;
    fn create_join_identity() -> Weight;
    fn approve_join_identity() -> Weight;
    fn join_identity() -> Weight;
    fn remove_admin() -> Weight;

    fn default_max_weight(max_weight: &Option<Weight>) -> Weight {
        max_weight.unwrap_or_else(|| {
            // TODO: Use a better default weight.
            Self::create_proposal()
        })
    }

    fn approve_and_execute(max_weight: &Option<Weight>) -> Weight {
        Self::approve()
            .saturating_add(Self::execute_proposal())
            .saturating_add(Self::default_max_weight(max_weight))
    }
}

/// This trait is used to add a signer to a multisig and enable unlinking multisig from an identity
pub trait MultiSigSubTrait<AccountId> {
    /// Returns `true` if the given `account_id` is a multisign account, otherwise returns `false`.
    fn is_multisig(account_id: &AccountId) -> bool;
}
