use arrayvec::ArrayVec;
use core::result::Result as StdResult;
use frame_support::dispatch::{DispatchError, DispatchResult};
use frame_support::storage::{StorageDoubleMap, StorageMap, StorageValue};
use frame_support::traits::Get;
use frame_support::{ensure, fail};
use sp_runtime::traits::Zero;
use sp_std::collections::btree_set::BTreeSet;

use pallet_base::{ensure_opt_string_limited, ensure_string_limited, try_next_pre};
use polymesh_common_utilities::compliance_manager::ComplianceFnConfig;
use polymesh_common_utilities::constants::currency::{MAX_SUPPLY, ONE_UNIT};
use polymesh_common_utilities::constants::*;
use polymesh_common_utilities::protocol_fee::ProtocolOp;
use polymesh_common_utilities::traits::asset::{Config, Event, RawEvent};
use polymesh_common_utilities::with_transaction;
use polymesh_common_utilities::ChargeProtocolFee;
use polymesh_primitives::agent::AgentGroup;
use polymesh_primitives::asset::{
    AssetName, AssetType, CheckpointId, CustomAssetTypeId, FundingRoundName,
};
use polymesh_primitives::asset_metadata::{
    AssetMetadataKey, AssetMetadataName, AssetMetadataSpec, AssetMetadataValue,
    AssetMetadataValueDetail,
};
use polymesh_primitives::settlement::InstructionId;
use polymesh_primitives::{
    AssetIdentifier, Balance, DocumentId, IdentityId, Memo, PortfolioId, PortfolioKind,
    PortfolioUpdateReason, SecondaryKey, Ticker, WeightMeter,
};

use crate::error::Error;
use crate::types::{
    AssetOwnershipRelation, SecurityToken, TickerRegistration, TickerRegistrationConfig,
    TickerRegistrationStatus,
};
use crate::{
    AssetDocuments, AssetMetadataGlobalKeyToName, AssetMetadataLocalKeyToName,
    AssetMetadataLocalNameToKey, AssetMetadataLocalSpecs, AssetMetadataNextLocalKey,
    AssetMetadataValueDetails, AssetMetadataValues, AssetNames, AssetOwnershipRelations, BalanceOf,
    Checkpoint, CustomTypeIdSequence, CustomTypes, CustomTypesInverse, ExternalAgents,
    FundingRound, Identifiers, Identity, IssuedInFundingRound, Module, Portfolio, Statistics,
    Tickers, Tokens, Vec,
};

impl<T: Config> Module<T> {
    /// Before registering a ticker, do some checks, and return the expiry moment.
    pub(crate) fn ticker_registration_checks(
        ticker: &Ticker,
        to_did: IdentityId,
        no_re_register: bool,
        config: impl FnOnce() -> TickerRegistrationConfig<T::Moment>,
    ) -> Result<Option<T::Moment>, DispatchError> {
        Self::verify_ticker_characters(&ticker)?;
        Self::ensure_asset_fresh(&ticker)?;

        let config = config();

        // Ensure the ticker is not too long.
        Self::ensure_ticker_length(&ticker, &config)?;

        // Ensure that the ticker is not registered by someone else (or `to_did`, possibly).
        if match Self::is_ticker_available_or_registered_to(&ticker, to_did) {
            TickerRegistrationStatus::RegisteredByOther => true,
            TickerRegistrationStatus::RegisteredByDid => no_re_register,
            _ => false,
        } {
            fail!(Error::<T>::TickerAlreadyRegistered);
        }

        Ok(config
            .registration_length
            .map(|exp| <pallet_timestamp::Pallet<T>>::get() + exp))
    }

    /// Returns `Ok` if the ticker contains only the following characters: `A`..`Z` `0`..`9` `_` `-` `.` `/`.
    pub fn verify_ticker_characters(ticker: &Ticker) -> DispatchResult {
        let ticker_bytes = ticker.as_ref();

        // The first byte of the ticker cannot be NULL
        if *ticker_bytes.first().unwrap_or(&0) == 0 {
            return Err(Error::<T>::TickerFirstByteNotValid.into());
        }

        // Allows the following characters: `A`..`Z` `0`..`9` `_` `-` `.` `/`
        let valid_characters = BTreeSet::from([b'_', b'-', b'.', b'/']);
        for (byte_index, ticker_byte) in ticker_bytes.iter().enumerate() {
            if !ticker_byte.is_ascii_uppercase()
                && !ticker_byte.is_ascii_digit()
                && !valid_characters.contains(ticker_byte)
            {
                if ticker_bytes[byte_index..].iter().all(|byte| *byte == 0) {
                    return Ok(());
                }

                return Err(Error::<T>::InvalidTickerCharacter.into());
            }
        }
        Ok(())
    }

    /// Ensure asset `ticker` doesn't exist yet.
    pub(crate) fn ensure_asset_fresh(ticker: &Ticker) -> DispatchResult {
        ensure!(
            !Tokens::contains_key(ticker),
            Error::<T>::AssetAlreadyCreated
        );
        Ok(())
    }

    /// Ensure ticker length is within limit per `config`.
    fn ensure_ticker_length<U>(
        ticker: &Ticker,
        config: &TickerRegistrationConfig<U>,
    ) -> DispatchResult {
        ensure!(
            ticker.len() <= usize::try_from(config.max_ticker_length).unwrap_or_default(),
            Error::<T>::TickerTooLong
        );
        Ok(())
    }

    /// Returns:
    /// - `RegisteredByOther` if ticker is registered to someone else.
    /// - `Available` if ticker is available for registry.
    /// - `RegisteredByDid` if ticker is already registered to provided did.
    fn is_ticker_available_or_registered_to(
        ticker: &Ticker,
        did: IdentityId,
    ) -> TickerRegistrationStatus {
        // Assumes uppercase ticker
        match Self::maybe_ticker(ticker) {
            Some(TickerRegistration { expiry, owner }) => match expiry {
                // Ticker registered to someone but expired and can be registered again.
                Some(expiry) if <pallet_timestamp::Pallet<T>>::get() > expiry => {
                    TickerRegistrationStatus::Available
                }
                // Ticker is already registered to provided did (may or may not expire in future).
                _ if owner == did => TickerRegistrationStatus::RegisteredByDid,
                // Ticker registered to someone else and hasn't expired.
                _ => TickerRegistrationStatus::RegisteredByOther,
            },
            // Ticker not registered yet.
            None => TickerRegistrationStatus::Available,
        }
    }

    fn maybe_ticker(ticker: &Ticker) -> Option<TickerRegistration<T::Moment>> {
        <Tickers<T>>::get(ticker)
    }

    pub fn token_details(ticker: &Ticker) -> Result<SecurityToken, DispatchError> {
        Ok(Tokens::try_get(ticker).or(Err(Error::<T>::NoSuchAsset))?)
    }

    /// Ensure `name` is within the global limit for asset name lengths.
    pub(crate) fn ensure_funding_round_name_bounded(name: &FundingRoundName) -> DispatchResult {
        ensure!(
            name.len() as u32 <= T::FundingRoundNameMaxLength::get(),
            Error::<T>::FundingRoundNameMaxLengthExceeded
        );
        Ok(())
    }

    /// Ensure `name` is within the global limit for asset name lengths.
    pub(crate) fn ensure_asset_name_bounded(name: &AssetName) -> DispatchResult {
        ensure!(
            name.len() as u32 <= T::AssetNameMaxLength::get(),
            Error::<T>::MaxLengthOfAssetNameExceeded
        );
        Ok(())
    }

    /// Ensure that all `idents` are valid.
    pub(crate) fn ensure_asset_idents_valid(idents: &[AssetIdentifier]) -> DispatchResult {
        ensure!(
            idents.iter().all(|i| i.is_valid()),
            Error::<T>::InvalidAssetIdentifier
        );
        Ok(())
    }

    /// Performs necessary checks on parameters of `create_asset`.
    fn ensure_create_asset_parameters(ticker: &Ticker) -> DispatchResult {
        Self::ensure_asset_fresh(&ticker)?;
        Self::ensure_ticker_length(&ticker, &Self::ticker_registration_config())
    }

    /// Ensures that `origin` is a permissioned agent for `ticker`, that the portfolio is valid and that calller
    /// has the access to the portfolio. If `ensure_custody` is `true`, also enforces the caller to have custody
    /// of the portfolio.
    pub fn ensure_origin_ticker_and_portfolio_permissions(
        origin: T::RuntimeOrigin,
        ticker: Ticker,
        portfolio_kind: PortfolioKind,
        ensure_custody: bool,
    ) -> Result<PortfolioId, DispatchError> {
        let origin_data = <ExternalAgents<T>>::ensure_agent_asset_perms(origin, ticker)?;
        let portfolio_id = PortfolioId::new(origin_data.primary_did, portfolio_kind);
        Portfolio::<T>::ensure_portfolio_validity(&portfolio_id)?;
        if ensure_custody {
            Portfolio::<T>::ensure_portfolio_custody(portfolio_id, origin_data.primary_did)?;
        }
        Portfolio::<T>::ensure_user_portfolio_permission(
            origin_data.secondary_key.as_ref(),
            portfolio_id,
        )?;
        Ok(portfolio_id)
    }

    pub(crate) fn ensure_granular(ticker: &Ticker, value: Balance) -> DispatchResult {
        ensure!(
            Self::check_granularity(&ticker, value),
            Error::<T>::InvalidGranularity
        );
        Ok(())
    }

    pub(crate) fn check_granularity(ticker: &Ticker, value: Balance) -> bool {
        Self::is_divisible(ticker) || Self::is_unit_multiple(value)
    }

    pub fn is_divisible(ticker: &Ticker) -> bool {
        Self::token_details(ticker)
            .map(|t| t.divisible)
            .unwrap_or_default()
    }

    /// Is `value` a multiple of "one unit"?
    fn is_unit_multiple(value: Balance) -> bool {
        value % ONE_UNIT == 0
    }

    pub fn check_asset_metadata_key_exists(ticker: &Ticker, key: &AssetMetadataKey) -> bool {
        match key {
            AssetMetadataKey::Global(key) => AssetMetadataGlobalKeyToName::contains_key(key),
            AssetMetadataKey::Local(key) => AssetMetadataLocalKeyToName::contains_key(ticker, key),
        }
    }

    pub(crate) fn is_asset_metadata_locked(ticker: Ticker, key: AssetMetadataKey) -> bool {
        AssetMetadataValueDetails::<T>::get(ticker, key).map_or(false, |details| {
            details.is_locked(<pallet_timestamp::Pallet<T>>::get())
        })
    }

    /// Ensure that `ticker` is a valid created asset.
    pub(crate) fn ensure_asset_exists(ticker: &Ticker) -> DispatchResult {
        ensure!(Tokens::contains_key(&ticker), Error::<T>::NoSuchAsset);
        Ok(())
    }

    /// Ensure `AssetType` is valid.
    /// This checks that the `AssetType::Custom(custom_type_id)` is valid.
    pub(crate) fn ensure_asset_type_valid(asset_type: AssetType) -> DispatchResult {
        if let AssetType::Custom(custom_type_id) = asset_type {
            ensure!(
                CustomTypes::contains_key(custom_type_id),
                Error::<T>::InvalidCustomAssetTypeId
            );
        }
        Ok(())
    }

    /// Ensure asset metadata `value` is within the global limit.
    fn ensure_asset_metadata_value_limited(value: &AssetMetadataValue) -> DispatchResult {
        ensure!(
            value.len() <= T::AssetMetadataValueMaxLength::get() as usize,
            Error::<T>::AssetMetadataValueMaxLengthExceeded
        );
        Ok(())
    }

    /// Ensure asset metadata `name` is within the global limit.
    pub(crate) fn ensure_asset_metadata_name_limited(name: &AssetMetadataName) -> DispatchResult {
        ensure!(
            name.len() <= T::AssetMetadataNameMaxLength::get() as usize,
            Error::<T>::AssetMetadataNameMaxLengthExceeded
        );
        Ok(())
    }

    /// Ensure asset metadata `spec` is within the global limit.
    pub(crate) fn ensure_asset_metadata_spec_limited(spec: &AssetMetadataSpec) -> DispatchResult {
        ensure_opt_string_limited::<T>(spec.url.as_deref())?;
        ensure_opt_string_limited::<T>(spec.description.as_deref())?;
        if let Some(ref type_def) = spec.type_def {
            ensure!(
                type_def.len() <= T::AssetMetadataTypeDefMaxLength::get() as usize,
                Error::<T>::AssetMetadataTypeDefMaxLengthExceeded
            );
        }
        Ok(())
    }

    /// Ensure `supply <= MAX_SUPPLY`.
    pub(crate) fn ensure_within_max_supply(supply: Balance) -> DispatchResult {
        ensure!(supply <= MAX_SUPPLY, Error::<T>::TotalSupplyAboveLimit);
        Ok(())
    }

    /// Returns `None` if there's no asset associated to the given ticker,
    /// returns Some(true) if the asset exists and is of type `AssetType::NonFungible`, and returns Some(false) otherwise.
    pub fn nft_asset(ticker: &Ticker) -> Option<bool> {
        let token = Tokens::try_get(ticker).ok()?;
        Some(token.asset_type.is_non_fungible())
    }

    /// Ensure that the document `doc` exists for `ticker`.
    pub fn ensure_doc_exists(ticker: &Ticker, doc: &DocumentId) -> DispatchResult {
        ensure!(
            AssetDocuments::contains_key(ticker, doc),
            Error::<T>::NoSuchDoc
        );
        Ok(())
    }

    pub fn get_balance_at(ticker: Ticker, did: IdentityId, at: CheckpointId) -> Balance {
        <Checkpoint<T>>::balance_at(ticker, did, at)
            .unwrap_or_else(|| Self::balance_of(&ticker, &did))
    }

    pub fn _is_valid_transfer(
        ticker: &Ticker,
        from_portfolio: PortfolioId,
        to_portfolio: PortfolioId,
        value: Balance,
        weight_meter: &mut WeightMeter,
    ) -> StdResult<u8, DispatchError> {
        if Self::frozen(ticker) {
            return Ok(ERC1400_TRANSFERS_HALTED);
        }

        if Self::portfolio_failure(&from_portfolio, &to_portfolio, ticker, value) {
            return Ok(PORTFOLIO_FAILURE);
        }

        if Self::statistics_failures(
            &from_portfolio.did,
            &to_portfolio.did,
            ticker,
            value,
            weight_meter,
        ) {
            return Ok(TRANSFER_MANAGER_FAILURE);
        }

        if !T::ComplianceManager::is_compliant(
            ticker,
            from_portfolio.did,
            to_portfolio.did,
            weight_meter,
        )? {
            return Ok(COMPLIANCE_MANAGER_FAILURE);
        }

        Ok(ERC1400_TRANSFER_SUCCESS)
    }

    fn portfolio_failure(
        from_portfolio: &PortfolioId,
        to_portfolio: &PortfolioId,
        ticker: &Ticker,
        value: Balance,
    ) -> bool {
        Portfolio::<T>::ensure_portfolio_transfer_validity(
            from_portfolio,
            to_portfolio,
            ticker,
            value,
        )
        .is_err()
    }

    fn statistics_failures(
        from_did: &IdentityId,
        to_did: &IdentityId,
        ticker: &Ticker,
        value: Balance,
        weight_meter: &mut WeightMeter,
    ) -> bool {
        let total_supply = Self::total_supply(ticker);
        Statistics::<T>::verify_transfer_restrictions(
            ticker,
            from_did,
            to_did,
            Self::balance_of(ticker, from_did),
            Self::balance_of(ticker, to_did),
            value,
            total_supply,
            weight_meter,
        )
        .is_err()
    }

    // Get the total supply of an asset `id`.
    pub fn total_supply(ticker: &Ticker) -> Balance {
        Self::token_details(ticker)
            .map(|t| t.total_supply)
            .unwrap_or_default()
    }
}

//==========================================================================
// All Storage Writes!
//==========================================================================

impl<T: Config> Module<T> {
    /// Registers the given `ticker` to the `owner` identity with an optional expiry time.
    ///
    /// ## Expected constraints
    /// - `owner` should be a valid IdentityId.
    /// - `ticker` should be valid, please see `ticker_registration_checks`.
    /// - `ticker` should be available or already registered by `owner`.
    pub(crate) fn unverified_register_ticker(
        ticker: &Ticker,
        owner: IdentityId,
        expiry: Option<T::Moment>,
    ) {
        if let Some(ticker_details) = Self::maybe_ticker(ticker) {
            AssetOwnershipRelations::remove(ticker_details.owner, ticker);
        }

        let ticker_registration = TickerRegistration { owner, expiry };

        // Store ticker registration details
        <Tickers<T>>::insert(ticker, ticker_registration);
        AssetOwnershipRelations::insert(owner, ticker, AssetOwnershipRelation::TickerOwned);

        Self::deposit_event(RawEvent::TickerRegistered(owner, *ticker, expiry));
    }

    /// Transfer the given `ticker`'s registration from `req.owner` to `to`.
    pub(crate) fn transfer_ticker(
        mut reg: TickerRegistration<T::Moment>,
        ticker: Ticker,
        to: IdentityId,
    ) {
        let from = reg.owner;
        AssetOwnershipRelations::remove(from, ticker);
        AssetOwnershipRelations::insert(to, ticker, AssetOwnershipRelation::TickerOwned);
        reg.owner = to;
        <Tickers<T>>::insert(&ticker, reg);
        Self::deposit_event(RawEvent::TickerTransferred(to, ticker, from));
    }

    pub(crate) fn unsafe_create_asset(
        did: IdentityId,
        secondary_key: Option<SecondaryKey<T::AccountId>>,
        name: AssetName,
        ticker: Ticker,
        divisible: bool,
        asset_type: AssetType,
        identifiers: Vec<AssetIdentifier>,
        funding_round: Option<FundingRoundName>,
    ) -> Result<IdentityId, DispatchError> {
        Self::ensure_asset_name_bounded(&name)?;
        if let Some(fr) = &funding_round {
            Self::ensure_funding_round_name_bounded(fr)?;
        }
        Self::ensure_asset_idents_valid(&identifiers)?;
        Self::ensure_asset_type_valid(asset_type)?;

        Self::ensure_create_asset_parameters(&ticker)?;

        // Ensure its registered by DID or at least expired, thus available.
        let available = match Self::is_ticker_available_or_registered_to(&ticker, did) {
            TickerRegistrationStatus::RegisteredByOther => {
                fail!(Error::<T>::TickerAlreadyRegistered)
            }
            TickerRegistrationStatus::RegisteredByDid => false,
            TickerRegistrationStatus::Available => true,
        };

        // If `ticker` isn't registered, it will be, so ensure it is fully ascii.
        if available {
            Self::verify_ticker_characters(&ticker)?;
        }

        let token_did = Identity::<T>::get_token_did(&ticker)?;
        // Ensure there's no pre-existing entry for the DID.
        // This should never happen, but let's be defensive here.
        Identity::<T>::ensure_no_id_record(token_did)?;

        // Ensure that the caller has relevant portfolio permissions
        let user_default_portfolio = PortfolioId::default_portfolio(did);
        Portfolio::<T>::ensure_portfolio_custody_and_permission(
            user_default_portfolio,
            did,
            secondary_key.as_ref(),
        )?;

        // Charge protocol fees.
        T::ProtocolFee::charge_fees(&{
            let mut fees = ArrayVec::<_, 2>::new();
            if available {
                fees.push(ProtocolOp::AssetRegisterTicker);
                fees.push(ProtocolOp::AssetCreateAsset);
            }
            fees
        })?;

        //==========================================================================
        // At this point all checks have been made; **only** storage changes follow!
        //==========================================================================

        Identity::<T>::commit_token_did(token_did, ticker);

        // Register the ticker or finish its registration.
        if available {
            // Ticker not registered by anyone (or registry expired), so register.
            Self::unverified_register_ticker(&ticker, did, None);
        } else {
            // Ticker already registered by the user.
            <Tickers<T>>::mutate(&ticker, |tr| {
                if let Some(tr) = tr {
                    tr.expiry = None;
                }
            });
        }

        let token = SecurityToken {
            total_supply: Zero::zero(),
            owner_did: did,
            divisible,
            asset_type,
        };
        Tokens::insert(&ticker, token);
        AssetNames::insert(&ticker, &name);
        // NB - At the time of asset creation it is obvious that the asset issuer will not have an
        // `InvestorUniqueness` claim. So we are skipping the scope claim based stats update as
        // those data points will get added in to the system whenever the asset issuer
        // has an InvestorUniqueness claim. This also applies when issuing assets.
        AssetOwnershipRelations::insert(did, ticker, AssetOwnershipRelation::AssetOwned);
        Self::deposit_event(RawEvent::AssetCreated(
            did,
            ticker,
            divisible,
            asset_type,
            did,
            name,
            identifiers.clone(),
            funding_round.clone(),
        ));

        // Add funding round name.
        if let Some(funding_round) = funding_round {
            FundingRound::insert(ticker, funding_round);
        }

        Self::unverified_update_idents(did, ticker, identifiers);

        // Grant owner full agent permissions.
        <ExternalAgents<T>>::unchecked_add_agent(ticker, did, AgentGroup::Full).unwrap();

        Ok(did)
    }

    /// Update identitifiers of `ticker` as `did`.
    ///
    /// Does not verify that actor `did` is permissioned for this call or that `idents` are valid.
    pub(crate) fn unverified_update_idents(
        did: IdentityId,
        ticker: Ticker,
        idents: Vec<AssetIdentifier>,
    ) {
        Identifiers::insert(ticker, idents.clone());
        Self::deposit_event(RawEvent::IdentifiersUpdated(did, ticker, idents));
    }

    pub(crate) fn _mint(
        ticker: &Ticker,
        to_did: IdentityId,
        value: Balance,
        protocol_fee_data: Option<ProtocolOp>,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        Self::ensure_granular(ticker, value)?;
        // Read the token details
        let mut token = Self::token_details(ticker)?;
        // Ensures the token is fungible
        ensure!(
            token.asset_type.is_fungible(),
            Error::<T>::UnexpectedNonFungibleToken
        );

        // Prepare the updated total supply.
        let updated_total_supply = token
            .total_supply
            .checked_add(value)
            .ok_or(Error::<T>::TotalSupplyOverflow)?;
        Self::ensure_within_max_supply(updated_total_supply)?;
        // Increase receiver balance.
        let current_to_balance = Self::balance_of(ticker, to_did);
        // No check since the total balance is always <= the total supply. The
        // total supply is already checked above.
        let updated_to_balance = current_to_balance + value;
        // No check since the default portfolio balance is always <= the total
        // supply. The total supply is already checked above.
        let updated_to_def_balance = Portfolio::<T>::portfolio_asset_balances(
            PortfolioId::default_portfolio(to_did),
            ticker,
        ) + value;

        // In transaction because we don't want fee to be charged if advancing fails.
        with_transaction(|| {
            // Charge the fee.
            if let Some(op) = protocol_fee_data {
                T::ProtocolFee::charge_fee(op)?;
            }

            // Advance checkpoint schedules and update last checkpoint.
            <Checkpoint<T>>::advance_update_balances(ticker, &[(to_did, current_to_balance)])
        })?;

        // Increase total supply.
        token.total_supply = updated_total_supply;
        BalanceOf::insert(ticker, &to_did, updated_to_balance);
        Portfolio::<T>::set_default_portfolio_balance(to_did, ticker, updated_to_def_balance);
        Tokens::insert(ticker, token);

        Statistics::<T>::update_asset_stats(
            &ticker,
            None,
            Some(&to_did),
            None,
            Some(updated_to_balance),
            value,
            weight_meter,
        )?;

        let round = Self::funding_round(ticker);
        let ticker_round = (*ticker, round.clone());
        // No check since the issued balance is always <= the total
        // supply. The total supply is already checked above.
        let issued_in_this_round = Self::issued_in_funding_round(&ticker_round) + value;
        IssuedInFundingRound::insert(&ticker_round, issued_in_this_round);

        Self::deposit_event(RawEvent::AssetBalanceUpdated(
            to_did,
            *ticker,
            value,
            None,
            Some(PortfolioId::default_portfolio(to_did)),
            PortfolioUpdateReason::Issued {
                funding_round_name: Some(round),
            },
        ));
        Ok(())
    }

    // Transfers tokens from one identity to another
    pub fn unsafe_transfer(
        from_portfolio: PortfolioId,
        to_portfolio: PortfolioId,
        ticker: &Ticker,
        value: Balance,
        instruction_id: Option<InstructionId>,
        instruction_memo: Option<Memo>,
        caller_did: IdentityId,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResult {
        Self::ensure_granular(ticker, value)?;

        let token = Self::token_details(ticker)?;
        // Ensures the token is fungible
        ensure!(
            token.asset_type.is_fungible(),
            Error::<T>::UnexpectedNonFungibleToken
        );

        ensure!(
            from_portfolio.did != to_portfolio.did,
            Error::<T>::SenderSameAsReceiver
        );

        let from_total_balance = Self::balance_of(ticker, from_portfolio.did);
        ensure!(from_total_balance >= value, Error::<T>::InsufficientBalance);
        let updated_from_total_balance = from_total_balance - value;

        let to_total_balance = Self::balance_of(ticker, to_portfolio.did);
        let updated_to_total_balance = to_total_balance
            .checked_add(value)
            .ok_or(Error::<T>::BalanceOverflow)?;

        <Checkpoint<T>>::advance_update_balances(
            ticker,
            &[
                (from_portfolio.did, from_total_balance),
                (to_portfolio.did, to_total_balance),
            ],
        )?;

        // reduce sender's balance
        BalanceOf::insert(ticker, &from_portfolio.did, updated_from_total_balance);
        // increase receiver's balance
        BalanceOf::insert(ticker, &to_portfolio.did, updated_to_total_balance);
        // transfer portfolio balances
        Portfolio::<T>::unchecked_transfer_portfolio_balance(
            &from_portfolio,
            &to_portfolio,
            ticker,
            value,
        );

        // Update statistic info.
        Statistics::<T>::update_asset_stats(
            ticker,
            Some(&from_portfolio.did),
            Some(&to_portfolio.did),
            Some(updated_from_total_balance),
            Some(updated_to_total_balance),
            value,
            weight_meter,
        )?;

        Self::deposit_event(RawEvent::AssetBalanceUpdated(
            caller_did,
            *ticker,
            value,
            Some(from_portfolio),
            Some(to_portfolio),
            PortfolioUpdateReason::Transferred {
                instruction_id,
                instruction_memo,
            },
        ));
        Ok(())
    }

    pub(crate) fn unsafe_register_custom_asset_type(
        did: IdentityId,
        ty: Vec<u8>,
    ) -> Result<CustomAssetTypeId, DispatchError> {
        ensure_string_limited::<T>(&ty)?;

        Ok(match CustomTypesInverse::try_get(&ty) {
            Ok(id) => {
                Self::deposit_event(Event::<T>::CustomAssetTypeExists(did, id, ty));
                id
            }
            Err(()) => {
                let id = CustomTypeIdSequence::try_mutate(try_next_pre::<T, _>)?;
                CustomTypesInverse::insert(&ty, id);
                CustomTypes::insert(id, ty.clone());
                Self::deposit_event(Event::<T>::CustomAssetTypeRegistered(did, id, ty));
                id
            }
        })
    }

    pub(crate) fn unverified_set_asset_metadata(
        did: IdentityId,
        ticker: Ticker,
        key: AssetMetadataKey,
        value: AssetMetadataValue,
        detail: Option<AssetMetadataValueDetail<T::Moment>>,
    ) -> DispatchResult {
        // Check value length limit.
        Self::ensure_asset_metadata_value_limited(&value)?;

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

        // Set asset metadata value for asset.
        AssetMetadataValues::insert(ticker, key, &value);

        // Set asset metadata value details.
        if let Some(ref detail) = detail {
            AssetMetadataValueDetails::<T>::insert(ticker, key, detail);
        }

        Self::deposit_event(RawEvent::SetAssetMetadataValue(did, ticker, value, detail));
        Ok(())
    }

    pub(crate) fn unverified_register_asset_metadata_local_type(
        did: IdentityId,
        ticker: Ticker,
        name: AssetMetadataName,
        spec: AssetMetadataSpec,
    ) -> Result<AssetMetadataKey, DispatchError> {
        Self::ensure_asset_metadata_name_limited(&name)?;
        Self::ensure_asset_metadata_spec_limited(&spec)?;

        // Check if key already exists.
        ensure!(
            !AssetMetadataLocalNameToKey::contains_key(ticker, &name),
            Error::<T>::AssetMetadataLocalKeyAlreadyExists
        );

        // Next local key for asset.
        let key = AssetMetadataNextLocalKey::try_mutate(ticker, try_next_pre::<T, _>)?;

        // Store local key <-> name mapping.
        AssetMetadataLocalNameToKey::insert(ticker, &name, key);
        AssetMetadataLocalKeyToName::insert(ticker, key, &name);

        // Store local specs.
        AssetMetadataLocalSpecs::insert(ticker, key, &spec);

        Self::deposit_event(RawEvent::RegisterAssetMetadataLocalType(
            did, ticker, name, key, spec,
        ));
        Ok(key.into())
    }
}
