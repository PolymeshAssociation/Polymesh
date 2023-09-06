#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use ink_lang as ink;

use polymesh_api::{
    ink::{
        basic_types::IdentityId,
        extension::{PolymeshEnvironment, PolymeshRuntimeErr},
        Error as PolymeshError,
    },
    polymesh::types::{
        polymesh_primitives::{
            asset::{AssetName, AssetType},
            identity_id::{PortfolioId, PortfolioKind},
            portfolio::{Fund, FundDescription},
            settlement::{Leg, SettlementType, VenueDetails, VenueId, VenueType},
            ticker::Ticker,
        },
    },
    Api,
};

#[ink::contract(env = PolymeshEnvironment)]
mod settlements {
    use alloc::vec;
    use ink_storage::{traits::SpreadAllocate, Mapping};

    use crate::*;

    pub const UNIT: Balance = 1_000_000u128;

    /// A contract that uses the settlements pallet.
    #[ink(storage)]
    #[derive(Default, SpreadAllocate)]
    pub struct Settlements {
        ticker1: Ticker,
        ticker2: Ticker,
        initialized: bool,
        /// Venue for settlements.
        venue: VenueId,
        /// Contract's identity.
        did: IdentityId,
        /// Custodial portfolios.
        portfolios: Mapping<IdentityId, PortfolioId>,
    }

    /// The contract error types.
    #[derive(Debug, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Caller needs to pay the contract for the protocol fee.
        /// (Amount needed)
        InsufficientTransferValue(Balance),
        /// Polymesh runtime error.
        PolymeshError(PolymeshError),
        /// Scale decode failed.
        ScaleError,
        /// Missing Identity.  MultiSig's are not supported.
        MissingIdentity,
        /// Contract hasn't been initialized.
        NotInitialized,
        /// Contract has already been initialized.
        AlreadyInitialized,
        /// Invalid portfolio authorization.
        InvalidPortfolioAuthorization,
        /// The caller has already initialized a portfolio.
        AlreadyHavePortfolio,
        /// The caller doesn't have a portfolio yet.
        NoPortfolio,
        /// Invalid ticker.
        InvalidTicker,
    }

    impl From<PolymeshError> for Error {
        fn from(err: PolymeshError) -> Self {
            Self::PolymeshError(err)
        }
    }

    impl From<PolymeshRuntimeErr> for Error {
        fn from(err: PolymeshRuntimeErr) -> Self {
            Self::PolymeshError(err.into())
        }
    }

    /// The contract result type.
    pub type Result<T> = core::result::Result<T, Error>;

    impl Settlements {
        /// Creates a new contract.
        #[ink(constructor)]
        pub fn new(ticker1: Ticker, ticker2: Ticker) -> Self {
            ink_lang::utils::initialize_contract(|contract| {
                Self::new_init(contract, ticker1, ticker2)
            })
        }

        fn new_init(&mut self, ticker1: Ticker, ticker2: Ticker) {
            self.ticker1 = ticker1;
            self.ticker2 = ticker2;
            // The contract should always have an identity.
            self.did = self.get_did(Self::env().account_id()).unwrap();
            self.initialized = false;
        }

        fn create_asset(&mut self, ticker: Ticker) -> Result<()> {
            let api = Api::new();
            // Create asset.
            api.call()
                .asset()
                .create_asset(
                    AssetName(b"".to_vec()),
                    ticker.into(),
                    true, // Divisible token.
                    AssetType::EquityCommon,
                    vec![],
                    None,
                )
                .submit()?;
            // Mint some tokens.
            api.call()
                .asset()
                .issue(ticker.into(), 1_000_000 * UNIT, PortfolioKind::Default)
                .submit()?;
            // Pause compliance rules to allow transfers.
            api.call()
                .compliance_manager()
                .pause_asset_compliance(ticker.into())
                .submit()?;
            Ok(())
        }

        fn get_did(&self, acc: AccountId) -> Result<IdentityId> {
            Self::env()
                .extension()
                .get_key_did(acc)?
                .map(|did| did.into())
                .ok_or(Error::MissingIdentity)
        }

        fn get_caller_did(&self) -> Result<IdentityId> {
            self.get_did(Self::env().caller())
        }

        fn ensure_ticker(&self, ticker: Ticker) -> Result<()> {
            if self.ticker1 != ticker && self.ticker2 != ticker {
                Err(Error::InvalidTicker)
            } else {
                Ok(())
            }
        }

        fn ensure_has_portfolio(&self, did: IdentityId) -> Result<PortfolioId> {
            self.portfolios.get(did).ok_or(Error::NoPortfolio)
        }

        fn ensure_no_portfolio(&self, did: IdentityId) -> Result<()> {
            if self.portfolios.get(did).is_some() {
                return Err(Error::AlreadyHavePortfolio);
            }
            Ok(())
        }

        fn ensure_initialized(&self) -> Result<()> {
            if !self.initialized {
                return Err(Error::NotInitialized);
            }
            Ok(())
        }

        fn init_venue(&mut self) -> Result<()> {
            if self.initialized {
                return Err(Error::AlreadyInitialized);
            }
            // Create tickers.
            self.create_asset(self.ticker1)?;
            self.create_asset(self.ticker2)?;

            let api = Api::new();
            // Get the next venue id.
            let id = api.query().settlement().venue_counter().map(|v| v.into())?;
            // Create Venue.
            api.call()
                .settlement()
                .create_venue(
                    VenueDetails(b"Contract Venue".to_vec()),
                    vec![],
                    VenueType::Other,
                )
                .submit()?;
            // Save venue id.
            self.venue = id;
            self.initialized = true;
            Ok(())
        }

        fn transfer_assets(&self, legs: Vec<Leg>, portfolios: Vec<PortfolioId>) -> Result<()> {
            let leg_count = legs.len() as u32;
            let api = Api::new();
            // Get the next instruction id.
            let instruction_id = api
                .query()
                .settlement()
                .instruction_counter()
                .map(|v| v.into())?;
            // Create settlement.
            api.call()
                .settlement()
                .add_and_affirm_instruction(
                    self.venue,
                    SettlementType::SettleManual(0),
                    None,
                    None,
                    legs,
                    portfolios,
                    None,
                )
                .submit()?;

            // Create settlement.
            api.call()
                .settlement()
                .execute_manual_instruction(instruction_id, None, leg_count, 0, 0, None)
                .submit()?;
            Ok(())
        }

        #[ink(message)]
        pub fn init(&mut self) -> Result<()> {
            self.init_venue()
        }

        #[ink(message)]
        pub fn venue(&self) -> Result<VenueId> {
            self.ensure_initialized()?;
            Ok(self.venue)
        }

        #[ink(message)]
        pub fn contract_did(&self) -> Result<IdentityId> {
            self.ensure_initialized()?;
            Ok(self.did)
        }

        fn fund_caller(&self) -> Result<()> {
            // Get the caller's identity.
            let caller_did = self.get_caller_did()?;

            // Ensure the caller has a portfolio.
            let caller_portfolio = self.ensure_has_portfolio(caller_did)?;

            // Transfer some tokens to the caller's portfolio.
            let our_portfolio = PortfolioId {
                did: self.did,
                kind: PortfolioKind::Default,
            };
            self.transfer_assets(
                vec![
                    Leg::Fungible {
                        sender: our_portfolio,
                        receiver: caller_portfolio,
                        ticker: self.ticker1,
                        amount: 10 * UNIT,
                    },
                    Leg::Fungible {
                        sender: our_portfolio,
                        receiver: caller_portfolio,
                        ticker: self.ticker2,
                        amount: 20 * UNIT,
                    },
                ],
                vec![our_portfolio, caller_portfolio],
            )?;

            Ok(())
        }

        #[ink(message)]
        /// Accept custody of a portfolio and give the caller some tokens.
        pub fn add_portfolio(&mut self, auth_id: u64, portfolio: PortfolioKind) -> Result<()> {
            self.ensure_initialized()?;
            // Get the caller's identity.
            let caller_did = self.get_caller_did()?;
            // Ensure the caller doesn't have a portfolio.
            self.ensure_no_portfolio(caller_did)?;

            let portfolio = PortfolioId {
                did: caller_did,
                kind: portfolio,
            };
            let api = Api::new();
            // Accept authorization.
            api.call()
                .portfolio()
                .accept_portfolio_custody(auth_id)
                .submit()?;
            // Check that we are the custodian.
            if !api
                .query()
                .portfolio()
                .portfolios_in_custody(self.did, portfolio)?
            {
                return Err(Error::InvalidPortfolioAuthorization);
            }
            // Save the caller's portfolio.
            self.portfolios.insert(caller_did, &portfolio);

            // Give the caller some funds.
            self.fund_caller()?;
            Ok(())
        }

        #[ink(message)]
        /// Allow the caller to withdrawal funds from the contract controlled portfolio.
        pub fn withdrawal(
            &mut self,
            ticker: Ticker,
            amount: Balance,
            dest: PortfolioKind,
        ) -> Result<()> {
            self.ensure_initialized()?;
            self.ensure_ticker(ticker)?;

            // Get the caller's identity.
            let caller_did = self.get_caller_did()?;
            let dest = PortfolioId {
                did: caller_did,
                kind: dest,
            };

            // Ensure the caller has a portfolio.
            let caller_portfolio = self.ensure_has_portfolio(caller_did)?;

            let api = Api::new();
            // Move funds out of the contract controlled portfolio.
            api.call()
                .portfolio()
                .move_portfolio_funds(
                    caller_portfolio, // Contract controlled portfolio.
                    dest.into(),      // Caller controlled portfolio.
                    vec![Fund {
                        description: FundDescription::Fungible {
                            ticker: ticker,
                            amount,
                        },
                        memo: None,
                    }],
                )
                .submit()?;
            Ok(())
        }

        #[ink(message)]
        /// Return the caller's portfolio custodianship back to them.
        pub fn withdrawal_all(&mut self) -> Result<()> {
            self.ensure_initialized()?;

            // Get the caller's identity.
            let caller_did = self.get_caller_did()?;

            // Ensure the caller has a portfolio.
            let portfolio = self.ensure_has_portfolio(caller_did)?;

            let api = Api::new();
            // Remove our custodianship.
            api.call()
                .portfolio()
                .quit_portfolio_custody(portfolio)
                .submit()?;
            // Remove the portfolio.
            self.portfolios.remove(caller_did);

            Ok(())
        }

        #[ink(message)]
        /// Trade.
        pub fn trade(
            &mut self,
            sell: Ticker,
            sell_amount: Balance,
            buy: Ticker,
            buy_amount: Balance,
        ) -> Result<()> {
            self.ensure_initialized()?;
            self.ensure_ticker(sell)?;
            self.ensure_ticker(buy)?;

            // Get the caller's identity.
            let caller_did = self.get_caller_did()?;

            // Ensure the caller has a portfolio.
            let caller_portfolio = self.ensure_has_portfolio(caller_did)?;

            // Use settlement to complete the trade.
            let our_portfolio = PortfolioId {
                did: self.did,
                kind: PortfolioKind::Default,
            };
            self.transfer_assets(
                vec![
                    Leg::Fungible {
                        sender: caller_portfolio,
                        receiver: our_portfolio,
                        ticker: sell,
                        amount: sell_amount,
                    },
                    Leg::Fungible {
                        sender: our_portfolio,
                        receiver: caller_portfolio,
                        ticker: buy,
                        amount: buy_amount,
                    },
                ],
                vec![our_portfolio, caller_portfolio],
            )?;

            Ok(())
        }
    }
}
