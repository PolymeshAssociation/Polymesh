#![cfg_attr(not(feature = "std"), no_std, no_main)]

extern crate alloc;

use polymesh_ink::*;

#[ink::contract(env = PolymeshEnvironment)]
mod wrapped_polyx {
    use alloc::vec;
    use ink::storage::Mapping;

    use crate::*;

    /// A contract to wrap and unwrap POLYX (native token)
    #[ink(storage)]
    #[derive(Default)]
    pub struct WrappedPolyx {
        initialized: bool,
        /// Upgradable Polymesh Ink API.
        api: PolymeshInk,
        /// WrappedPolyx token.
        ticker: Ticker,
        /// Venue for settlements.
        venue: VenueId,
        /// Contract's identity.
        did: IdentityId,
        /// Custodial portfolios.
        portfolios: Mapping<IdentityId, PortfolioId>,
    }

    /// Event emitted when POLYX is wrapped
    #[ink(event)]
    pub struct PolyxWrapped {
        #[ink(topic)]
        did: IdentityId,
        #[ink(topic)]
        key: AccountId,
        amount: Balance,
    }

    /// Event emitted when POLYX is unwrapped
    #[ink(event)]
    pub struct PolyxUnwrapped {
        #[ink(topic)]
        did: IdentityId,
        #[ink(topic)]
        key: AccountId,
        amount: Balance,
    }

    /// Event emitted when POLYX is unwrapped
    #[ink(event)]
    pub struct PortfolioAdded {
        #[ink(topic)]
        did: IdentityId,
        portfolio_kind: PortfolioKind,
    }

    /// Event emitted when POLYX is unwrapped
    #[ink(event)]
    pub struct PortfolioRemoved {
        #[ink(topic)]
        did: IdentityId,
        portfolio_kind: PortfolioKind,
    }

    /// The contract error types.
    #[derive(Debug, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// PolymeshInk errors.
        PolymeshInk(PolymeshError),
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
    }

    impl From<PolymeshError> for Error {
        fn from(err: PolymeshError) -> Self {
            Self::PolymeshInk(err)
        }
    }

    /// The contract result type.
    pub type Result<T> = core::result::Result<T, Error>;

    impl WrappedPolyx {
        /// Creates a new contract.
        #[ink(constructor)]
        pub fn new(ticker: Ticker) -> Result<Self> {
            Ok(Self {
                api: PolymeshInk::new()?,
                ticker,
                did: PolymeshInk::get_our_did()?,
                ..Default::default()
            })
        }

        fn create_wrapped_polyx(&mut self) -> Result<()> {
            self.api.asset_create_and_issue(
                AssetName(b"Wrapped POLYX".to_vec()),
                self.ticker,
                AssetType::EquityCommon,
                true, // Divisible token.
                None,
            )?;
            Ok(())
        }

        fn ensure_has_portfolio(&self) -> Result<PortfolioId> {
            // Get the caller's identity.
            let caller_did = PolymeshInk::get_caller_did()?;
            self.portfolios.get(caller_did).ok_or(Error::NoPortfolio)
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

        fn init_asset_and_venue(&mut self) -> Result<()> {
            if self.initialized {
                return Err(Error::AlreadyInitialized);
            }
            // Update our identity id.
            self.did = PolymeshInk::get_our_did()?;
            // Create ticker.
            self.create_wrapped_polyx()?;

            // Create venue.
            self.venue = self
                .api
                .create_venue(VenueDetails(b"Contract Venue".to_vec()), VenueType::Other)?;
            self.initialized = true;
            Ok(())
        }

        #[ink(message, payable)]
        pub fn init(&mut self) -> Result<()> {
            self.init_asset_and_venue()
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

        #[ink(message)]
        /// Accept custody of a portfolio and give the caller some tokens.
        pub fn add_portfolio(&mut self, auth_id: u64, portfolio: PortfolioKind) -> Result<()> {
            self.ensure_initialized()?;
            // Accept portfolio custody and ensure we have custody.
            let portfolio_id = self.api.accept_portfolio_custody(auth_id, portfolio)?;
            let caller_did = portfolio_id.did;
            // Ensure the caller doesn't have a portfolio.
            self.ensure_no_portfolio(caller_did)?;
            // Save the caller's portfolio.
            self.portfolios.insert(caller_did, &portfolio_id);

            Self::env().emit_event(PortfolioAdded {
                did: caller_did,
                portfolio_kind: portfolio,
            });

            Ok(())
        }

        fn transfer(&self, sender: PortfolioId, receiver: PortfolioId, amount: Balance) -> Result<()> {
            self.api.settlement_execute(self.venue, vec![Leg::Fungible {
                sender,
                receiver,
                ticker: self.ticker,
                amount: amount,
            }], vec![sender, receiver])?;
            Ok(())
        }

        #[ink(message, payable)]
        /// Allow the caller to withdrawal funds from the contract controlled portfolio.
        pub fn mint_wrapped_polyx(&mut self) -> Result<()> {
            self.ensure_initialized()?;
            let amount = Self::env().transferred_value();

            // Ensure the caller has a portfolio.
            let caller_portfolio = self.ensure_has_portfolio()?;
            let caller_did = caller_portfolio.did;

            // Mint some tokens.
            self.api.asset_issue(self.ticker, amount, PortfolioKind::Default)?;
            // Transfer tokens to the caller's portfolio.
            let our_portfolio = PortfolioId {
                did: self.did,
                kind: PortfolioKind::Default,
            };
            self.transfer(our_portfolio, caller_portfolio, amount)?;

            Self::env().emit_event(PolyxWrapped {
                did: caller_did,
                key: Self::env().caller(),
                amount: amount,
            });
            Ok(())
        }

        #[ink(message)]
        /// Allow the caller to withdrawal funds from the contract controlled portfolio.
        pub fn burn_wrapped_polyx(&mut self, amount: Balance) -> Result<()> {
            self.ensure_initialized()?;

            // Ensure the caller has a portfolio.
            let caller_portfolio = self.ensure_has_portfolio()?;
            let caller_did = caller_portfolio.did;

            // Transfer tokens from the caller's portfolio.
            let our_portfolio = PortfolioId {
                did: self.did,
                kind: PortfolioKind::Default,
            };
            self.transfer(caller_portfolio, our_portfolio, amount)?;

            // Redeem the tokens.
            self.api.asset_redeem(self.ticker, amount, PortfolioKind::Default)?;

            if Self::env()
                .transfer(Self::env().caller(), amount)
                .is_err()
            {
                panic!("error transferring")
            }

            Self::env().emit_event(PolyxUnwrapped {
                did: caller_did,
                key: Self::env().caller(),
                amount: amount,
            });

            Ok(())
        }

        #[ink(message)]
        /// Return the caller's portfolio custodianship back to them.
        pub fn remove_portfolio(&mut self) -> Result<()> {
            self.ensure_initialized()?;

            // Ensure the caller has a portfolio.
            let portfolio = self.ensure_has_portfolio()?;
            let caller_did = portfolio.did;

            // Remove our custodianship.
            self.api.quit_portfolio_custody(portfolio)?;
            // Remove the portfolio.
            self.portfolios.remove(caller_did);
            Self::env().emit_event(PortfolioRemoved {
                did: caller_did,
                portfolio_kind: portfolio.kind,
            });

            Ok(())
        }

        #[ink(message)]
        /// Allow the caller to withdrawal funds from the contract controlled portfolio.
        pub fn withdraw_polyx(&mut self, amount: Balance, dest: PortfolioKind) -> Result<()> {
            self.ensure_initialized()?;

            // Ensure the caller has a portfolio.
            let caller_portfolio = self.ensure_has_portfolio()?;
            let dest = PortfolioId {
                did: caller_portfolio.did,
                kind: dest,
            };

            // Move funds out of the contract controlled portfolio.
            self.api.move_portfolio_funds(caller_portfolio, dest, vec![
                Fund {
                    description: FundDescription::Fungible {
                        ticker: self.ticker,
                        amount,
                    },
                    memo: None,
                }
            ])?;
            Ok(())
        }
    }
}
