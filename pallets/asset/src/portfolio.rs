use frame_support::{decl_error, decl_event, decl_module, decl_storage};
use frame_system::{self as system, ensure_signed};
use polymesh_common_utilities::balances;
use polymesh_primitives::{PortfolioId, PortfolioName};
use sp_std::prelude::Vec;

pub trait Trait: balances::Trait {}

decl_storage! {
    trait Store for Module<T: Trait> as Session {
        /// The map of identities' portfolios.
        pub Portfolios get(portfolios): double_map hasher(blake2_128_concat) IdentityId,
            hasher(blake2_128_concat) PortfolioName => Vec<(Ticker, T::Balance)>
    }
}

decl_event! {
    pub enum Event<T> {
        /// The portfolio has been successfully created.
        PortfolioCreated(PortfolioId),
        /// A token amount has been moved from one portfoliio to another.
        MovedBetweenPortfolios(IdentityId, PortfolioName, PortfolioName, Ticker, T::Balance),
    }
}

decl_error! {
    /// The portfolio couldn't be created because it already exists.
    PortfolioAlreadyExists,
    /// The portfolio doesn't exist or has an insufficient amount of tokens.
    PortfolioNotFound,
    /// Insufficient balance for a transaction.
    InsufficientBalance,
    /// The ticker has zero balance in a given portfolio.
    TickerNotFound,
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
            let did_name = &(did, name.clone());
            ensure!(!<Portfolios>::contains_key(did_name), Error::<T>::PortfolioNotFound);
            <Portfolios<T>>::insert(did, name.clone(), vec![]);
            Self::deposit_event(RawEvent::PortfolioCreated(PortfolioId {
                did,
                name
            }));
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
            let did_name = &(did, name.clone());
            ensure!(!<Portfolios<T>>::contains_key(did_name), Error::<T>::PortfolioNotFound);
            let portfolio = Self::portfolios(&did, &name);
            let balance = portfolio.iter().find_map(|&&entry| {
                if entry.0 == ticker {
                    Some(entry.1)
                } else {
                    None
                }
            }).unwrap_or_default();
            ensure!(balance >= amount, Error::<T>::InsufficientBalance);
            // TODO
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
