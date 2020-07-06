use frame_support::{decl_error, decl_event, decl_module, decl_storage};
use frame_system::{self as system, ensure_signed};
use polymesh_common_utilities::balances;
use polymesh_primitives::{PortfolioId, PortfolioName, PortfolioNumber};
use sp_std::prelude::Vec;

pub trait Trait: balances::Trait {}

decl_storage! {
    trait Store for Module<T: Trait> as Session {
        /// The set of existing portfolios with their names. If a certain pair of a DID and
        /// portfolio number maps to `None` then such a portfolio doesn't exist. Conversely, if a
        /// pair maps to `Some(name)` then such a portfolio exists and is called `name`.
        pub Portfolios get(portfolios):
            double_map hasher(blake2_128_concat) IdentityId, hasher(twox_128_concat) PortfolioNumber =>
            Option<PortfolioName>;
        /// Asset balances of portfolios.
        pub PortfolioAssetBalances get(portfolio_asset_balances):
            double_map hasher(blake2_128_concat) PortfolioId, hasher(blake2_128_concat) Ticker =>
            T::Balance;
        /// The next portfolio sequence number.
        pub NextPortfolioNumber get(next_portfolio_number) build(|_| 1): u128;
    }
}

decl_event! {
    pub enum Event<T> {
        /// The portfolio has been successfully created.
        PortfolioCreated(IdentityId, PortfolioNumber, PortfolioName),
        /// The portfolio has been successfully removed.
        PortfolioDeleted(IdentityId, PortfolioNumber),
        /// A token amount has been moved from one portfoliio to another. `None` denotes the default
        /// portfolio of the DID.
        MovedBetweenPortfolios(
            IdentityId,
            Option<PortfolioNumber>,
            Option<PortfolioNumber>,
            Ticker,
            T::Balance
        ),
        /// The portfolio identified with `num` has been renamed to `name`.
        PortfolioRenamed(IdentityId, PortfolioNumber, PortfolioName),
        /// All non-default portfolio numbers and names of a DID.
        UserPortfolios(IdentityId, Vec<(PortfolioNumber, PortfolioName)>),
    }
}

decl_error! {
    /// The portfolio doesn't exist.
    PortfolioDoesNotExist,
    /// Insufficient balance for a transaction.
    InsufficientBalance,
    /// The source and destination portfolios should be different.
    DestinationIsSamePortfolio,
    /// The portfolio couldn't be renamed because the chosen name is already in use.
    PortfolioNameAlreadyInUse,
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
            let name_uniq = <Portfolios<T>>::iter_prefix(&did).all(|(_, n)| n != &name);
            ensure!(name_uniq, Error::<T>::PortfolioNameAlreadyInUse);
            let num = Self::get_next_portfolio_number();
            <Portfolios<T>>::insert(&did, &num, Some(name.clone()));
            let portfolio_id = PortfolioId::User();
            Self::deposit_event(RawEvent::PortfolioCreated(did, num, name));
            Ok(())
        }

        /// Deletes a user portfolio and moves all its assets to the default portfolio.
        pub fn delete_portfolio(origin, num: PortfolioNumber) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            ensure!(Self::portfolios(&did, &num), Error::<T>::PortfolioDoesNotExist);
            let portfolio_id = PortfolioId {
                did,
                num,
            };
            let def_portfolio_id = PortfolioId::default_portfolio(did);
            for (ticker, balance) in <PortfolioAssetBalances<T>>::iter_prefix(&portfolio_id) {
                <PortfolioAssetBalances<T>>::mutate(&def_portfolio_id, ticker, |v| {
                    *v = *v.saturating_add(balance)
                });
                Self::deposit_event(RawEvent::MovedBetweenPortfolios(
                    did,
                    Some(num),
                    None,
                    *ticker,
                    *balance,
                ));
            }
            <PortfolioAssetBalances<T>>::remove_prefix(&portfolio_id);
            <Portfolios<T>>::remove(&did, &num);
            Self::deposit_event(RawEvent::PortfolioDeleted(did, num));
            Ok(())
        }

        /// Moves a token amount from one portfolio of an identity to another portfolio of the same
        /// identity.
        pub fn move_portfolio(
            origin,
            from_num: Option<PortfolioNumber>,
            to_num: Option<PortfolioNumber>,
            ticker: Ticker,
            amount: T::Balance
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            ensure!(from_num != to_num, Error::<T>::DestinationIsSamePortfolio);
            let did_name = &(did, name.clone());
            ensure!(Self::portfolios(&did, &from_num), Error::<T>::PortfolioDoesNotExist);
            ensure!(Self::portfolios(&did, &to_num), Error::<T>::PortfolioDoesNotExist);
            let from_portfolio_id = from_num
                .and_then(|num| PortfolioId::User(did, num))
                .unwrap_or_else(|| Portfolio::Default(did));
            let to_portfolio_id = to_num
                .and_then(|num| PortfolioId::User(did, num))
                .unwrap_or_else(|| Portfolio::Default(did));
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
                from_num,
                to_num,
                ticker,
                amount
            ));
            Ok(())
        }

        /// Renames a non-default portfolio.
        pub fn rename_portfolio(
            origin,
            num: PortfolioNumber,
            to_name: PortfolioName,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            ensure!(Self::portfolios(&did, &num), Error::<T>::PortfolioDoesNotExist);
            let name_uniq = <Portfolios<T>>::iter_prefix(&did).all(|(_, name)| name != to_name);
            ensure!(name_uniq, Error::<T>::PortfolioNameAlreadyInUse);
            <Portfolios<T>>::mutate(&did, &num, |name| *name = Some(to_name));
            deposit_event(RawEvent::PortfolioRenamed(
                did,
                num,
                to_name,
            ));
            Ok(())
        }

        /// Emits an event containing all non-default portfolio numbers and names of a given DID.
        pub fn get_portfolios(origin) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            let portfolios = <Portfolios<T>>::iter_prefix(&did).collect();
            deposit_event(RawEvent::UserPortfolios(
                did,
                portfolios,
            ));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    /// Returns the next portfolio number and increments the stored number.
    fn get_next_portfolio_number() -> PortfolioNumber {
        let num = Self::next_portfolio_number();
        <NextPortfolioNumber>::insert(num + 1);
        num
    }
}
