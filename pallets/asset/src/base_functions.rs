use core::mem;
use frame_support::dispatch::{DispatchError, DispatchResult};
use frame_support::ensure;
use frame_support::storage::{StorageDoubleMap, StorageMap, StorageValue};
use frame_support::BoundedBTreeSet;
use frame_system::ensure_root;

use pallet_base::Error::CounterOverflow;
use pallet_base::{ensure_opt_string_limited, ensure_string_limited, try_next_pre};
use pallet_identity::PermissionedCallOriginData;
use polymesh_common_utilities::constants::ERC1400_TRANSFER_SUCCESS;
use polymesh_common_utilities::nft::NFTTrait;
use polymesh_common_utilities::protocol_fee::{ChargeProtocolFee, ProtocolOp};
use polymesh_common_utilities::traits::asset::{Config, RawEvent};
use polymesh_common_utilities::with_transaction;
use polymesh_primitives::asset::{AssetName, AssetType, CustomAssetTypeId, FundingRoundName};
use polymesh_primitives::asset_metadata::{
    AssetMetadataKey, AssetMetadataLocalKey, AssetMetadataName, AssetMetadataSpec,
    AssetMetadataValue, AssetMetadataValueDetail,
};
use polymesh_primitives::settlement::InstructionId;
use polymesh_primitives::{
    extract_auth, AssetIdentifier, Balance, Document, DocumentId, IdentityId, Memo, PortfolioId,
    PortfolioKind, PortfolioUpdateReason, Ticker, WeightMeter,
};

use crate::{
    AssetDocuments, AssetDocumentsIdSequence, AssetMetadataGlobalKeyToName,
    AssetMetadataGlobalNameToKey, AssetMetadataGlobalSpecs, AssetMetadataLocalKeyToName,
    AssetMetadataLocalNameToKey, AssetMetadataLocalSpecs, AssetMetadataNextGlobalKey,
    AssetMetadataValueDetails, AssetMetadataValues, AssetNames, AssetOwnershipRelation,
    AssetOwnershipRelations, BalanceOf, Checkpoint, Error, ExternalAgents, Frozen, FundingRound,
    Identity, MandatoryMediators, Module, Portfolio, PreApprovedTicker, Statistics, Tickers,
    TickersExemptFromAffirmation, Tokens, Vec,
};

impl<T: Config> Module<T> {
    pub(crate) fn base_register_ticker(origin: T::RuntimeOrigin, ticker: Ticker) -> DispatchResult {
        let to_did = Identity::<T>::ensure_perms(origin)?;
        let expiry = Self::ticker_registration_checks(&ticker, to_did, false, || {
            Self::ticker_registration_config()
        })?;

        T::ProtocolFee::charge_fee(ProtocolOp::AssetRegisterTicker)?;
        Self::unverified_register_ticker(&ticker, to_did, expiry);

        Ok(())
    }

    /// Accepts and executes the ticker transfer.
    pub(crate) fn base_accept_ticker_transfer(
        origin: T::RuntimeOrigin,
        auth_id: u64,
    ) -> DispatchResult {
        let to = Identity::<T>::ensure_perms(origin)?;
        <Identity<T>>::accept_auth_with(&to.into(), auth_id, |data, auth_by| {
            let ticker = extract_auth!(data, TransferTicker(t));

            Self::ensure_asset_fresh(&ticker)?;

            let reg =
                Self::ticker_registration(&ticker).ok_or(Error::<T>::TickerRegistrationExpired)?;
            <Identity<T>>::ensure_auth_by(auth_by, reg.owner)?;

            Self::transfer_ticker(reg, ticker, to);
            Ok(())
        })
    }

    /// Accept and process a token ownership transfer.
    pub(crate) fn base_accept_token_ownership_transfer(
        origin: T::RuntimeOrigin,
        id: u64,
    ) -> DispatchResult {
        let to = Identity::<T>::ensure_perms(origin)?;
        <Identity<T>>::accept_auth_with(&to.into(), id, |data, auth_by| {
            let ticker = extract_auth!(data, TransferAssetOwnership(t));

            // Get the token details and ensure it exists.
            let mut token = Self::token_details(&ticker)?;

            // Ensure the authorization was created by a permissioned agent.
            <ExternalAgents<T>>::ensure_agent_permissioned(ticker, auth_by)?;

            // Get the ticker registration and ensure it exists.
            let mut reg = Self::ticker_registration(&ticker).ok_or(Error::<T>::NoSuchAsset)?;
            let old_owner = reg.owner;
            AssetOwnershipRelations::remove(old_owner, ticker);
            AssetOwnershipRelations::insert(to, ticker, AssetOwnershipRelation::AssetOwned);
            // Update ticker registration.
            reg.owner = to;
            <Tickers<T>>::insert(&ticker, reg);
            // Update token details.
            token.owner_did = to;
            Tokens::insert(&ticker, token);
            Self::deposit_event(RawEvent::AssetOwnershipTransferred(to, ticker, old_owner));
            Ok(())
        })
    }

    pub(crate) fn base_create_asset(
        origin: T::RuntimeOrigin,
        name: AssetName,
        ticker: Ticker,
        divisible: bool,
        asset_type: AssetType,
        identifiers: Vec<AssetIdentifier>,
        funding_round: Option<FundingRoundName>,
    ) -> Result<IdentityId, DispatchError> {
        let PermissionedCallOriginData {
            primary_did,
            secondary_key,
            ..
        } = Identity::<T>::ensure_origin_call_permissions(origin)?;
        Self::unsafe_create_asset(
            primary_did,
            secondary_key,
            name,
            ticker,
            divisible,
            asset_type,
            identifiers,
            funding_round,
        )
    }

    pub(crate) fn base_set_freeze(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        freeze: bool,
    ) -> DispatchResult {
        let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
        Self::ensure_asset_exists(&ticker)?;

        let (event, error) = match freeze {
            true => (
                RawEvent::AssetFrozen(did, ticker),
                Error::<T>::AlreadyFrozen,
            ),
            false => (RawEvent::AssetUnfrozen(did, ticker), Error::<T>::NotFrozen),
        };

        ensure!(Self::frozen(&ticker) != freeze, error);
        Frozen::insert(&ticker, freeze);

        Self::deposit_event(event);
        Ok(())
    }

    pub(crate) fn base_rename_asset(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        name: AssetName,
    ) -> DispatchResult {
        Self::ensure_asset_name_bounded(&name)?;
        Self::ensure_asset_exists(&ticker)?;
        let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;

        AssetNames::insert(&ticker, name.clone());
        Self::deposit_event(RawEvent::AssetRenamed(did, ticker, name));
        Ok(())
    }

    /// Issues `amount` tokens for `ticker` into the caller's portfolio.
    pub(crate) fn base_issue(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        amount: Balance,
        portfolio_kind: PortfolioKind,
    ) -> DispatchResult {
        let portfolio_id = Self::ensure_origin_ticker_and_portfolio_permissions(
            origin,
            ticker,
            portfolio_kind,
            false,
        )?;
        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        Self::_mint(
            &ticker,
            portfolio_id.did,
            amount,
            Some(ProtocolOp::AssetIssue),
            &mut weight_meter,
        )
    }

    pub(crate) fn base_redeem(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        value: Balance,
        portfolio_kind: PortfolioKind,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        let portfolio = Self::ensure_origin_ticker_and_portfolio_permissions(
            origin,
            ticker,
            portfolio_kind,
            true,
        )?;

        Self::ensure_granular(&ticker, value)?;

        let mut token = Self::token_details(&ticker)?;
        // Ensures the token is fungible
        ensure!(
            token.asset_type.is_fungible(),
            Error::<T>::UnexpectedNonFungibleToken
        );

        // Reduce caller's portfolio balance. This makes sure that the caller has enough unlocked tokens.
        // If `advance_update_balances` fails, `reduce_portfolio_balance` shouldn't modify storage.

        with_transaction(|| {
            Portfolio::<T>::reduce_portfolio_balance(&portfolio, &ticker, value)?;

            // Try updating the total supply.
            token.total_supply = token
                .total_supply
                .checked_sub(value)
                .ok_or(Error::<T>::TotalSupplyOverflow)?;

            <Checkpoint<T>>::advance_update_balances(
                &ticker,
                &[(portfolio.did, Self::balance_of(ticker, portfolio.did))],
            )
        })?;

        let updated_balance = Self::balance_of(ticker, portfolio.did) - value;

        // Update identity balances and total supply
        BalanceOf::insert(ticker, &portfolio.did, updated_balance);
        Tokens::insert(ticker, token);

        // Update statistic info.
        Statistics::<T>::update_asset_stats(
            &ticker,
            Some(&portfolio.did),
            None,
            Some(updated_balance),
            None,
            value,
            weight_meter,
        )?;

        Self::deposit_event(RawEvent::AssetBalanceUpdated(
            portfolio.did,
            ticker,
            value,
            Some(portfolio),
            None,
            PortfolioUpdateReason::Redeemed,
        ));
        Ok(())
    }

    pub(crate) fn base_make_divisible(origin: T::RuntimeOrigin, ticker: Ticker) -> DispatchResult {
        let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;

        Tokens::try_mutate(&ticker, |token| -> DispatchResult {
            let token = token.as_mut().ok_or(Error::<T>::NoSuchAsset)?;
            // Ensures the token is fungible
            ensure!(
                token.asset_type.is_fungible(),
                Error::<T>::UnexpectedNonFungibleToken
            );
            ensure!(!token.divisible, Error::<T>::AssetAlreadyDivisible);
            token.divisible = true;

            Self::deposit_event(RawEvent::DivisibilityChanged(did, ticker, true));
            Ok(())
        })
    }

    pub(crate) fn base_add_documents(
        origin: T::RuntimeOrigin,
        docs: Vec<Document>,
        ticker: Ticker,
    ) -> DispatchResult {
        let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;

        // Ensure strings are limited.
        for doc in &docs {
            ensure_string_limited::<T>(&doc.uri)?;
            ensure_string_limited::<T>(&doc.name)?;
            ensure_opt_string_limited::<T>(doc.doc_type.as_deref())?;
        }

        // Ensure we can advance documents ID sequence by `len`.
        let pre = AssetDocumentsIdSequence::try_mutate(ticker, |id| {
            id.0.checked_add(docs.len() as u32)
                .ok_or(CounterOverflow::<T>)
                .map(|new| mem::replace(id, DocumentId(new)))
        })?;

        // Charge fee.
        T::ProtocolFee::batch_charge_fee(ProtocolOp::AssetAddDocuments, docs.len())?;

        // Add the documents & emit events.
        for (id, doc) in (pre.0..).map(DocumentId).zip(docs) {
            AssetDocuments::insert(ticker, id, doc.clone());
            Self::deposit_event(RawEvent::DocumentAdded(did, ticker, id, doc));
        }
        Ok(())
    }

    pub(crate) fn base_remove_documents(
        origin: T::RuntimeOrigin,
        ids: Vec<DocumentId>,
        ticker: Ticker,
    ) -> DispatchResult {
        let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
        for id in ids {
            AssetDocuments::remove(ticker, id);
            Self::deposit_event(RawEvent::DocumentRemoved(did, ticker, id));
        }
        Ok(())
    }

    pub(crate) fn base_set_funding_round(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        name: FundingRoundName,
    ) -> DispatchResult {
        Self::ensure_funding_round_name_bounded(&name)?;
        let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;

        FundingRound::insert(ticker, name.clone());
        Self::deposit_event(RawEvent::FundingRoundSet(did, ticker, name));
        Ok(())
    }

    pub(crate) fn base_update_identifiers(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        identifiers: Vec<AssetIdentifier>,
    ) -> DispatchResult {
        let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
        Self::ensure_asset_idents_valid(&identifiers)?;
        Self::unverified_update_idents(did, ticker, identifiers);
        Ok(())
    }

    pub(crate) fn base_controller_transfer(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        value: Balance,
        from_portfolio: PortfolioId,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        let to_portfolio = Self::ensure_origin_ticker_and_portfolio_permissions(
            origin,
            ticker,
            PortfolioKind::Default,
            false,
        )?;

        // Transfer `value` of ticker tokens from `investor_did` to controller
        Self::unsafe_transfer(
            from_portfolio,
            to_portfolio,
            &ticker,
            value,
            None,
            None,
            to_portfolio.did,
            weight_meter,
        )?;
        Self::deposit_event(RawEvent::ControllerTransfer(
            to_portfolio.did,
            ticker,
            from_portfolio,
            value,
        ));
        Ok(())
    }

    pub(crate) fn base_register_custom_asset_type(
        origin: T::RuntimeOrigin,
        ty: Vec<u8>,
    ) -> Result<CustomAssetTypeId, DispatchError> {
        let did = Identity::<T>::ensure_perms(origin)?;
        Self::unsafe_register_custom_asset_type(did, ty)
    }

    pub(crate) fn base_set_asset_metadata(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        key: AssetMetadataKey,
        value: AssetMetadataValue,
        detail: Option<AssetMetadataValueDetail<T::Moment>>,
    ) -> DispatchResult {
        // Ensure the caller has the correct permissions for this asset.
        let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;

        Self::unverified_set_asset_metadata(did, ticker, key, value, detail)
    }

    pub(crate) fn base_set_asset_metadata_details(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        key: AssetMetadataKey,
        detail: AssetMetadataValueDetail<T::Moment>,
    ) -> DispatchResult {
        // Ensure the caller has the correct permissions for this asset.
        let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;

        // Check key exists.
        ensure!(
            Self::check_asset_metadata_key_exists(&ticker, &key),
            Error::<T>::AssetMetadataKeyIsMissing
        );

        // Check if value is currently locked.
        ensure!(
            !Self::is_asset_metadata_locked(ticker, key),
            Error::<T>::AssetMetadataValueIsLocked
        );

        // Prevent locking an asset metadata with no value
        if detail.is_locked(<pallet_timestamp::Pallet<T>>::get()) {
            AssetMetadataValues::try_get(&ticker, &key)
                .map_err(|_| Error::<T>::AssetMetadataValueIsEmpty)?;
        }

        // Set asset metadata value details.
        AssetMetadataValueDetails::<T>::insert(ticker, key, &detail);

        Self::deposit_event(RawEvent::SetAssetMetadataValueDetails(did, ticker, detail));
        Ok(())
    }

    pub(crate) fn base_register_and_set_local_asset_metadata(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        name: AssetMetadataName,
        spec: AssetMetadataSpec,
        value: AssetMetadataValue,
        detail: Option<AssetMetadataValueDetail<T::Moment>>,
    ) -> DispatchResult {
        // Ensure the caller has the correct permissions for this asset.
        let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;

        // Register local metadata type.
        let key = Self::unverified_register_asset_metadata_local_type(did, ticker, name, spec)?;

        Self::unverified_set_asset_metadata(did, ticker, key, value, detail)
    }

    pub(crate) fn base_register_asset_metadata_local_type(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        name: AssetMetadataName,
        spec: AssetMetadataSpec,
    ) -> DispatchResult {
        // Ensure the caller has the correct permissions for this asset.
        let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;

        Self::unverified_register_asset_metadata_local_type(did, ticker, name, spec).map(drop)
    }

    pub(crate) fn base_register_asset_metadata_global_type(
        origin: T::RuntimeOrigin,
        name: AssetMetadataName,
        spec: AssetMetadataSpec,
    ) -> DispatchResult {
        Self::ensure_asset_metadata_name_limited(&name)?;
        Self::ensure_asset_metadata_spec_limited(&spec)?;

        // Only allow global metadata types to be registered by root.
        ensure_root(origin)?;

        // Check if key already exists.
        ensure!(
            !AssetMetadataGlobalNameToKey::contains_key(&name),
            Error::<T>::AssetMetadataGlobalKeyAlreadyExists
        );

        // Next global key.
        let key = AssetMetadataNextGlobalKey::try_mutate(try_next_pre::<T, _>)?;

        // Store global key <-> name mapping.
        AssetMetadataGlobalNameToKey::insert(&name, key);
        AssetMetadataGlobalKeyToName::insert(key, &name);

        // Store global specs.
        AssetMetadataGlobalSpecs::insert(key, &spec);

        Self::deposit_event(RawEvent::RegisterAssetMetadataGlobalType(name, key, spec));
        Ok(())
    }

    pub(crate) fn base_update_asset_type(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        asset_type: AssetType,
    ) -> DispatchResult {
        Self::ensure_asset_exists(&ticker)?;
        Self::ensure_asset_type_valid(asset_type)?;
        let did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
        Tokens::try_mutate(&ticker, |token| -> DispatchResult {
            let token = token.as_mut().ok_or(Error::<T>::NoSuchAsset)?;
            // Ensures that both parameters are non fungible types or if both are fungible types.
            ensure!(
                token.asset_type.is_fungible() == asset_type.is_fungible(),
                Error::<T>::IncompatibleAssetTypeUpdate
            );
            token.asset_type = asset_type;
            Ok(())
        })?;
        Self::deposit_event(RawEvent::AssetTypeChanged(did, ticker, asset_type));
        Ok(())
    }

    pub(crate) fn base_remove_local_metadata_key(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        local_key: AssetMetadataLocalKey,
    ) -> DispatchResult {
        // Verifies if the caller has the correct permissions for this asset
        let caller_did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
        // Verifies if the key exists.
        let name = AssetMetadataLocalKeyToName::try_get(ticker, &local_key)
            .map_err(|_| Error::<T>::AssetMetadataKeyIsMissing)?;
        // Verifies if the value is locked
        let metadata_key = AssetMetadataKey::Local(local_key);
        if let Some(value_detail) = AssetMetadataValueDetails::<T>::get(&ticker, &metadata_key) {
            ensure!(
                !value_detail.is_locked(<pallet_timestamp::Pallet<T>>::get()),
                Error::<T>::AssetMetadataValueIsLocked
            );
        }
        // Verifies if the key belongs to an NFT collection
        ensure!(
            !T::NFTFn::is_collection_key(&ticker, &metadata_key),
            Error::<T>::AssetMetadataKeyBelongsToNFTCollection
        );
        // Remove key from storage
        AssetMetadataValues::remove(&ticker, &metadata_key);
        AssetMetadataValueDetails::<T>::remove(&ticker, &metadata_key);
        AssetMetadataLocalNameToKey::remove(&ticker, &name);
        AssetMetadataLocalKeyToName::remove(&ticker, &local_key);
        AssetMetadataLocalSpecs::remove(&ticker, &local_key);
        Self::deposit_event(RawEvent::LocalMetadataKeyDeleted(
            caller_did, ticker, local_key,
        ));
        Ok(())
    }

    pub(crate) fn base_remove_metadata_value(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        metadata_key: AssetMetadataKey,
    ) -> DispatchResult {
        // Verifies if the caller has the correct permissions for this asset
        let caller_did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
        // Verifies if the key exists.
        match metadata_key {
            AssetMetadataKey::Global(global_key) => {
                if !AssetMetadataGlobalKeyToName::contains_key(&global_key) {
                    return Err(Error::<T>::AssetMetadataKeyIsMissing.into());
                }
            }
            AssetMetadataKey::Local(local_key) => {
                if !AssetMetadataLocalKeyToName::contains_key(ticker, &local_key) {
                    return Err(Error::<T>::AssetMetadataKeyIsMissing.into());
                }
            }
        }
        // Verifies if the value is locked
        if let Some(value_detail) = AssetMetadataValueDetails::<T>::get(&ticker, &metadata_key) {
            ensure!(
                !value_detail.is_locked(<pallet_timestamp::Pallet<T>>::get()),
                Error::<T>::AssetMetadataValueIsLocked
            );
        }
        // Remove the metadata value from storage
        AssetMetadataValues::remove(&ticker, &metadata_key);
        AssetMetadataValueDetails::<T>::remove(&ticker, &metadata_key);
        Self::deposit_event(RawEvent::MetadataValueDeleted(
            caller_did,
            ticker,
            metadata_key,
        ));
        Ok(())
    }

    /// Pre-approves the receivement of the asset for all identities.
    pub(crate) fn base_exempt_ticker_affirmation(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
    ) -> DispatchResult {
        ensure_root(origin)?;
        TickersExemptFromAffirmation::insert(&ticker, true);
        Self::deposit_event(RawEvent::AssetAffirmationExemption(ticker));
        Ok(())
    }

    /// Removes the pre-approval of the asset for all identities.
    pub(crate) fn base_remove_ticker_affirmation_exemption(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
    ) -> DispatchResult {
        ensure_root(origin)?;
        TickersExemptFromAffirmation::remove(&ticker);
        Self::deposit_event(RawEvent::RemoveAssetAffirmationExemption(ticker));
        Ok(())
    }

    /// Pre-approves the receivement of an asset.
    pub(crate) fn base_pre_approve_ticker(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
    ) -> DispatchResult {
        let caller_did = Identity::<T>::ensure_perms(origin)?;
        PreApprovedTicker::insert(&caller_did, &ticker, true);
        Self::deposit_event(RawEvent::PreApprovedAsset(caller_did, ticker));
        Ok(())
    }

    /// Removes the pre approval of an asset.
    pub(crate) fn base_remove_ticker_pre_approval(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
    ) -> DispatchResult {
        let caller_did = Identity::<T>::ensure_perms(origin)?;
        PreApprovedTicker::remove(&caller_did, &ticker);
        Self::deposit_event(RawEvent::RemovePreApprovedAsset(caller_did, ticker));
        Ok(())
    }

    /// Sets all identities in the `mediators` set as mandatory mediators for any instruction transfering `ticker`.
    pub(crate) fn base_add_mandatory_mediators(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        new_mediators: BoundedBTreeSet<IdentityId, T::MaxAssetMediators>,
    ) -> DispatchResult {
        // Verifies if the caller has the correct permissions for this asset
        let caller_did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
        // Tries to add all new identities as mandatory mediators for the asset
        MandatoryMediators::<T>::try_mutate(ticker, |mandatory_mediators| -> DispatchResult {
            for new_mediator in &new_mediators {
                mandatory_mediators
                    .try_insert(*new_mediator)
                    .map_err(|_| Error::<T>::NumberOfAssetMediatorsExceeded)?;
            }
            Ok(())
        })?;

        Self::deposit_event(RawEvent::AssetMediatorsAdded(
            caller_did,
            ticker,
            new_mediators.into_inner(),
        ));
        Ok(())
    }

    /// Removes all identities in the `mediators` set from the mandatory mediators list for the given `ticker`.
    pub(crate) fn base_remove_mandatory_mediators(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        mediators: BoundedBTreeSet<IdentityId, T::MaxAssetMediators>,
    ) -> DispatchResult {
        // Verifies if the caller has the correct permissions for this asset
        let caller_did = <ExternalAgents<T>>::ensure_perms(origin, ticker)?;
        // Removes the identities from the mandatory mediators list
        MandatoryMediators::<T>::mutate(ticker, |mandatory_mediators| {
            for mediator in &mediators {
                mandatory_mediators.remove(mediator);
            }
        });
        Self::deposit_event(RawEvent::AssetMediatorsRemoved(
            caller_did,
            ticker,
            mediators.into_inner(),
        ));
        Ok(())
    }

    /// Transfers an asset from one identity portfolio to another
    pub fn base_transfer(
        from_portfolio: PortfolioId,
        to_portfolio: PortfolioId,
        ticker: &Ticker,
        value: Balance,
        instruction_id: Option<InstructionId>,
        instruction_memo: Option<Memo>,
        caller_did: IdentityId,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        // NB: This function does not check if the sender/receiver have custodian permissions on the portfolios.
        // The custodian permissions must be checked before this function is called.
        // The only place this function is used right now is the settlement engine and the settlement engine
        // checks custodial permissions when the instruction is authorized.

        // Validate the transfer
        let is_transfer_success =
            Self::_is_valid_transfer(&ticker, from_portfolio, to_portfolio, value, weight_meter)?;

        ensure!(
            is_transfer_success == ERC1400_TRANSFER_SUCCESS,
            Error::<T>::InvalidTransfer
        );

        Self::unsafe_transfer(
            from_portfolio,
            to_portfolio,
            ticker,
            value,
            instruction_id,
            instruction_memo,
            caller_did,
            weight_meter,
        )?;

        Ok(())
    }
}
