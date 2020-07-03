use frame_support::{decl_error, decl_event, decl_module, decl_storage};
use frame_system::{self as system, ensure_signed};
use polymesh_common_utilities::balances;
use polymesh_primitives::{PortfolioId, PortfolioName};
use sp_std::prelude::Vec;

pub trait Trait: balances::Trait {}

decl_storage! {
    trait Store for Module<T: Trait> as Session {
        /// The set of existing portfolios.
        pub Portfolios get(portfolios):
            double_map hasher(blake2_128_concat) IdentityId, hasher(blake2_128_concat) PortfolioName =>
            bool;
        /// Asset balances of portfolios.
        pub PortfolioAssetBalances get(portfolio_asset_balances):
            double_map hasher(blake2_128_concat) PortfolioId, hasher(blake2_128_concat) Ticker =>
            T::Balance;
    }
}

decl_event! {
    pub enum Event<T> {
        /// The portfolio has been successfully created.
        PortfolioCreated(PortfolioId),
        /// The portfolio has been successfully removed.
        PortfolioCreated(PortfolioId),
        /// A token amount has been moved from one portfoliio to another.
        MovedBetweenPortfolios(IdentityId, PortfolioName, PortfolioName, Ticker, T::Balance),
    }
}

decl_error! {
    /// The portfolio couldn't be created because it already exists.
    PortfolioAlreadyExists,
    /// The portfolio doesn't exist.
    PortfolioDoesNotExist,
    /// Insufficient balance for a transaction.
    InsufficientBalance,
    /// The ticker has zero balance in a given portfolio.
    TickerNotFound,
    /// The source and destination portfolios should be different.
    CannotMoveIntoSamePortfolio,
    /// The default portfolio cannot be removed.
    CannotRemoveDefaultPortfolio,
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        /// The event logger.
        fn deposit_event() = default;

        /// Creates a portfolio.
        pub fn create_portfolio(origin, name: PortfolioName) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            ensure!(!Self::portfolios(&did, &name), Error::<T>::PortfolioAlreadyExists);
            <Portfolios<T>>::insert(&did, &name, true);
            let portfolio_id = PortfolioId {
                did,
                name,
            };
            Self::deposit_event(RawEvent::PortfolioCreated(portfolio_id));
            Ok(())
        }

        /// Removes a portfolio other than the default portfolio and moves all its assets to the
        /// default portfolio.
        pub fn remove_portfolio(origin, name: PortfolioName) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            ensure!(Self::portfolios(&did, &name), Error::<T>::PortfolioDoesNotExist);
            ensure!(name != PortfolioName::default(), Error::<T>::CannotRemoveDefaultPortfolio);
            let portfolio_id = PortfolioId {
                did,
                name.clone(),
            };
            let def_portfolio_id = PortfolioId::default_portfolio(did);
            for (ticker, balance) in <PortfolioAssetBalances<T>>w::iter_prefix(&portfolio_id)w {
                <PortfolioAssetBalances<T>>::mutate(&def_portfolio_id, ticker, |v| v = v + balance);
                Self::deposit_event(RawEvent::MovedBetweenPortfolios(
                    did,
                    name.clone(),
                    PortfolioName::default(),
                    ticker,
                    balance,
                ));
            }
            <PortfolioAssetBalances<T>>::remove_prefix(&portfolio_id);
            <Portfolios<T>>::remove(&did, &name);
            Self::deposit_event(RawEvent::PortfolioRemoved(portfolio_id));
            Ok(())
        }

        /// Moves a token amount from one portfolio of an identity to another portfolio of the same
        /// identity.
        pub fn move_portfolio(
            origin,
            from_name: PortfolioName,
            to_name: PortfolioName,
            ticker: Ticker,
            amount: T::Balance
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            ensure!(from_name != to_name, Error::<T>::CannotMoveIntoSamePortfolio);
            let did_name = &(did, name.clone());
            ensure!(Self::portfolios(&did, &from_name), Error::<T>::PortfolioDoesNotExist);
            ensure!(Self::portfolios(&did, &to_name), Error::<T>::PortfolioDoesNotExist);
            let from_portfolio_id = PortfolioId {
                did,
                from_name.clone(),
            };
            let to_portfolio_id = PortfolioId {
                did,
                to_name.clone(),
            };
            let balance = Self::portfolio_asset_balances(&from_portfolio_id, &ticker);
            ensure!(balance >= amount, Error::<T>::InsufficientBalance);
            <PortfolioAssetBalances<T>>::insert(&from_portfolio_id, &ticker, balance - amount);
            <PortfolioAssetBalances<T>>::insert(
                &to_portfolio_id,
                &ticker,
                balance.saturating_add(amount)
            );
            deposit_event(RawEvent::MovedBetweenPortfolios(
                did,
                from_name,
                to_name,
                ticker,
                amount
            ));
            Ok(())
        }
    }
}
