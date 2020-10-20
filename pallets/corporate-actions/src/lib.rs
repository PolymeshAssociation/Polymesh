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

//! # Corporate Actions module.
//!
//! TODO
//!
//! ## Overview
//!
//! TODO
//!
//! ## Interface
//!
//! TODO
//!
//! ### Dispatchable Functions
//!
//! TODO
//!
//! ### Public Functions
//!
//! TODO
//!

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(bool_to_option)]

use codec::{Decode, Encode};
use core::convert::TryInto;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    weights::Weight,
};
use frame_system::ensure_root;
use pallet_asset as asset;
use pallet_identity as identity;
use polymesh_common_utilities::{
    balances::Trait as BalancesTrait,
    identity::{IdentityToCorporateAction, Trait as IdentityTrait},
    GC_DID,
};
use polymesh_primitives::{AuthorizationData, DocumentId, IdentityId, Moment, Ticker};
use polymesh_primitives_derive::VecU8StrongTyped;
use sp_arithmetic::Permill;
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::prelude::*;

/// Representation of a % to tax, with 10^6 precision.
pub type Tax = Permill;

/// How should `identities` in `TargetIdentities` be used?
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, Debug)]
pub enum TargetTreatment {
    /// Only those identities should be included.
    Include,
    /// All identities *but* those should be included.
    Exclude,
}

impl Default for TargetTreatment {
    fn default() -> Self {
        // By default, an empty list of identities to exclude means all identities are included.
        Self::Exclude
    }
}

/// A description of which identities that a CA will apply to.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Eq, Encode, Decode, Default, Debug)]
pub struct TargetIdentities {
    /// The specified identities either relevant or irrelevant, depending on `treatment`, for CAs.
    pub identities: Vec<IdentityId>,
    /// How should `identities` be treated?
    pub treatment: TargetTreatment,
}

impl TargetIdentities {
    /// Sort and deduplicate all identities.
    fn dedup(mut self) -> Self {
        self.identities.sort_unstable();
        self.identities.dedup();
        self
    }
}

/// The kind of a `CorporateAction`.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, Debug)]
pub enum CAKind {
    /// A predictable benefit.
    /// These are known at the time the asset is created.
    /// Examples include bonds and warrants.
    PredictableBenefit,
    /// An unpredictable benefit.
    /// These are announced during the *"life"* of the asset.
    /// Examples include dividends, bonus issues.
    UnpredictableBenfit,
    /// A notice to the position holders, where the goal is to dessiminate information to them,
    /// resulting in no change to the securities or cash position of the position holder.
    /// Examples include Annual General Meetings.
    IssuerNotice,
    /// A reorganization of the tokens.
    /// For example, for every 1 ACME token a holder owns, turn them into 2 tokens.
    /// These do not really change the position of holders, and is more of an accounting exercise.
    /// However, a reorganization does increase the supply of tokens, which could matter for indivisible ones.
    Reorganization,
    /// Some generic uncategorized CA.
    /// In other words, none of the above.
    Other,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Eq, Encode, Decode, Default, Debug, VecU8StrongTyped)]
pub struct CADetails(pub Vec<u8>);

/// Details of a generic CA.
/// The `(Ticker, ID)` denoting a unique identifier for the CA is stored as a key outside.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Eq, Encode, Decode, Debug)]
pub struct CorporateAction {
    /// The kind of CA that this is.
    pub kind: CAKind,
    /// Date at which any impact, if any, should be calculated.
    pub record_date: Option<Moment>,
    /// Free-form text up to a limit.
    pub details: CADetails,
    /// The identities this CA is relevant to.
    pub targets: TargetIdentities,
    /// The default withholding tax at the time of CA creation.
    /// For more on withholding tax, see the `DefaultWitholdingTax` storage item.
    pub default_withholding_tax: Tax,
    /// Any per-DID withholding tax overrides in relation to the default.
    pub withholding_tax: Vec<(IdentityId, Tax)>,
}

/// A `Ticker`-local CA ID.
/// By *local*, we mean that the same number might be used for a different `Ticker`
/// to uniquely identify a different CA.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, Default, Debug)]
pub struct LocalCAId(pub u32);

/// A unique global identifier for a CA.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, Debug)]
pub struct CAId {
    /// The `Ticker` component used to disambiguate the `local` one.
    pub ticker: Ticker,
    /// The per-`Ticker` local identifier.
    pub local_id: LocalCAId,
}

/// Weight abstraction for the corporate actions module.
pub trait WeightInfo {
    fn set_max_details_length() -> Weight;

    fn reset_caa() -> Weight;
    fn set_default_targets(num_targets: u32) -> Weight;
    fn set_default_withholding_tax() -> Weight;
    fn set_did_withholding_tax() -> Weight;

    fn initiate_corporate_action() -> Weight;
    fn link_ca_doc(num_docs: u32) -> Weight;
}

/// The module's configuration trait.
pub trait Trait: frame_system::Trait + BalancesTrait + IdentityTrait + asset::Trait {
    /// The overarching event type.
    type Event: From<Event> + Into<<Self as frame_system::Trait>::Event>;

    /// Weight information for extrinsics in the corporate actions pallet.
    type WeightInfo: WeightInfo;
}

type Identity<T> = identity::Module<T>;
type Asset<T> = asset::Module<T>;

decl_storage! {
    trait Store for Module<T: Trait> as CorporateAction {
        /// Determines the maximum number of bytes that the free-form `details` of a CA can store.
        ///
        /// Note that this is not the number of `char`s or the number of [graphemes].
        /// While this may be unnatural in terms of human understanding of a text's length,
        /// it more closely reflects actual storage costs (`'a'` is cheaper to store than an emoji).
        ///
        /// [graphemes]: https://en.wikipedia.org/wiki/Grapheme
        pub MaxDetailsLength get(fn max_details_length) config(): u32;

        /// A corporate action agent (CAA) of a ticker, if specified,
        /// that may be different from the asset owner (AO).
        /// If `None`, the AO is the CAA.
        ///
        /// The CAA may be distict from the AO because the CAA may require a money services license,
        /// and the assets would need to originate from the CAA's identity, not the AO's identity.
        ///
        /// (ticker => did?)
        pub Agent get(fn agent): map hasher(blake2_128_concat) Ticker => Option<IdentityId>;

        /// The identities targeted by default for CAs for this ticker,
        /// either to be excluded or included.
        ///
        /// (ticker => target identities)
        pub DefaultTargetIdentities get(fn default_target_identities): map hasher(blake2_128_concat) Ticker => TargetIdentities;

        /// The default amount of tax to withhold ("withholding tax", WT) for this ticker when distributing dividends.
        ///
        /// To understand withholding tax, e.g., let's assume that you hold ACME shares.
        /// ACME now decides to distribute 100 SEK to Alice.
        /// Alice lives in Sweden, so Skatteverket (the Swedish tax authority) wants 30% of that.
        /// Then those 100 * 30% are withheld from Alice, and ACME will send them to Skatteverket.
        ///
        /// (ticker => % to withhold)
        pub DefaultWitholdingTax get(fn default_withholding_tax): map hasher(blake2_128_concat) Ticker => Tax;

        /// The amount of tax to withhold ("withholding tax", WT) for a certain ticker x DID.
        /// If an entry exists for a certain DID, it overrides the default in `DefaultWithholdingTax`.
        ///
        /// (ticker => [(did, % to withhold)]
        pub DidWitholdingTax get(fn did_withholding_tax): map hasher(blake2_128_concat) Ticker => Vec<(IdentityId, Tax)>;

        /// The next per-`Ticker` CA ID in the sequence.
        /// The full ID is defined as a combination of `Ticker` and a number in this sequence.
        pub CAIdSequence get(fn ca_id_sequence): map hasher(blake2_128_concat) Ticker => LocalCAId;

        /// All recorded CAs thus far.
        /// Only generic information is stored here.
        /// Specific `CAKind`s, e.g., benefits and corporate ballots, may use additional on-chain storage.
        ///
        /// (ticker => local ID => the corporate action)
        pub CorporateActions get(fn corporate_actions):
            double_map hasher(blake2_128_concat) Ticker, hasher(blake2_128_concat) LocalCAId => Option<CorporateAction>;

        /// Associations from CAs to `Document`s via their IDs.
        /// (CAId => [DocumentId])
        ///
        /// The `CorporateActions` map stores `Ticker => LocalId => The CA`,
        /// so we can infer `Ticker => CAId`. Therefore, we don't need a double map.
        pub CADocLink get(fn ca_doc_link): map hasher(blake2_128_concat) CAId => Vec<DocumentId>;
    }
}

// Public interface for this runtime module.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        /// initialize the default event for this module
        fn deposit_event() = default;

        /// Set the max `length` of `details` in terms of bytes.
        /// May only be called via a PIP.
        #[weight = <T as Trait>::WeightInfo::set_max_details_length()]
        pub fn set_max_details_length(origin, length: u32) {
            ensure_root(origin)?;
            MaxDetailsLength::put(length);
            Self::deposit_event(Event::MaxDetailsLengthChanged(GC_DID, length));
        }

        /// Reset the CAA of `ticker` to its owner.
        ///
        /// ## Arguments
        /// - `ticker` for which the CAA is reset.
        ///
        /// ## Errors
        /// - `Unauthorized` if `origin` isn't `ticker`'s owner.
        #[weight = <T as Trait>::WeightInfo::reset_caa()]
        pub fn reset_caa(origin, ticker: Ticker) {
            let did = <Asset<T>>::ensure_perms_owner(origin, &ticker)?;
            Self::change_ca_agent(did, ticker, None);
        }

        /// Set the default CA `TargetIdentities` to `targets`.
        ///
        /// ## Arguments
        /// - `ticker` for which the default identities are changing.
        /// - `targets` the default target identities for a CA.
        ///
        /// ## Errors
        /// - `UnauthorizedAsAgent` if `origin` is not `ticker`'s sole CAA (owner is not necessarily the CAA).
        #[weight = <T as Trait>::WeightInfo::set_default_targets(targets.identities.len() as u32)]
        pub fn set_default_targets(origin, ticker: Ticker, targets: TargetIdentities) {
            let caa = Self::ensure_ca_agent(origin, ticker)?;

            // Dedup any DIDs in `targets` to optimize iteration later.
            let new = targets.dedup();

            // Commit + emit event.
            DefaultTargetIdentities::mutate(ticker, |slot| *slot = new.clone());
            Self::deposit_event(Event::DefaultTargetIdentitiesChanged(caa, ticker, new));
        }

        /// Set the default withholding tax for all DIDs and CAs relevant to this `ticker`.
        ///
        /// ## Arguments
        /// - `ticker` that the withholding tax will apply to.
        /// - `tax` that should be withheld when distributing dividends, etc.
        ///
        /// ## Errors
        /// - `UnauthorizedAsAgent` if `origin` is not `ticker`'s sole CAA (owner is not necessarily the CAA).
        #[weight = <T as Trait>::WeightInfo::set_default_withholding_tax()]
        pub fn set_default_withholding_tax(origin, ticker: Ticker, tax: Tax) {
            let caa = Self::ensure_ca_agent(origin, ticker)?;
            DefaultWitholdingTax::mutate(ticker, |slot| *slot = tax);
            Self::deposit_event(Event::DefaultWithholdingTaxChanged(caa, ticker, tax));
        }

        /// Set the withholding tax of `ticker` for `taxed_did` to `tax`.
        /// If `Some(tax)`, this overrides the default withholding tax of `ticker` to `tax` for `taxed_did`.
        /// Otherwise, if `None`, the default withholding tax will be used.
        ///
        /// ## Arguments
        /// - `ticker` that the withholding tax will apply to.
        /// - `taxed_did` that will have its withholding tax updated.
        /// - `tax` that should be withheld when distributing dividends, etc.
        ///
        /// ## Errors
        /// - `UnauthorizedAsAgent` if `origin` is not `ticker`'s sole CAA (owner is not necessarily the CAA).
        #[weight = <T as Trait>::WeightInfo::set_did_withholding_tax()]
        pub fn set_did_withholding_tax(origin, ticker: Ticker, taxed_did: IdentityId, tax: Option<Tax>) {
            let caa = Self::ensure_ca_agent(origin, ticker)?;
            DidWitholdingTax::mutate(ticker, |whts| {
                match (whts.iter().position(|(did, _)| did == &taxed_did), tax) {
                    (None, Some(tax)) => whts.push((taxed_did, tax)),
                    (Some(idx), None) => drop(whts.swap_remove(idx)),
                    (Some(idx), Some(tax)) => whts[idx] = (taxed_did, tax),
                    (None, None) => {}
                }
            });
            Self::deposit_event(Event::DidWithholdingTaxChanged(caa, ticker, taxed_did, tax));
        }

        /// Initiates a CA for `ticker` of `kind` with `details` and other provided arguments.
        ///
        /// ## Arguments
        /// - `ticker` that the CA is made for.
        /// - `kind` of CA being initiated.
        /// - `record_date`, if any, to calculate the impact of this CA.
        ///    If provided, this results in a scheduled balance snapshot ("checkpoint") at the date.
        /// - `details` of the CA in free-text form, up to a certain number of bytes in length.
        /// - `targets`, if any, which this CA is relevant/irrelevant to.
        ///    Overrides, if provided, the default at the asset level (`set_default_targets`).
        /// - `default_wt`, if any, is the default withholding tax to use for this CA.
        ///    Overrides, if provided, the default at the asset level (`set_default_withholding_tax`).
        /// - `wt`, if any, provides per-DID withholding tax overrides.
        ///    Overrides, if provided, the default at the asset level (`set_did_withholding_tax`).
        ///
        /// # Errors
        /// - `DetailsTooLong` if `details.len()` goes beyond `max_details_length`.
        /// - `UnauthorizedAsAgent` if `origin` is not `ticker`'s sole CAA (owner is not necessarily the CAA).
        /// - `LocalCAIdOverflow` in the unlikely event that so many CAs were created for this `ticker`,
        ///   that integer overflow would have occured if instead allowed.
        /// - `DuplicateDidTax` if a DID is included more than once in `wt`.
        #[weight = <T as Trait>::WeightInfo::initiate_corporate_action()]
        pub fn initiate_corporate_action(
            origin,
            ticker: Ticker,
            kind: CAKind,
            record_date: Option<Moment>,
            details: CADetails,
            targets: Option<TargetIdentities>,
            default_wt: Option<Tax>,
            wt: Option<Vec<(IdentityId, Tax)>>,
        ) {
            // Ensure that `details` is short enough.
            details
                .len()
                .try_into()
                .ok()
                .filter(|&len: &u32| len <= Self::max_details_length())
                .ok_or(Error::<T>::DetailsTooLong)?;

            // Ensure that CAA is calling.
            let caa = Self::ensure_ca_agent(origin, ticker)?;

            // Ensure that the next local CA ID doesn't overflow.
            let local_id = CAIdSequence::get(ticker);
            let next_id = local_id.0.checked_add(1).map(LocalCAId).ok_or(Error::<T>::LocalCAIdOverflow)?;
            let id = CAId { ticker, local_id };

            // Ensure there are no duplicates in withholding tax overrides.
            let mut wt = wt;
            if let Some(wt) = &mut wt {
                let before = wt.len();
                wt.sort_unstable_by_key(|&(did, _)| did);
                wt.dedup_by_key(|&mut (did, _)| did);
                ensure!(before == wt.len(), Error::<T>::DuplicateDidTax);
            }

            // Create a checkpoint at `record_date`, if any.
            if let Some(record_date) = record_date {
                // TODO(Centril): This immediately creates a checkpoint, but we want to schedule.
                // However, checkpoint scheduling needs some changes to support CAs.
                // Also, we need to consider checkpoint ID reservation.
                <Asset<T>>::_create_checkpoint_emit(ticker, record_date, caa)?;
            }

            // Commit the next local CA ID.
            CAIdSequence::insert(ticker, next_id);

            // Use asset level defaults if data not provided here.
            let targets = targets
                .map(|t| t.dedup())
                .unwrap_or_else(|| Self::default_target_identities(ticker));
            let dwt = default_wt.unwrap_or_else(|| Self::default_withholding_tax(ticker));
            let wt = wt.unwrap_or_else(|| Self::did_withholding_tax(ticker));

            // Commit CA to storage.
            CorporateActions::insert(ticker, id.local_id, CorporateAction {
                kind,
                record_date,
                details: details.clone(),
                targets: targets.clone(),
                default_withholding_tax: dwt,
                withholding_tax: wt.clone(),
            });

            // Emit event.
            Self::deposit_event(Event::CAInitiated(caa, id, kind, record_date, details, targets, dwt, wt));
        }

        /// Link the given CA `id` to the given `docs`.
        /// Any previous links for the CA are removed in favor of `docs`.
        ///
        /// The workflow here is to add the documents and initiating the CA in any order desired.
        /// Once both exist, they can now be linked together.
        ///
        /// ## Arguments
        /// - `id` of the CA to associate with `docs`.
        /// - `docs` to associate with the CA with `id`.
        ///
        /// # Errors
        /// - `UnauthorizedAsAgent` if `origin` is not `ticker`'s sole CAA (owner is not necessarily the CAA).
        /// - `NoSuchCA` if `id` does not identify an existing CA.
        /// - `NoSuchDoc` if any of `docs` does not identify an existing document.
        #[weight = <T as Trait>::WeightInfo::link_ca_doc(docs.len() as u32)]
        pub fn link_ca_doc(origin, id: CAId, docs: Vec<DocumentId>) {
            // Ensure that CAA is calling and that CA and the docs exists.
            let caa = Self::ensure_ca_agent(origin, id.ticker)?;
            Self::ensure_ca_exists(id)?;
            for doc in &docs {
                <Asset<T>>::ensure_doc_exists(&id.ticker, doc)?;
            }

            // Add the link and emit event.
            CADocLink::mutate(id, |slot| *slot = docs.clone());
            Self::deposit_event(Event::CALinkedToDoc(caa, id, docs));
        }
    }
}

decl_event! {
    pub enum Event {
        /// The maximum length of `details` in bytes was changed.
        /// (GC DID, new length)
        MaxDetailsLengthChanged(IdentityId, u32),
        /// The set of default `TargetIdentities` for a ticker changed.
        /// (CAA DID, Ticker, New TargetIdentities)
        DefaultTargetIdentitiesChanged(IdentityId, Ticker, TargetIdentities),
        /// The default withholding tax for a ticker changed.
        /// (CAA DID, Ticker, New Tax).
        DefaultWithholdingTaxChanged(IdentityId, Ticker, Tax),
        /// The withholding tax specific to a DID for a ticker changed.
        /// (CAA DID, Ticker, Taxed DID, New Tax).
        DidWithholdingTaxChanged(IdentityId, Ticker, IdentityId, Option<Tax>),
        /// A new DID was made the CAA.
        /// (New CAA DID, Ticker, New CAA DID).
        CAATransferred(IdentityId, Ticker, IdentityId),
        /// A CA was initiated.
        /// (CAA DID, CA id, kind of CA, record date, free-form text, targeted ids,
        /// withholding tax, per-did tax override)
        CAInitiated(
            IdentityId,
            CAId,
            CAKind,
            Option<Moment>,
            CADetails,
            TargetIdentities,
            Tax,
            Vec<(IdentityId, Tax)>,
        ),
        /// A CA was linked to a set of docs.
        /// (CAA, CA Id, List of doc identifiers)
        CALinkedToDoc(IdentityId, CAId, Vec<DocumentId>),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// The signer is not authorized to act as a CAA for this asset.
        UnauthorizedAsAgent,
        /// The authorization type is not to transfer the CAA to another DID.
        AuthNotCAATransfer,
        /// The `details` of a CA exceeded the max allowed length.
        DetailsTooLong,
        /// There have been too many CAs for this ticker and the ID would overflow.
        /// This won't occur in practice.
        LocalCAIdOverflow,
        /// A withholding tax override for a given DID was specified more than once.
        /// The chain refused to make a choice, and hence there was an error.
        DuplicateDidTax,
        /// A CA with the given `CAId` did not exist.
        NoSuchCA,
    }
}

impl<T: Trait> IdentityToCorporateAction for Module<T> {
    fn accept_corporate_action_agent_transfer(did: IdentityId, auth_id: u64) -> DispatchResult {
        // Ensure we have authorization to transfer to `did`...
        let auth = <Identity<T>>::ensure_authorization(&did.into(), auth_id)?;
        let ticker = match auth.authorization_data {
            AuthorizationData::TransferCorporateActionAgent(ticker) => ticker,
            _ => return Err(Error::<T>::AuthNotCAATransfer.into()),
        };
        <Asset<T>>::consume_auth_by_owner(&ticker, did, auth_id)?;
        // ..and then transfer.
        Self::change_ca_agent(did, ticker, Some(did));
        Ok(())
    }
}

impl<T: Trait> Module<T> {
    /// Ensure that a CA with `id` exists, erroring otherwise.
    fn ensure_ca_exists(id: CAId) -> DispatchResult {
        ensure!(
            CorporateActions::contains_key(id.ticker, id.local_id),
            Error::<T>::NoSuchCA
        );
        Ok(())
    }

    /// Change `ticker`'s CAA to `new_caa` and emit an event.
    fn change_ca_agent(did: IdentityId, ticker: Ticker, new_caa: Option<IdentityId>) {
        // Transfer CAA status to `did`.
        Agent::mutate(ticker, |caa| *caa = new_caa);

        // Emit event.
        let new_caa = new_caa.unwrap_or_else(|| <Asset<T>>::token_details(ticker).owner_did);
        Self::deposit_event(Event::CAATransferred(did, ticker, new_caa));
    }

    /// Ensure that `origin` is authorized as a CA agent of the asset `ticker`.
    /// When `origin` is unsigned, `BadOrigin` occurs.
    /// Otherwise, should the DID not be the CAA of `ticker`, `UnauthorizedAsAgent` occurs.
    fn ensure_ca_agent(origin: T::Origin, ticker: Ticker) -> Result<IdentityId, DispatchError> {
        let did = <Identity<T>>::ensure_perms(origin)?;
        ensure!(
            Self::agent(ticker)
                .map_or_else(|| <Asset<T>>::is_owner(&ticker, did), |caa| caa == did),
            Error::<T>::UnauthorizedAsAgent
        );
        Ok(did)
    }
}
