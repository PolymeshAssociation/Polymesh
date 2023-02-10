#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use ink_lang as ink;

use polymesh_ink::*;

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
        /// The `AccountId` of a privileged account that override the
        /// code hash for `PolymeshInk`.
        ///
        /// This address is set to the account that instantiated this contract.
        admin: AccountId,
        /// Upgradable Polymesh Ink API.
        api: PolymeshInk,

        /// Ticker pair.
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
        /// PolymeshInk errors.
        PolymeshInk(PolymeshError),
        /// Upgrade error.
        UpgradeError(UpgradeError),
        /// Caller needs to pay the contract for the protocol fee.
        /// (Amount needed)
        InsufficientTransferValue(Balance),
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
            Self::PolymeshInk(err)
        }
    }

    impl From<UpgradeError> for Error {
        fn from(err: UpgradeError) -> Self {
            Self::UpgradeError(err)
        }
    }

    /// The contract result type.
    pub type Result<T> = core::result::Result<T, Error>;

    impl Settlements {
        /// Creates a new contract.
        #[ink(constructor)]
        pub fn new(ticker1: Ticker, ticker2: Ticker, hash: Hash, tracker: Option<UpgradeTrackerRef>) -> Self {
            ink_lang::utils::initialize_contract(|contract| {
                Self::new_init(contract, ticker1, ticker2, hash, tracker)
            })
        }

        fn new_init(&mut self, ticker1: Ticker, ticker2: Ticker, hash: Hash, tracker: Option<UpgradeTrackerRef>) {
            self.admin = Self::env().caller();
            self.api = PolymeshInk::new(hash, tracker);
            self.ticker1 = ticker1;
            self.ticker2 = ticker2;
            // The contract should always have an identity.
            self.did = self.get_did(Self::env().account_id()).unwrap();
            self.initialized = false;
        }

        /// Update the code hash of the polymesh runtime API.
        ///
        /// Only the `admin` is allowed to call this.
        #[ink(message)]
        pub fn update_code_hash(&mut self, hash: Hash) {
            assert_eq!(
                self.env().caller(),
                self.admin,
                "caller {:?} does not have sufficient permissions, only {:?} does",
                self.env().caller(),
                self.admin,
            );
            self.api.update_code_hash(hash);
        }

        /// Update the `polymesh-ink` API using the tracker.
        ///
        /// Anyone can pay the gas fees to do the update using the tracker.
        #[ink(message)]
        pub fn update_polymesh_ink(&mut self) -> Result<()> {
            self.api.check_for_upgrade()?;
            Ok(())
        }

        fn create_asset(&mut self, ticker: Ticker) -> Result<()> {
            self.api
                .asset_create_and_issue(
                    AssetName(b"".to_vec()),
                    ticker,
                    AssetType::EquityCommon,
                    true, // Divisible token.
                    Some(1_000_000 * UNIT),
                )?;
            Ok(())
        }

        fn get_did(&self, acc: AccountId) -> Result<IdentityId> {
            Ok(self.api.get_key_did(acc)?)
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

            // Create venue.
            self.venue = self.api.create_venue(
                    VenueDetails(b"Contract Venue".to_vec()),
                    VenueType::Other,
                )?;
            self.initialized = true;
            Ok(())
        }

        fn transfer_assets(&self, legs: Vec<Leg>, portfolios: Vec<PortfolioId>) -> Result<()> {
            self.api.settlement_execute(
                self.venue,
                legs,
                portfolios
                )?;
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
                    Leg {
                        from: our_portfolio,
                        to: caller_portfolio,
                        asset: self.ticker1,
                        amount: 10 * UNIT,
                    },
                    Leg {
                        from: our_portfolio,
                        to: caller_portfolio,
                        asset: self.ticker2,
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

            self.api.accept_portfolio_custody(auth_id, portfolio)?;
            let portfolio = PortfolioId {
                did: caller_did,
                kind: portfolio,
            };
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

            self.api.move_portfolio_funds(
                caller_portfolio, // Contract controlled portfolio.
                dest,            // Caller controlled portfolio.
                vec![MovePortfolioItem {
                    ticker: ticker,
                    amount,
                    memo: None,
                }],
            )?;
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

            // Remove our custodianship.
            self.api.quit_portfolio_custody(portfolio)?;
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
                    Leg {
                        from: caller_portfolio,
                        to: our_portfolio,
                        asset: sell,
                        amount: sell_amount,
                    },
                    Leg {
                        from: our_portfolio,
                        to: caller_portfolio,
                        asset: buy,
                        amount: buy_amount,
                    },
                ],
                vec![our_portfolio, caller_portfolio],
            )?;

            Ok(())
        }
    }
}
