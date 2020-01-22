use crate::balances;

use rstd::vec::Vec;
use srml_support::{decl_module, decl_storage};

type AssetId = Vec<u8>;
type Counter = u64;

pub trait Trait: balances::Trait {}

decl_storage! {
    trait Store for Module<T: Trait> as statistics {
        pub InvestorCountPerAsset get(investor_count_per_asset): map AssetId => Counter ;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    }
}

pub trait StatisticTrait<T: Trait> {
    fn update_transfer_stats(
        ticker: &Vec<u8>,
        updated_from_balance: T::Balance,
        updated_to_balance: T::Balance,
        amount: T::Balance,
    );
}

impl<T: Trait> StatisticTrait<T> for Module<T> {
    fn update_transfer_stats(
        ticker: &Vec<u8>,
        updated_from_balance: T::Balance,
        updated_to_balance: T::Balance,
        amount: T::Balance,
    ) {
        if amount != 0u128.into() {
            let counter = Self::investor_count_per_asset(ticker);
            let mut new_counter = counter;

            if updated_from_balance == 0u128.into() {
                new_counter = new_counter.checked_sub(1).unwrap_or(new_counter);
            }
            if updated_to_balance == amount {
                new_counter = new_counter.checked_add(1).unwrap_or(new_counter);
            }

            if new_counter != counter {
                <InvestorCountPerAsset>::insert(ticker, new_counter)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        asset::{self, SecurityToken},
        statistics,
        test::storage::{build_ext, register_keyring_account, TestStorage},
    };

    use sr_io::with_externalities;
    use srml_support::assert_ok;
    use test_client::AccountKeyring;

    type Origin = <TestStorage as system::Trait>::Origin;
    type Asset = asset::Module<TestStorage>;
    type Statistic = statistics::Module<TestStorage>;

    #[test]
    fn investor_count_per_asset() {
        with_externalities(&mut build_ext(), investor_count_per_asset_with_ext);
    }

    fn investor_count_per_asset_with_ext() {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let alice_signed = Origin::signed(AccountKeyring::Alice.public());
        let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
        let bob_signed = Origin::signed(AccountKeyring::Bob.public());
        let charlie_did = register_keyring_account(AccountKeyring::Charlie).unwrap();

        // 1. Alice create an asset.
        let token = SecurityToken {
            name: vec![0x01],
            owner_did: alice_did,
            total_supply: 1_000_000,
            divisible: true,
        };

        assert_ok!(Asset::create_token(
            alice_signed.clone(),
            alice_did,
            token.name.clone(),
            token.name.clone(),
            1_000_000, // Total supply over the limit
            true
        ));

        // Alice sends some tokens to Bob. Token has only one investor.
        assert_ok!(Asset::transfer(
            alice_signed.clone(),
            alice_did,
            token.name.clone(),
            bob_did,
            500
        ));
        assert_eq!(Statistic::investor_count_per_asset(&token.name), 1);

        // Alice sends some tokens to Charlie. Token has now two investors.
        assert_ok!(Asset::transfer(
            alice_signed,
            alice_did,
            token.name.clone(),
            charlie_did,
            5000
        ));
        assert_eq!(Statistic::investor_count_per_asset(&token.name), 2);

        // Bob sends all his tokens to Charlie, so now we have one investor again.
        assert_ok!(Asset::transfer(
            bob_signed,
            bob_did,
            token.name.clone(),
            charlie_did,
            500
        ));
        assert_eq!(Statistic::investor_count_per_asset(&token.name), 1);
    }
}
