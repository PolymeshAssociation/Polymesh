use frame_support::dispatch::DispatchError;

use polymesh_common_utilities::compliance_manager::ComplianceFnConfig;
use polymesh_common_utilities::traits::asset::Config;

use polymesh_primitives::asset::GranularCanTransferResult;
use polymesh_primitives::transfer_compliance::TransferConditionResult;
use polymesh_primitives::{Balance, IdentityId, PortfolioId, Ticker, WeightMeter};

use crate::{Identity, Module, Portfolio, Statistics, Vec};

impl<T: Config> Module<T> {
    pub fn unsafe_can_transfer_granular(
        from_custodian: Option<IdentityId>,
        from_portfolio: PortfolioId,
        to_custodian: Option<IdentityId>,
        to_portfolio: PortfolioId,
        ticker: &Ticker,
        value: Balance,
        weight_meter: &mut WeightMeter,
    ) -> Result<GranularCanTransferResult, DispatchError> {
        let invalid_granularity = Self::invalid_granularity(ticker, value);
        let self_transfer = Self::self_transfer(&from_portfolio, &to_portfolio);
        let invalid_receiver_cdd = Self::invalid_cdd(to_portfolio.did);
        let invalid_sender_cdd = Self::invalid_cdd(from_portfolio.did);
        let receiver_custodian_error =
            Self::custodian_error(to_portfolio, to_custodian.unwrap_or(to_portfolio.did));
        let sender_custodian_error =
            Self::custodian_error(from_portfolio, from_custodian.unwrap_or(from_portfolio.did));
        let sender_insufficient_balance =
            Self::insufficient_balance(&ticker, from_portfolio.did, value);
        let portfolio_validity_result = <Portfolio<T>>::ensure_portfolio_transfer_validity_granular(
            &from_portfolio,
            &to_portfolio,
            ticker,
            value,
        );
        let asset_frozen = Self::frozen(ticker);
        let transfer_condition_result = Self::transfer_condition_failures_granular(
            &from_portfolio.did,
            &to_portfolio.did,
            ticker,
            value,
            weight_meter,
        )?;
        let compliance_result = T::ComplianceManager::verify_restriction_granular(
            ticker,
            Some(from_portfolio.did),
            Some(to_portfolio.did),
            weight_meter,
        )?;

        Ok(GranularCanTransferResult {
            invalid_granularity,
            self_transfer,
            invalid_receiver_cdd,
            invalid_sender_cdd,
            receiver_custodian_error,
            sender_custodian_error,
            sender_insufficient_balance,
            asset_frozen,
            result: !invalid_granularity
                && !self_transfer
                && !invalid_receiver_cdd
                && !invalid_sender_cdd
                && !receiver_custodian_error
                && !sender_custodian_error
                && !sender_insufficient_balance
                && portfolio_validity_result.result
                && !asset_frozen
                && transfer_condition_result.iter().all(|result| result.result)
                && compliance_result.result,
            transfer_condition_result,
            compliance_result,
            consumed_weight: Some(weight_meter.consumed()),
            portfolio_validity_result,
        })
    }
}

impl<T: Config> Module<T> {
    fn invalid_granularity(ticker: &Ticker, value: Balance) -> bool {
        !Self::check_granularity(&ticker, value)
    }

    fn self_transfer(from: &PortfolioId, to: &PortfolioId) -> bool {
        from.did == to.did
    }

    fn invalid_cdd(did: IdentityId) -> bool {
        !Identity::<T>::has_valid_cdd(did)
    }

    fn custodian_error(from: PortfolioId, custodian: IdentityId) -> bool {
        Portfolio::<T>::ensure_portfolio_custody(from, custodian).is_err()
    }

    fn insufficient_balance(ticker: &Ticker, did: IdentityId, value: Balance) -> bool {
        Self::balance_of(&ticker, did) < value
    }

    fn transfer_condition_failures_granular(
        from_did: &IdentityId,
        to_did: &IdentityId,
        ticker: &Ticker,
        value: Balance,
        weight_meter: &mut WeightMeter,
    ) -> Result<Vec<TransferConditionResult>, DispatchError> {
        let total_supply = Self::total_supply(ticker);
        Statistics::<T>::get_transfer_restrictions_results(
            ticker,
            from_did,
            to_did,
            Self::balance_of(ticker, from_did),
            Self::balance_of(ticker, to_did),
            value,
            total_supply,
            weight_meter,
        )
    }
}
