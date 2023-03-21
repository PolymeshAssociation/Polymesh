// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
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
//! The corporate actions module provides functionality for handling corporate actions (CAs) on-chain.
//!
//! Any CA is associated with an asset,
//! so most module dispatchables must be called by an external agent with appropriate corporate actions permissions.
//!
//! The starting point of any CA begins with executing `initiate_corporate_action`,
//! provided with the associated ticker, what sort of CA it is, e.g., a notice or a benefit,
//! and when, if any, a checkpoint should be recorded, or was recorded, if an existing one is to be used.
//! Additonally, free-form details, serving as on-chain documentation, may be provided.
//!
//! A CA targets a set of identities (`TargetIdentities`), but this need not be every asset holder.
//! Instead, when initiating a CA,
//! the targets may be specified either by exhaustively specifying every identity to include.
//! This is achieved through `TargetTreatment::Include`.
//! Instead of specifying an exhaustive set,
//! a set of identities can be excluded from the universe of asset holders.
//! This can be achieved through `TargetTreatment::Exclude`.
//! If the target set for an asset is usually the same,
//! a default may be specified through `set_default_targets(targets)`.
//!
//! Finally, CAs which imply some sort of benefit may have a taxable element, e.g., due to capital gains tax.
//! Sometimes, the responsibiliy paying such tax falls to the asset issuer (or external agents),
//! To handle such circumstances, a portion of the benefits may be withheld.
//! This governed by specifying a withholding tax % on-chain.
//! The tax is first and foremost specified for every identity,
//! but may also be overriden for specific identities (e.g., for DIDs in different jurisdictions).
//! As with targets, if the taxes are usually the same for every CA,
//! asset-level defaults may also be specified with `set_default_withholding_tax`
//! and `set_did_withholding_tax`.
//!
//! After having created a CA and some asset documents,
//! such documents may also be linked to the CA.
//! To do so, `link_ca_doc(ca_id, docs)` can be called,
//! with the ID of the CA specified in `ca_id` as well the IDs of each document in `docs`.
//!
//! Beyond this module, two other modules exist dedicated to CAs. These are:
//!
//! - The corporate ballots module, with which e.g., annual general meetings can be conducted on-chain.
//! - The capital distributions module, with which e.g., dividends and other benefits may be distributed.
//!
//! For more details, consult the documentation in those modules.
//!
//! ## Overview
//!
//! The module provides functions for:
//!
//! - Configuring the max length of details (chain global configuration, through PIPs)
//! - Specifying asset level CA configuration for the target set and withholding tax.
//! - Initiating CAs.
//! - Linking existing asset documentation to an existing CA.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `set_max_details_length(origin, length)` sets the maximum `length` in bytes for the `details` of any CA.
//!    Must be called via the PIP process.
//! - `set_default_targets(origin, ticker, targets)` sets the default `targets`
//!    for all CAs associated with `ticker`.
//! - `set_default_withholding_tax(origin, ticker, tax)` sets the default withholding tax
//!    for every identity for all CAs associated with `ticker`.
//! - `set_did_withholding_tax(origin, ticker, taxed_did, tax)` sets a withholding tax
//!    for CAs associated with `ticker` and specific to `taxed_did` to `tax`,
//!    or resets the tax of `taxed_did` to the default if `tax` is `None`.
//! - `initiate_corporate_action(...)` initates a corporate action.
//! - `link_ca_doc(origin, id, docs)` is called by external agents to associate `docs` to the CA with `id`.
//! - `remove_ca(origin, id)` removes the CA identified by `id`.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

pub mod ballot;
pub mod distribution;

use codec::{Decode, Encode};
use distribution::WeightInfo as DistWeightInfoTrait;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::Get,
    weights::Weight,
};
use frame_system::ensure_root;
use pallet_asset::checkpoint::{self, SchedulePoints, ScheduleRefCount};
use pallet_base::try_next_post;
use pallet_identity::PermissionedCallOriginData;
use polymesh_common_utilities::{
    balances::Config as BalancesConfig, identity::Config as IdentityConfig, traits::asset,
    traits::checkpoint::ScheduleId, with_transaction, GC_DID,
};
use polymesh_primitives::{
    calendar::CheckpointId, impl_checked_inc, storage_migration_ver, Balance, DocumentId, EventDid,
    IdentityId, Moment, PortfolioNumber, Ticker,
};
use polymesh_primitives_derive::VecU8StrongTyped;
use scale_info::TypeInfo;
use sp_arithmetic::Permill;
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::prelude::*;

/// Representation of a % to tax, with 10^6 precision.
pub type Tax = Permill;

/// How should `identities` in `TargetIdentities` be used?
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, Debug)]
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

impl TargetTreatment {
    /// Is this the `Include` treatment?
    pub fn is_include(self) -> bool {
        match self {
            Self::Include => true,
            Self::Exclude => false,
        }
    }
}

/// A description of which identities that a CA will apply to.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Eq, Encode, Decode, TypeInfo, Default, Debug)]
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

    /// Does this target `did`?
    /// Complexity: O(log n) with `n` being the number of identities listed.
    pub fn targets(&self, did: &IdentityId) -> bool {
        // N.B. The binary search here is OK since the list of identities is sorted.
        self.treatment.is_include() == self.identities.binary_search(&did).is_ok()
    }
}

/// The kind of a `CorporateAction`.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, TypeInfo, Debug)]
pub enum CAKind {
    /// A predictable benefit.
    /// These are known at the time the asset is created.
    /// Examples include bonds and warrants.
    PredictableBenefit,
    /// An unpredictable benefit.
    /// These are announced during the *"life"* of the asset.
    /// Examples include dividends, bonus issues.
    UnpredictableBenefit,
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

impl CAKind {
    /// Is this some sort of benefit CA?
    pub fn is_benefit(&self) -> bool {
        matches!(self, Self::PredictableBenefit | Self::UnpredictableBenefit)
    }
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, TypeInfo, VecU8StrongTyped)]
#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct CADetails(pub Vec<u8>);

/// Defines how to identify a CA's associated checkpoint, if any.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, TypeInfo, Debug)]
pub enum CACheckpoint {
    /// CA uses a record date scheduled to occur in the future.
    /// Checkpoint ID will be taken after the record date.
    ///
    /// Since a schedule can be recurring,
    /// the `u64` stores the number of checkpoints before the CA was made.
    /// This allows indexing into the list of CPs, getting exactly the right one.
    Scheduled(ScheduleId, u64),
    /// CA uses an existing checkpoint ID which was recorded in the past.
    Existing(CheckpointId),
}

/// Defines the record date, at which impact should be calculated,
/// along with checkpoint info to assess the impact at the date.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, TypeInfo, Debug)]
pub struct RecordDate {
    /// When the impact should be calculated, or already has.
    pub date: Moment,
    /// Info used to determine the `CheckpointId` once `date` has passed.
    pub checkpoint: CACheckpoint,
}

/// Input specification of the record date used to derive impact for a CA.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, TypeInfo, Debug)]
pub enum RecordDateSpec {
    /// Record date is in the future.
    /// A checkpoint should be created.
    Scheduled(Moment),
    /// A schedule already exists, infer record date from it.
    ExistingSchedule(ScheduleId),
    /// Checkpoint already exists, infer record date instead.
    Existing(CheckpointId),
}

/// Details of a generic CA.
/// The `(Ticker, ID)` denoting a unique identifier for the CA is stored as a key outside.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Eq, Encode, Decode, TypeInfo, Debug)]
pub struct CorporateAction {
    /// The kind of CA that this is.
    pub kind: CAKind,
    /// When the CA was declared off-chain.
    pub decl_date: Moment,
    /// Date at which any impact, if any, should be calculated.
    pub record_date: Option<RecordDate>,
    /// The identities this CA is relevant to.
    pub targets: TargetIdentities,
    /// The default withholding tax at the time of CA creation.
    /// For more on withholding tax, see the `DefaultWithholdingTax` storage item.
    pub default_withholding_tax: Tax,
    /// Any per-DID withholding tax overrides in relation to the default.
    pub withholding_tax: Vec<(IdentityId, Tax)>,
}

impl CorporateAction {
    /// Returns the tax of `did` in this CA.
    fn tax_of(&self, did: &IdentityId) -> Tax {
        // N.B. we maintain a sorted list to enable O(log n) access here.
        self.withholding_tax
            .binary_search_by_key(&did, |(did, _)| did)
            .map(|idx| self.withholding_tax[idx].1)
            .unwrap_or_else(|_| self.default_withholding_tax)
    }
}

/// A `Ticker`-local CA ID.
/// By *local*, we mean that the same number might be used for a different `Ticker`
/// to uniquely identify a different CA.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, TypeInfo, Default, Debug)]
pub struct LocalCAId(pub u32);
impl_checked_inc!(LocalCAId);

/// A unique global identifier for a CA.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, TypeInfo, Debug)]
pub struct CAId {
    /// The `Ticker` component used to disambiguate the `local` one.
    pub ticker: Ticker,
    /// The per-`Ticker` local identifier.
    pub local_id: LocalCAId,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Eq, Encode, Decode, TypeInfo, Debug)]
pub struct InitiateCorporateActionArgs {
    ticker: Ticker,
    kind: CAKind,
    decl_date: Moment,
    record_date: Option<RecordDateSpec>,
    details: CADetails,
    targets: Option<TargetIdentities>,
    default_withholding_tax: Option<Tax>,
    withholding_tax: Option<Vec<(IdentityId, Tax)>>,
}

/// Weight abstraction for the corporate actions module.
pub trait WeightInfo {
    fn set_max_details_length() -> Weight;
    fn set_default_targets(i: u32) -> Weight;
    fn set_default_withholding_tax() -> Weight;
    fn set_did_withholding_tax(existing_overrides: u32) -> Weight;
    fn initiate_corporate_action_use_defaults(whts: u32, target_ids: u32) -> Weight;
    fn initiate_corporate_action_provided(whts: u32, target_ids: u32) -> Weight;
    fn link_ca_doc(docs: u32) -> Weight;
    fn remove_ca_with_ballot() -> Weight;
    fn remove_ca_with_dist() -> Weight;
    fn change_record_date_with_ballot() -> Weight;
    fn change_record_date_with_dist() -> Weight;
}

/// The module's configuration trait.
pub trait Config: frame_system::Config + BalancesConfig + IdentityConfig + asset::Config {
    /// The overarching event type.
    type RuntimeEvent: From<Event>
        + From<ballot::Event>
        + From<distribution::Event>
        + Into<<Self as frame_system::Config>::RuntimeEvent>;

    /// Max number of DID specified in `TargetIdentities`.
    type MaxTargetIds: Get<u32>;

    /// Max number of per-DID withholding tax overrides.
    type MaxDidWhts: Get<u32>;

    /// Weight information for extrinsics in the corporate actions pallet.
    type WeightInfo: WeightInfo;

    /// Weight information for extrinsics in the corporate ballot pallet.
    type BallotWeightInfo: ballot::WeightInfo;

    /// Weight information for extrinsics in the capital distribution pallet.
    type DistWeightInfo: distribution::WeightInfo;
}

type Asset<T> = pallet_asset::Module<T>;
type Ballot<T> = ballot::Module<T>;
type Checkpoint<T> = checkpoint::Module<T>;
type Distribution<T> = distribution::Module<T>;
type ExternalAgents<T> = pallet_external_agents::Module<T>;

decl_storage! {
    trait Store for Module<T: Config> as CorporateAction {
        /// Determines the maximum number of bytes that the free-form `details` of a CA can store.
        ///
        /// Note that this is not the number of `char`s or the number of [graphemes].
        /// While this may be unnatural in terms of human understanding of a text's length,
        /// it more closely reflects actual storage costs (`'a'` is cheaper to store than an emoji).
        ///
        /// [graphemes]: https://en.wikipedia.org/wiki/Grapheme
        pub MaxDetailsLength get(fn max_details_length) config(): u32;

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
        pub DefaultWithholdingTax get(fn default_withholding_tax): map hasher(blake2_128_concat) Ticker => Tax;

        /// The amount of tax to withhold ("withholding tax", WT) for a certain ticker x DID.
        /// If an entry exists for a certain DID, it overrides the default in `DefaultWithholdingTax`.
        ///
        /// (ticker => [(did, % to withhold)]
        pub DidWithholdingTax get(fn did_withholding_tax): map hasher(blake2_128_concat) Ticker => Vec<(IdentityId, Tax)>;

        /// The next per-`Ticker` CA ID in the sequence.
        /// The full ID is defined as a combination of `Ticker` and a number in this sequence.
        pub CAIdSequence get(fn ca_id_sequence): map hasher(blake2_128_concat) Ticker => LocalCAId;

        /// All recorded CAs thus far.
        /// Only generic information is stored here.
        /// Specific `CAKind`s, e.g., benefits and corporate ballots, may use additional on-chain storage.
        ///
        /// (ticker => local ID => the corporate action)
        pub CorporateActions get(fn corporate_actions):
            double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) LocalCAId => Option<CorporateAction>;

        /// Associations from CAs to `Document`s via their IDs.
        /// (CAId => [DocumentId])
        ///
        /// The `CorporateActions` map stores `Ticker => LocalId => The CA`,
        /// so we can infer `Ticker => CAId`. Therefore, we don't need a double map.
        pub CADocLink get(fn ca_doc_link): map hasher(blake2_128_concat) CAId => Vec<DocumentId>;

        /// Associates details in free-form text with a CA by its ID.
        /// (CAId => CADetails)
        pub Details get(fn details): map hasher(blake2_128_concat) CAId => CADetails;

        /// Storage version.
        StorageVersion get(fn storage_version) build(|_| Version::new(0)): Version;
    }
}

storage_migration_ver!(0);

// Public interface for this runtime module.
decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::RuntimeOrigin {
        type Error = Error<T>;

        /// initialize the default event for this module
        fn deposit_event() = default;

        const MaxTargetIds: u32 = T::MaxTargetIds::get();
        const MaxDidWhts: u32 = T::MaxDidWhts::get();

        /// Set the max `length` of `details` in terms of bytes.
        /// May only be called via a PIP.
        #[weight = <T as Config>::WeightInfo::set_max_details_length()]
        pub fn set_max_details_length(origin, length: u32) {
            ensure_root(origin)?;
            MaxDetailsLength::put(length);
            Self::deposit_event(Event::MaxDetailsLengthChanged(GC_DID, length));
        }

        /// Set the default CA `TargetIdentities` to `targets`.
        ///
        /// ## Arguments
        /// - `origin` which must be an external agent of `ticker` with relevant permissions.
        /// - `ticker` for which the default identities are changing.
        /// - `targets` the default target identities for a CA.
        ///
        /// ## Errors
        /// - `UnauthorizedAgent` if `origin` is not agent-permissioned for `ticker`.
        /// - `TooManyTargetIds` if `targets.identities.len() > T::MaxTargetIds::get()`.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::set_default_targets(targets.identities.len() as u32)]
        pub fn set_default_targets(origin, ticker: Ticker, targets: TargetIdentities) {
            let agent = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;

            Self::ensure_target_ids_limited(&targets)?;

            // Dedup + sort any DIDs in `targets` for `O(log n)` containment check later.
            let new = targets.dedup();

            // Commit + emit event.
            DefaultTargetIdentities::mutate(ticker, |slot| *slot = new.clone());
            Self::deposit_event(Event::DefaultTargetIdentitiesChanged(agent, ticker, new));
        }

        /// Set the default withholding tax for all DIDs and CAs relevant to this `ticker`.
        ///
        /// ## Arguments
        /// - `origin` which must be an external agent of `ticker` with relevant permissions.
        /// - `ticker` that the withholding tax will apply to.
        /// - `tax` that should be withheld when distributing dividends, etc.
        ///
        /// ## Errors
        /// - `UnauthorizedAgent` if `origin` is not agent-permissioned for `ticker`.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::set_default_withholding_tax()]
        pub fn set_default_withholding_tax(origin, ticker: Ticker, tax: Tax) {
            let agent = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
            DefaultWithholdingTax::mutate(ticker, |slot| *slot = tax);
            Self::deposit_event(Event::DefaultWithholdingTaxChanged(agent, ticker, tax));
        }

        /// Set the withholding tax of `ticker` for `taxed_did` to `tax`.
        /// If `Some(tax)`, this overrides the default withholding tax of `ticker` to `tax` for `taxed_did`.
        /// Otherwise, if `None`, the default withholding tax will be used.
        ///
        /// ## Arguments
        /// - `origin` which must be an external agent of `ticker` with relevant permissions.
        /// - `ticker` that the withholding tax will apply to.
        /// - `taxed_did` that will have its withholding tax updated.
        /// - `tax` that should be withheld when distributing dividends, etc.
        ///
        /// ## Errors
        /// - `UnauthorizedAgent` if `origin` is not agent-permissioned for `ticker`.
        /// - `TooManyDidTaxes` if `Some(tax)` and adding the override would go over the limit `MaxDidWhts`.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::set_did_withholding_tax(T::MaxDidWhts::get())]
        pub fn set_did_withholding_tax(origin, ticker: Ticker, taxed_did: IdentityId, tax: Option<Tax>) {
            let agent = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
            DidWithholdingTax::try_mutate(ticker, |whts| -> DispatchResult {
                // We maintain sorted order, so we get O(log n) search but O(n) insertion/deletion.
                // This is maintained to get O(log n) in capital distribution.
                match (tax, whts.binary_search_by_key(&taxed_did, |(did, _)| *did)) {
                    (Some(tax), Ok(idx)) => whts[idx] = (taxed_did, tax),
                    (Some(tax), Err(idx)) => {
                        Self::ensure_did_whts_limited(whts.len() + 1)?;
                        whts.insert(idx, (taxed_did, tax))
                    }
                    (None, Ok(idx)) => drop(whts.remove(idx)),
                    (None, Err(_)) => {}
                }
                Ok(())
            })?;
            Self::deposit_event(Event::DidWithholdingTaxChanged(agent, ticker, taxed_did, tax));
        }

        /// Initiates a CA for `ticker` of `kind` with `details` and other provided arguments.
        ///
        /// ## Arguments
        /// - `origin` which must be an external agent of `ticker` with relevant permissions.
        /// - `ticker` that the CA is made for.
        /// - `kind` of CA being initiated.
        /// - `decl_date` of CA bring initialized.
        /// - `record_date`, if any, to calculate the impact of this CA.
        ///    If provided, this results in a scheduled balance snapshot ("checkpoint") at the date.
        /// - `details` of the CA in free-text form, up to a certain number of bytes in length.
        /// - `targets`, if any, which this CA is relevant/irrelevant to.
        ///    Overrides, if provided, the default at the asset level (`set_default_targets`).
        /// - `default_withholding_tax`, if any, is the default withholding tax to use for this CA.
        ///    Overrides, if provided, the default at the asset level (`set_default_withholding_tax`).
        /// - `withholding_tax`, if any, provides per-DID withholding tax overrides.
        ///    Overrides, if provided, the default at the asset level (`set_did_withholding_tax`).
        ///
        /// # Errors
        /// - `DetailsTooLong` if `details.len()` goes beyond `max_details_length`.
        /// - `UnauthorizedAgent` if `origin` is not agent-permissioned for `ticker`.
        /// - `CounterOverflow` in the unlikely event that so many CAs were created for this `ticker`,
        ///   that integer overflow would have occured if instead allowed.
        /// - `TooManyDidTaxes` if `withholding_tax.unwrap().len()` would go over the limit `MaxDidWhts`.
        /// - `DuplicateDidTax` if a DID is included more than once in `wt`.
        /// - `TooManyTargetIds` if `targets.unwrap().identities.len() > T::MaxTargetIds::get()`.
        /// - `DeclDateInFuture` if the declaration date is not in the past.
        /// - When `record_date.is_some()`, other errors due to checkpoint scheduling may occur.
        ///
        /// # Permissions
        /// * Asset
        #[weight = initiate_corporate_action_weight::<T>(targets, withholding_tax)]
        pub fn initiate_corporate_action(
            origin,
            ticker: Ticker,
            kind: CAKind,
            decl_date: Moment,
            record_date: Option<RecordDateSpec>,
            details: CADetails,
            targets: Option<TargetIdentities>,
            default_withholding_tax: Option<Tax>,
            withholding_tax: Option<Vec<(IdentityId, Tax)>>,
        ) -> DispatchResult {
            // Ensure that a permissioned agent is calling.
            let agent = <ExternalAgents<T>>::ensure_perms(origin, ticker)?.for_event();

            Self::unsafe_initiate_corporate_action(
                agent,
                ticker,
                kind,
                decl_date,
                record_date,
                details,
                targets,
                default_withholding_tax,
                withholding_tax,
            ).map(drop)
        }

        /// Link the given CA `id` to the given `docs`.
        /// Any previous links for the CA are removed in favor of `docs`.
        ///
        /// The workflow here is to add the documents and initiating the CA in any order desired.
        /// Once both exist, they can now be linked together.
        ///
        /// ## Arguments
        /// - `origin` which must be an external agent of `id.ticker` with relevant permissions.
        /// - `id` of the CA to associate with `docs`.
        /// - `docs` to associate with the CA with `id`.
        ///
        /// # Errors
        /// - `UnauthorizedAgent` if `origin` is not agent-permissioned for `ticker`.
        /// - `NoSuchCA` if `id` does not identify an existing CA.
        /// - `NoSuchDoc` if any of `docs` does not identify an existing document.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::link_ca_doc(docs.len() as u32)]
        pub fn link_ca_doc(origin, id: CAId, docs: Vec<DocumentId>) {
            // Ensure that a permissioned agent is calling and that CA and the docs exists.
            let agent = <ExternalAgents<T>>::ensure_perms(origin, id.ticker)?;
            Self::ensure_ca_exists(id)?;
            for doc in &docs {
                <Asset<T>>::ensure_doc_exists(&id.ticker, doc)?;
            }

            // Add the link and emit event.
            CADocLink::mutate(id, |slot| *slot = docs.clone());
            Self::deposit_event(Event::CALinkedToDoc(agent, id, docs));
        }

        /// Removes the CA identified by `ca_id`.
        ///
        /// Associated data, such as document links, ballots,
        /// and capital distributions are also removed.
        ///
        /// Any schedule associated with the record date will see
        /// `strong_ref_count(schedule_id)` decremented.
        ///
        /// ## Arguments
        /// - `origin` which must be an external agent of `ca_id.ticker` with relevant permissions.
        /// - `ca_id` of the CA to remove.
        ///
        /// # Errors
        /// - `UnauthorizedAgent` if `origin` is not agent-permissioned for `ticker`.
        /// - `NoSuchCA` if `id` does not identify an existing CA.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::remove_ca_with_ballot()
            .max(<T as Config>::WeightInfo::remove_ca_with_dist())]
        pub fn remove_ca(origin, ca_id: CAId) {
            // Ensure origin is a permissioned agent + CA exists.
            let agent = <ExternalAgents<T>>::ensure_perms(origin, ca_id.ticker)?.for_event();
            let ca = Self::ensure_ca_exists(ca_id)?;

            // Remove associated services.
            match ca.kind {
                CAKind::Other | CAKind::Reorganization => {}
                CAKind::IssuerNotice => {
                    if let Some(range) = <Ballot<T>>::time_ranges(ca_id) {
                        <Ballot<T>>::remove_ballot_base(agent, ca_id, range)?;
                    }
                }
                CAKind::PredictableBenefit | CAKind::UnpredictableBenefit => {
                    if let Some(dist) = <Distribution<T>>::distributions(ca_id) {
                        <Distribution<T>>::unverified_remove_distribution(agent, ca_id, &dist)?;
                    }
                }
            }

            // Decrement, Remove, and Emit event.
            Self::dec_strong_ref_count(ca_id, ca.record_date);
            CorporateActions::remove(ca_id.ticker, ca_id.local_id);
            CADocLink::remove(ca_id);
            Details::remove(ca_id);
            Self::deposit_event(Event::CARemoved(agent, ca_id));
        }

        /// Changes the record date of the CA identified by `ca_id`.
        ///
        /// ## Arguments
        /// - `origin` which must be an external agent of `ca_id.ticker` with relevant permissions.
        /// - `ca_id` of the CA to alter.
        /// - `record_date`, if any, to calculate the impact of the CA.
        ///    If provided, this results in a scheduled balance snapshot ("checkpoint") at the date.
        ///
        /// # Errors
        /// - `UnauthorizedAgent` if `origin` is not agent-permissioned for `ticker`.
        /// - `NoSuchCA` if `id` does not identify an existing CA.
        /// - When `record_date.is_some()`, other errors due to checkpoint scheduling may occur.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::change_record_date_with_ballot()
            .max(<T as Config>::WeightInfo::change_record_date_with_dist())]
        pub fn change_record_date(origin, ca_id: CAId, record_date: Option<RecordDateSpec>) {
            // Ensure origin is a permissioned agent + CA exists.
            let agent = <ExternalAgents<T>>::ensure_perms(origin, ca_id.ticker)?.for_event();
            let mut ca = Self::ensure_ca_exists(ca_id)?;

            with_transaction(|| -> DispatchResult {
                // If provided, either use the existing CP ID or schedule one to be made.
                Self::dec_strong_ref_count(ca_id, ca.record_date);
                ca.record_date = record_date
                    .map(|date| Self::handle_record_date(agent, ca_id.ticker, date))
                    .transpose()?;

                // Ensure associated services allow changing the date.
                match ca.kind {
                    CAKind::Other | CAKind::Reorganization => {}
                    CAKind::IssuerNotice => {
                        if let Some(range) = <Ballot<T>>::time_ranges(ca_id) {
                            Self::ensure_record_date_before_start(&ca, range.start)?;
                            <Ballot<T>>::ensure_ballot_not_started(range)?;
                        }
                    }
                    CAKind::PredictableBenefit | CAKind::UnpredictableBenefit => {
                        if let Some(dist) = <Distribution<T>>::distributions(ca_id) {
                            Self::ensure_record_date_before_start(&ca, dist.payment_at)?;
                            <Distribution<T>>::ensure_distribution_not_started(&dist)?;
                        }
                    }
                }
                Ok(())
            })?;

            // Commit changes + emit event.
            CorporateActions::insert(ca_id.ticker, ca_id.local_id, ca.clone());
            Self::deposit_event(Event::RecordDateChanged(agent, ca_id, ca));
        }

        /// Utility extrinsic to batch `initiate_corporate_action` and `distribute`
         #[weight = initiate_corporate_action_weight::<T>(&ca_args.targets, &ca_args.withholding_tax)
            .saturating_add(<T as Config>::DistWeightInfo::distribute())]
        pub fn initiate_corporate_action_and_distribute(
            origin,
            ca_args: InitiateCorporateActionArgs,
            portfolio: Option<PortfolioNumber>,
            currency: Ticker,
            per_share: Balance,
            amount: Balance,
            payment_at: Moment,
            expires_at: Option<Moment>,
        ) -> DispatchResult {
            let InitiateCorporateActionArgs {
                ticker,
                kind,
                decl_date,
                record_date,
                details,
                targets,
                default_withholding_tax,
                withholding_tax
            } = ca_args;

            let PermissionedCallOriginData {
                primary_did: agent,
                secondary_key,
                ..
            } = <ExternalAgents<T>>::ensure_agent_asset_perms(origin, ticker)?;

            with_transaction(|| {
                let ca_id = Self::unsafe_initiate_corporate_action(
                    agent.for_event(),
                    ticker,
                    kind,
                    decl_date,
                    record_date,
                    details,
                    targets,
                    default_withholding_tax,
                    withholding_tax
                )?;

                <distribution::Module<T>>::unverified_distribute(
                    agent,
                    secondary_key,
                    ca_id,
                    portfolio,
                    currency,
                    per_share,
                    amount,
                    payment_at,
                    expires_at,
                )
            })
        }
    }
}

decl_event! {
    pub enum Event {
        /// The maximum length of `details` in bytes was changed.
        /// (GC DID, new length)
        MaxDetailsLengthChanged(IdentityId, u32),
        /// The set of default `TargetIdentities` for a ticker changed.
        /// (Agent DID, Ticker, New TargetIdentities)
        DefaultTargetIdentitiesChanged(IdentityId, Ticker, TargetIdentities),
        /// The default withholding tax for a ticker changed.
        /// (Agent DID, Ticker, New Tax).
        DefaultWithholdingTaxChanged(IdentityId, Ticker, Tax),
        /// The withholding tax specific to a DID for a ticker changed.
        /// (Agent DID, Ticker, Taxed DID, New Tax).
        DidWithholdingTaxChanged(IdentityId, Ticker, IdentityId, Option<Tax>),
        /// A new DID was made the CAA.
        /// (New CAA DID, Ticker, New CAA DID).
        CAATransferred(IdentityId, Ticker, IdentityId),
        /// A CA was initiated.
        /// (Agent DID, CA id, the CA, the CA details)
        CAInitiated(EventDid, CAId, CorporateAction, CADetails),
        /// A CA was linked to a set of docs.
        /// (Agent DID, CA Id, List of doc identifiers)
        CALinkedToDoc(IdentityId, CAId, Vec<DocumentId>),
        /// A CA was removed.
        /// (Agent DID, CA Id)
        CARemoved(EventDid, CAId),
        /// A CA's record date changed.
        RecordDateChanged(EventDid, CAId, CorporateAction),
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        /// The authorization type is not to transfer the CAA to another DID.
        AuthNotCAATransfer,
        /// The `details` of a CA exceeded the max allowed length.
        DetailsTooLong,
        /// A withholding tax override for a given DID was specified more than once.
        /// The chain refused to make a choice, and hence there was an error.
        DuplicateDidTax,
        /// Too many withholding tax overrides were specified.
        TooManyDidTaxes,
        /// Too many identities in `TargetIdentities` were specified.
        TooManyTargetIds,
        /// On CA creation, a checkpoint ID was provided which doesn't exist.
        NoSuchCheckpointId,
        /// A CA with the given `CAId` did not exist.
        NoSuchCA,
        /// The CA did not have a record date.
        NoRecordDate,
        /// A CA's record date was strictly after the "start" time,
        /// where "start" is context dependent.
        /// For example, it could be the start of a ballot, or the start-of-payment in capital distribution.
        RecordDateAfterStart,
        /// A CA's declaration date was strictly after its record date.
        DeclDateAfterRecordDate,
        /// A CA's declaration date occurs in the future.
        DeclDateInFuture,
        /// CA does not target the DID.
        NotTargetedByCA,
    }
}

impl<T: Config> Module<T> {
    fn unsafe_initiate_corporate_action(
        agent: EventDid,
        ticker: Ticker,
        kind: CAKind,
        decl_date: Moment,
        record_date: Option<RecordDateSpec>,
        details: CADetails,
        targets: Option<TargetIdentities>,
        default_withholding_tax: Option<Tax>,
        withholding_tax: Option<Vec<(IdentityId, Tax)>>,
    ) -> Result<CAId, DispatchError> {
        // Ensure that `details` is short enough.
        ensure!(
            details.len() <= Self::max_details_length() as usize,
            Error::<T>::DetailsTooLong
        );

        // Ensure that the next local CA ID doesn't overflow.
        let mut next_id = CAIdSequence::get(ticker);
        let local_id = try_next_post::<T, _>(&mut next_id)?;
        let id = CAId { ticker, local_id };

        // Ensure there are no duplicates in withholding tax overrides
        // and that we're within the limit.
        let mut withholding_tax = withholding_tax;
        if let Some(wt) = &mut withholding_tax {
            let before = wt.len();
            Self::ensure_did_whts_limited(before)?;
            wt.sort_unstable_by_key(|&(did, _)| did);
            wt.dedup_by_key(|&mut (did, _)| did);
            ensure!(before == wt.len(), Error::<T>::DuplicateDidTax);
        }

        // Ensure target ids are limited in number if provided.
        if let Some(ref targets) = targets {
            Self::ensure_target_ids_limited(targets)?;
        }

        // Declaration date must be <= now.
        ensure!(
            decl_date <= <Checkpoint<T>>::now_unix(),
            Error::<T>::DeclDateInFuture
        );

        // If provided, either use the existing CP ID or schedule one to be made.
        let record_date = record_date
            .map(|date| {
                with_transaction(|| -> Result<_, DispatchError> {
                    let rd = Self::handle_record_date(agent, ticker, date)?;
                    ensure!(decl_date <= rd.date, Error::<T>::DeclDateAfterRecordDate);
                    Ok(rd)
                })
            })
            .transpose()?;

        // Commit the next local CA ID.
        CAIdSequence::insert(ticker, next_id);

        // Use asset level defaults if data not provided here.
        let targets = targets
            .map(|t| t.dedup())
            .unwrap_or_else(|| Self::default_target_identities(ticker));
        let default_withholding_tax =
            default_withholding_tax.unwrap_or_else(|| Self::default_withholding_tax(ticker));
        let withholding_tax = withholding_tax.unwrap_or_else(|| Self::did_withholding_tax(ticker));

        // Commit CA to storage.
        let ca = CorporateAction {
            kind,
            decl_date,
            record_date,
            targets,
            default_withholding_tax,
            withholding_tax,
        };
        CorporateActions::insert(ticker, id.local_id, ca.clone());
        Details::insert(id, details.clone());

        // Emit event.
        Self::deposit_event(Event::CAInitiated(agent, id, ca, details));
        Ok(id)
    }

    /// Ensure number of identities in `TargetIdentities` are limited.
    fn ensure_target_ids_limited(targets: &TargetIdentities) -> DispatchResult {
        ensure!(
            targets.identities.len() <= T::MaxTargetIds::get() as usize,
            Error::<T>::TooManyTargetIds
        );
        Ok(())
    }

    /// Ensure number of per-DID withholding tax overrides are limited.
    fn ensure_did_whts_limited(len: usize) -> DispatchResult {
        ensure!(
            len <= T::MaxDidWhts::get() as usize,
            Error::<T>::TooManyDidTaxes
        );
        Ok(())
    }

    // Ensure that `record_date <= start`.
    pub(crate) fn ensure_record_date_before_start(
        ca: &CorporateAction,
        start: Moment,
    ) -> DispatchResult {
        match ca.record_date {
            Some(rd) if rd.date <= start => Ok(()),
            Some(_) => Err(Error::<T>::RecordDateAfterStart.into()),
            None => Err(Error::<T>::NoRecordDate.into()),
        }
    }

    /// Returns the balance for `did` at `cp`, if any, or `did`'s current balance otherwise.
    pub(crate) fn balance_at_cp(did: IdentityId, ca_id: CAId, cp: Option<CheckpointId>) -> Balance {
        let ticker = ca_id.ticker;
        match cp {
            // CP exists, use it.
            Some(cp_id) => <Asset<T>>::get_balance_at(ticker, did, cp_id),
            // Although record date has passed, no transfers have happened yet for `ticker`.
            // Thus, there is no checkpoint ID, and we must use current balance instead.
            None => <Asset<T>>::balance_of(ticker, did),
        }
    }

    // Extract checkpoint ID for the CA's record date, if any.
    // Assumes the CA has a record date where `date <= now`.
    pub(crate) fn record_date_cp(ca: &CorporateAction, ca_id: CAId) -> Option<CheckpointId> {
        // Record date has passed by definition.
        let ticker = ca_id.ticker;
        match ca.record_date.unwrap().checkpoint {
            CACheckpoint::Existing(id) => Some(id),
            // For CAs, there can be more than one CP,
            // since you may attach a pre-existing and recurring schedule to it.
            // However, the record date stores the index for the CP,
            // assuming a transfer has happened since the record date.
            CACheckpoint::Scheduled(id, idx) => <Checkpoint<T>>::schedule_points(ticker, id)
                .get(idx as usize)
                .copied(),
        }
    }

    /// Ensure that `ca` targets `did`.
    pub(crate) fn ensure_ca_targets(ca: &CorporateAction, did: &IdentityId) -> DispatchResult {
        ensure!(ca.targets.targets(did), Error::<T>::NotTargetedByCA);
        Ok(())
    }

    /// Decrement the strong reference count of any schedule used in the `record_date` of `ca_id`.
    fn dec_strong_ref_count(ca_id: CAId, record_date: Option<RecordDate>) {
        if let Some(RecordDate {
            checkpoint: CACheckpoint::Scheduled(sh_id, _),
            ..
        }) = record_date
        {
            // We've proven by getting here that `c > 0`, so `c - 1` cannot underflow.
            ScheduleRefCount::mutate(ca_id.ticker, sh_id, |c| *c -= 1);
        }
    }

    /// Translate record date to a format we can store.
    /// In the process, create a checkpoint schedule if needed.
    fn handle_record_date(
        agent: EventDid,
        ticker: Ticker,
        date: RecordDateSpec,
    ) -> Result<RecordDate, DispatchError> {
        let (date, checkpoint) = match date {
            RecordDateSpec::Scheduled(date) => {
                // Create the schedule and extract the date + id.
                // We set initial `strong_ref_count(id) <- 1`.
                let date = date.into();
                let schedule = <Checkpoint<T>>::create_schedule_base(agent, ticker, date, 1)?;
                // It might be the case that the CP was instantly created ^--.
                // Or it might not have. In either case, it will end up at index 0.
                (schedule.at, CACheckpoint::Scheduled(schedule.id, 0))
            }
            RecordDateSpec::ExistingSchedule(id) => {
                // Ensure the schedule exists and extract the record date.
                let schedules = <Checkpoint<T>>::schedules(ticker);
                let schedule = schedules[<Checkpoint<T>>::ensure_schedule_exists(&schedules, id)?];
                // Schedule cannot be removable, otherwise the CP module may remove it,
                // so we increment the strong reference count of `id`.
                ScheduleRefCount::mutate(ticker, id, |c| *c += 1);
                let cp_at_idx = SchedulePoints::decode_len(ticker, id).unwrap_or(0) as u64;
                (schedule.at, CACheckpoint::Scheduled(schedule.id, cp_at_idx))
            }
            RecordDateSpec::Existing(id) => {
                // Ensure the CP exists.
                ensure!(
                    <Checkpoint<T>>::checkpoint_exists(&ticker, id),
                    Error::<T>::NoSuchCheckpointId
                );
                (
                    <Checkpoint<T>>::timestamps(ticker, id),
                    CACheckpoint::Existing(id),
                )
            }
        };
        Ok(RecordDate { date, checkpoint })
    }

    /// Ensure that a CA with `id` exists, returning it, and erroring otherwise.
    fn ensure_ca_exists(id: CAId) -> Result<CorporateAction, DispatchError> {
        CorporateActions::get(id.ticker, id.local_id).ok_or_else(|| Error::<T>::NoSuchCA.into())
    }
}

fn initiate_corporate_action_weight<T: Config>(
    targets: &Option<TargetIdentities>,
    withholding_tax: &Option<Vec<(IdentityId, Tax)>>,
) -> Weight {
    <T as Config>::WeightInfo::initiate_corporate_action_use_defaults(
        T::MaxDidWhts::get(),
        T::MaxTargetIds::get(),
    )
    .max(
        <T as Config>::WeightInfo::initiate_corporate_action_provided(
            withholding_tax.as_ref().map_or(0, |whts| whts.len() as u32),
            targets.as_ref().map_or(0, |t| t.identities.len() as u32),
        ),
    )
}
