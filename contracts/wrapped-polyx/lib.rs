#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use ink_lang as ink;

use polymesh_api::{
  Api,
  ink::{
    extension::{PolymeshEnvironment, PolymeshRuntimeErr},
    basic_types::IdentityId,
    Error as PolymeshError,
  },
  polymesh::types::{
    pallet_portfolio::MovePortfolioItem,
    pallet_settlement::{
      VenueId,
      VenueDetails,
      VenueType,
      SettlementType,
      Leg,
    },
    polymesh_primitives::{
      ticker::Ticker,
      asset::{
        AssetName,
        AssetType,
      },
      identity_id::{
        PortfolioId,
        PortfolioKind,
      },
    },
  },
};

#[ink::contract(env = PolymeshEnvironment)]
mod wrapped_polyx {
    use ink_storage::{
        traits::{
            SpreadAllocate,
        },
        Mapping,
    };
    use alloc::vec;

    use crate::*;

    /// A contract to wrap and unwrap POLYX (native token)
    #[ink(storage)]
    #[derive(Default, SpreadAllocate)]
    pub struct WrappedPolyx {
      ticker: Ticker,
      initialized: bool,
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

    impl WrappedPolyx {
        /// Creates a new contract.
        #[ink(constructor)]
        pub fn new(ticker: Ticker) -> Self {
          ink_lang::utils::initialize_contract(|contract| {
              Self::new_init(contract, ticker)
          })
        }

        fn new_init(&mut self, ticker: Ticker) {
          // The contract should always have an identity.
          self.did = self.get_did(Self::env().account_id()).unwrap();
          self.ticker = ticker;
          self.initialized = false;
        }

        fn create_wrapped_polyx(&mut self) -> Result<()> {
            let api = Api::new();
            // Create asset.
            api.call().asset().create_asset(
              AssetName(b"Wrapped POLYX".to_vec()),
              self.ticker,
              true, // Divisible token.
              //TODO: Create Other asset type for wrapped tokens
              AssetType::EquityCommon,
              vec![],
              None,
              true // Disable Investor uniqueness requirements.
            ).submit()?;
            // Pause compliance rules to allow transfers.
            api.call().compliance_manager().pause_asset_compliance(self.ticker).submit()?;
            Ok(())
        }

        fn get_did(&self, acc: AccountId) -> Result<IdentityId> {
            Self::env().extension().get_key_did(acc)?
              .map(|did| did.into())
              .ok_or(Error::MissingIdentity)
        }

        fn get_caller_did(&self) -> Result<IdentityId> {
          self.get_did(Self::env().caller())
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

        fn init_asset_and_venue(&mut self) -> Result<()> {
            if self.initialized {
              return Err(Error::AlreadyInitialized);
            }
            // Create tickers.
            self.create_wrapped_polyx()?;

            let api = Api::new();
            // Get the next venue id.
            let id = api.query().settlement().venue_counter()
              .map(|v| v.into())?;
            // Create Venue.
            api.call().settlement().create_venue(
              VenueDetails(b"Contract Venue".to_vec()),
              vec![],
              VenueType::Other
            ).submit()?;
            // Save venue id.
            self.venue = id;
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
            // Get the caller's identity.
            let caller_did = self.get_caller_did()?;
            // Ensure the caller doesn't have a portfolio.
            self.ensure_no_portfolio(caller_did)?;

            let portfolio_id = PortfolioId {
              did: caller_did,
              kind: portfolio,
            };
            let api = Api::new();
            // Accept authorization.
            api.call().portfolio().accept_portfolio_custody(auth_id).submit()?;
            // Check that we are the custodian.
            if !api.query().portfolio().portfolios_in_custody(self.did, portfolio_id)? {
              return Err(Error::InvalidPortfolioAuthorization);
            }
            // Save the caller's portfolio.
            self.portfolios.insert(caller_did, &portfolio_id);

            Self::env().emit_event(PortfolioAdded {
              did: caller_did,
              portfolio_kind: portfolio,
            });

            Ok(())
        }

        #[ink(message, payable)]
        /// Allow the caller to withdrawal funds from the contract controlled portfolio.
        pub fn mint_wrapped_polyx(&mut self) -> Result<()> {

            self.ensure_initialized()?;
            let amount = Self::env().transferred_value();

            // Get the caller's identity.
            let caller_did = self.get_caller_did()?;

            // Ensure the caller has a portfolio.
            let caller_portfolio = self.ensure_has_portfolio(caller_did)?;

            let api = Api::new();
            // Mint some tokens.
            api.call()
                .asset()
                .issue(self.ticker, amount)
                .submit()?;
            // Get the next instruction id.
            let instruction_id = api
                .query()
                .settlement()
                .instruction_counter()
                .map(|v| v.into())?;

            // Transfer tokens to the caller's portfolio.
            let our_portfolio = PortfolioId {
              did: self.did,
              kind: PortfolioKind::Default,
            };
            api.call().settlement().add_and_affirm_instruction(
              self.venue,
              SettlementType::SettleManual(0),
              None,
              None,
              vec![Leg {
                from: our_portfolio,
                to: caller_portfolio,
                asset: self.ticker,
                amount: amount,
              }],
              vec![
                our_portfolio,
                caller_portfolio,
              ],
            ).submit()?;

            api.call().settlement().execute_manual_instruction(
                instruction_id,
                1,
                None
            ).submit()?;

            Self::env().emit_event(PolyxWrapped {
              did: caller_did,
              key: Self::env().caller().into(),
              amount: amount,
            });
            Ok(())
        }

        #[ink(message)]
        /// Allow the caller to withdrawal funds from the contract controlled portfolio.
        pub fn burn_wrapped_polyx(&mut self, amount: Balance) -> Result<()> {
            self.ensure_initialized()?;

            // Get the caller's identity.
            let caller_did = self.get_caller_did()?;

            // Ensure the caller has a portfolio.
            let caller_portfolio = self.ensure_has_portfolio(caller_did)?;

            let api = Api::new();
            // Get the next instruction id.
            let instruction_id = api
                .query()
                .settlement()
                .instruction_counter()
                .map(|v| v.into())?;

            // Transfer tokens to the caller's portfolio.
            let our_portfolio = PortfolioId {
              did: self.did,
              kind: PortfolioKind::Default,
            };
            api.call().settlement().add_and_affirm_instruction(
              self.venue,
              SettlementType::SettleManual(0),
              None,
              None,
              vec![Leg {
                from: caller_portfolio,
                to: our_portfolio,
                asset: self.ticker,
                amount: amount,
              }],
              vec![
                our_portfolio,
                caller_portfolio,
              ],
            ).submit()?;

            api.call().settlement().execute_manual_instruction(
                instruction_id,
                1,
                None
            ).submit()?;

            api.call().asset().redeem(
              self.ticker,
              amount,
            ).submit()?;

            if Self::env().transfer(Self::env().caller().into(), amount).is_err() {
              panic!("error transferring")
            }

            Self::env().emit_event(PolyxUnwrapped {
              did: caller_did,
              key: Self::env().caller().into(),
              amount: amount,
            });

            Ok(())
        }

        #[ink(message)]
        /// Return the caller's portfolio custodianship back to them.
        pub fn remove_portfolio(&mut self) -> Result<()> {
            self.ensure_initialized()?;

            // Get the caller's identity.
            let caller_did = self.get_caller_did()?;

            // Ensure the caller has a portfolio.
            let portfolio = self.ensure_has_portfolio(caller_did)?;

            let api = Api::new();
            // Remove our custodianship.
            api.call().portfolio().quit_portfolio_custody(portfolio).submit()?;
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
        pub fn withdraw_polyx(
            &mut self,
            amount: Balance,
            dest: PortfolioKind,
        ) -> Result<()> {
            self.ensure_initialized()?;

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
                    vec![MovePortfolioItem {
                        ticker: self.ticker,
                        amount,
                        memo: None,
                    }],
                )
                .submit()?;
            Ok(())
        }
    }
}
