use crate::asset::{self, AssetTrait};
use crate::constants::*;
use crate::identity;
use crate::utils;
use codec::Encode;
use core::result::Result as StdResult;
use identity::ClaimValue;
use primitives::Key;
use rstd::{convert::TryFrom, prelude::*};
use srml_support::{decl_event, decl_module, decl_storage, dispatch::Result, ensure};
use system::{self, ensure_signed};

#[derive(codec::Encode, codec::Decode, Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum Operators {
    EqualTo,
    NotEqualTo,
    LessThan,
    GreaterThan,
    LessOrEqualTo,
    GreaterOrEqualTo,
}

impl Default for Operators {
    fn default() -> Self {
        Operators::EqualTo
    }
}

/// The module's configuration trait.
pub trait Trait: timestamp::Trait + system::Trait + utils::Trait + identity::Trait {
    // TODO: Add other types and constants required configure this module.

    /// The overarching event type.
    type Event: From<Event> + Into<<Self as system::Trait>::Event>;
    type Asset: asset::AssetTrait<Self::TokenBalance>;
}

#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct AssetRule {
    sender_rules: Vec<RuleData>,
    receiver_rules: Vec<RuleData>,
}

#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct RuleData {
    key: Vec<u8>,
    value: Vec<u8>,
    trusted_issuers: Vec<Vec<u8>>,
    operator: Operators,
}

decl_storage! {
    trait Store for Module<T: Trait> as GeneralTM {
        // (Asset -> AssetRules)
        pub ActiveRules get(active_rules): map Vec<u8> => Vec<AssetRule>;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        fn add_asset_rule(origin, did: Vec<u8>, _ticker: Vec<u8>, asset_rule: AssetRule) -> Result {
            let ticker = utils::bytes_to_upper(_ticker.as_slice());
            let sender = ensure_signed(origin)?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signing_key(&did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");

            ensure!(Self::is_owner(ticker.clone(), did.clone()), "user is not authorized");

            <ActiveRules>::mutate(ticker.clone(), |old_asset_rules| {
                if !old_asset_rules.contains(&asset_rule) {
                    old_asset_rules.push(asset_rule.clone());
                }
            });

            Self::deposit_event(Event::NewAssetRule(ticker, asset_rule));

            Ok(())
        }

        fn remove_asset_rule(origin, did: Vec<u8>, _ticker: Vec<u8>, asset_rule: AssetRule) -> Result {
            let ticker = utils::bytes_to_upper(_ticker.as_slice());
            let sender = ensure_signed(origin)?;

            ensure!(<identity::Module<T>>::is_signing_key(&did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");

            ensure!(Self::is_owner(ticker.clone(), did.clone()), "user is not authorized");

            <ActiveRules>::mutate(ticker.clone(), |old_asset_rules| {
                *old_asset_rules = old_asset_rules
                    .iter()
                    .cloned()
                    .filter(|an_asset_rule| *an_asset_rule != asset_rule)
                    .collect();
            });

            Self::deposit_event(Event::RemoveAssetRule(ticker, asset_rule));

            Ok(())
        }
    }
}

decl_event!(
    pub enum Event {
        NewAssetRule(Vec<u8>, AssetRule),
        RemoveAssetRule(Vec<u8>, AssetRule),
    }
);

impl<T: Trait> Module<T> {
    pub fn is_owner(_ticker: Vec<u8>, sender_did: Vec<u8>) -> bool {
        let ticker = utils::bytes_to_upper(_ticker.as_slice());
        T::Asset::is_owner(&ticker, &sender_did)
    }

    pub fn fetch_value(
        did: Vec<u8>,
        key: Vec<u8>,
        trusted_issuers: Vec<Vec<u8>>,
    ) -> Option<ClaimValue> {
        <identity::Module<T>>::fetch_claim_value_multiple_issuers(did, key, trusted_issuers)
    }

    ///  Sender restriction verification
    pub fn verify_restriction(
        ticker: &Vec<u8>,
        from_did: &Vec<u8>,
        to_did: &Vec<u8>,
        _value: T::TokenBalance,
    ) -> StdResult<u8, &'static str> {
        // Transfer is valid if All reciever and sender rules of any asset rule are valid.
        let ticker = utils::bytes_to_upper(ticker.as_slice());
        let active_rules = Self::active_rules(ticker.clone());
        for active_rule in active_rules {
            let mut rule_broken = false;
            for sender_rule in active_rule.sender_rules {
                let identity_value = Self::fetch_value(
                    from_did.clone(),
                    sender_rule.key,
                    sender_rule.trusted_issuers,
                );
                rule_broken = match identity_value {
                    None => true,
                    Some(x) => utils::check_rule(
                        sender_rule.value,
                        x.value,
                        x.data_type,
                        sender_rule.operator,
                    ),
                };
                if rule_broken {
                    break;
                }
            }
            if rule_broken {
                continue;
            }
            for receiver_rule in active_rule.receiver_rules {
                let identity_value = Self::fetch_value(
                    from_did.clone(),
                    receiver_rule.key,
                    receiver_rule.trusted_issuers,
                );
                rule_broken = match identity_value {
                    None => true,
                    Some(x) => utils::check_rule(
                        receiver_rule.value,
                        x.value,
                        x.data_type,
                        receiver_rule.operator,
                    ),
                };
                if rule_broken {
                    break;
                }
            }
            if !rule_broken {
                return Ok(ERC1400_TRANSFER_SUCCESS);
            }
        }

        sr_primitives::print("Identity TM restrictions not satisfied");
        Ok(ERC1400_TRANSFER_FAILURE)
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use chrono::{prelude::*, Duration};
    use lazy_static::lazy_static;
    use sr_io::with_externalities;
    use sr_primitives::{
        testing::{Header, UintAuthorityId},
        traits::{BlakeTwo256, ConvertInto, IdentityLookup, OpaqueKeys},
        Perbill,
    };
    use srml_support::traits::Currency;
    use srml_support::{assert_ok, impl_outer_origin, parameter_types};
    use substrate_primitives::{Blake2Hasher, H256};

    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use crate::{asset::SecurityToken, balances, exemption, identity, percentage_tm, registry};

    type SessionIndex = u32;
    type AuthorityId = u64;
    type BlockNumber = u64;

    pub struct TestOnSessionEnding;
    impl session::OnSessionEnding<AuthorityId> for TestOnSessionEnding {
        fn on_session_ending(_: SessionIndex, _: SessionIndex) -> Option<Vec<AuthorityId>> {
            None
        }
    }

    pub struct TestSessionHandler;
    impl session::SessionHandler<AuthorityId> for TestSessionHandler {
        fn on_new_session<Ks: OpaqueKeys>(
            _changed: bool,
            _validators: &[(AuthorityId, Ks)],
            _queued_validators: &[(AuthorityId, Ks)],
        ) {
        }

        fn on_disabled(_validator_index: usize) {}

        fn on_genesis_session<Ks: OpaqueKeys>(_validators: &[(AuthorityId, Ks)]) {}
    }

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    parameter_types! {
        pub const BlockHashCount: u32 = 250;
        pub const MaximumBlockWeight: u32 = 4 * 1024 * 1024;
        pub const MaximumBlockLength: u32 = 4 * 1024 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    }
    impl system::Trait for Test {
        type Origin = Origin;
        type Call = ();
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<u64>;
        type WeightMultiplierUpdate = ();
        type Header = Header;
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type AvailableBlockRatio = AvailableBlockRatio;
        type MaximumBlockLength = MaximumBlockLength;
        type Version = ();
    }

    parameter_types! {
        pub const Period: BlockNumber = 1;
        pub const Offset: BlockNumber = 0;
        pub const ExistentialDeposit: u64 = 0;
        pub const TransferFee: u64 = 0;
        pub const CreationFee: u64 = 0;
        pub const TransactionBaseFee: u64 = 0;
        pub const TransactionByteFee: u64 = 0;
    }

    parameter_types! {
        pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(33);
    }

    impl session::Trait for Test {
        type OnSessionEnding = TestOnSessionEnding;
        type Keys = UintAuthorityId;
        type ShouldEndSession = session::PeriodicSessions<Period, Offset>;
        type SessionHandler = TestSessionHandler;
        type Event = ();
        type ValidatorId = AuthorityId;
        type ValidatorIdOf = ConvertInto;
        type SelectInitialValidators = ();
        type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
    }

    impl session::historical::Trait for Test {
        type FullIdentification = ();
        type FullIdentificationOf = ();
    }

    impl balances::Trait for Test {
        type Balance = u128;
        type OnFreeBalanceZero = ();
        type OnNewAccount = ();
        type Event = ();
        type TransactionPayment = ();
        type DustRemoval = ();
        type TransferPayment = ();
        type ExistentialDeposit = ExistentialDeposit;
        type TransferFee = TransferFee;
        type CreationFee = CreationFee;
        type TransactionBaseFee = TransactionBaseFee;
        type TransactionByteFee = TransactionByteFee;
        type WeightToFee = ConvertInto;
        type Identity = identity::Module<Test>;
    }

    impl asset::Trait for Test {
        type Event = ();
        type Currency = balances::Module<Test>;
    }

    impl identity::Trait for Test {
        type Event = ();
    }

    impl exemption::Trait for Test {
        type Event = ();
        type Asset = Module<Test>;
    }

    impl percentage_tm::Trait for Test {
        type Event = ();
    }

    parameter_types! {
        pub const MinimumPeriod: u64 = 3;
    }

    impl timestamp::Trait for Test {
        type Moment = u64;
        type OnTimestampSet = ();
        type MinimumPeriod = MinimumPeriod;
    }

    impl utils::Trait for Test {
        type TokenBalance = u128;
        fn as_u128(v: Self::TokenBalance) -> u128 {
            v
        }
        fn as_tb(v: u128) -> Self::TokenBalance {
            v
        }
        fn token_balance_to_balance(v: Self::TokenBalance) -> <Self as balances::Trait>::Balance {
            v
        }
        fn balance_to_token_balance(v: <Self as balances::Trait>::Balance) -> Self::TokenBalance {
            v
        }
        fn validator_id_to_account_id(v: <Self as session::Trait>::ValidatorId) -> Self::AccountId {
            v
        }
    }

    impl registry::Trait for Test {}

    impl Trait for Test {
        type Event = ();
    }
    impl asset::AssetTrait<<Test as utils::Trait>::TokenBalance> for Module<Test> {
        fn is_owner(ticker: &Vec<u8>, sender_did: &Vec<u8>) -> bool {
            if let Some(token) = TOKEN_MAP.lock().unwrap().get(ticker) {
                token.owner_did == *sender_did
            } else {
                false
            }
        }

        fn _mint_from_sto(
            _ticker: &[u8],
            _sender_did: &Vec<u8>,
            _tokens_purchased: <Test as utils::Trait>::TokenBalance,
        ) -> Result {
            unimplemented!();
        }

        /// Get the asset `id` balance of `who`.
        fn balance(_ticker: &[u8], _did: Vec<u8>) -> <Test as utils::Trait>::TokenBalance {
            unimplemented!();
        }

        // Get the total supply of an asset `id`
        fn total_supply(_ticker: &[u8]) -> <Test as utils::Trait>::TokenBalance {
            unimplemented!();
        }
    }

    fn _check_investor_status(_holder_did: &Vec<u8>) -> Result {
        // TODO check with claim.
        /*let investor = <identity::DidRecords<T>>::get(holder_did);
        ensure!(
            investor.has_signing_keys_role(IdentityRole::Investor),
            "Account is not an investor"
        );*/
        Ok(())
    }

    type DividendModule = Module<Test>;
    type Balances = balances::Module<Test>;
    type Asset = asset::Module<Test>;
    type Identity = identity::Module<Test>;

    /// Build a genesis identity instance owned by the specified account
    fn identity_owned_by(id: u64) -> sr_io::TestExternalities<Blake2Hasher> {
        let mut t = system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        identity::GenesisConfig::<Test> {
            owner: id,
            did_creation_fee: 250,
        }
        .assimilate_storage(&mut t)
        .unwrap();
        sr_io::TestExternalities::new(t)
    }

    #[test]
    fn verify_transfer_should_work() {
        let identity_owner_id = 1;
        with_externalities(&mut identity_owned_by(identity_owner_id), || {
            let token_owner_acc = 1;
            let token_owner_did = "did:poly:1".as_bytes().to_vec();

            // A token representing 1M shares
            let token = SecurityToken {
                name: vec![0x01],
                owner_did: token_owner_did.clone(),
                total_supply: 1_000_000,
                granularity: 1,
                decimals: 18,
            };

            Balances::make_free_balance_be(&token_owner_acc, 1_000_000);
            Identity::register_did(
                Origin::signed(token_owner_acc),
                token_owner_did.clone(),
                vec![],
            )
            .expect("Could not create token_owner_did");

            identity::Module::<Test>::do_create_issuer(token.owner_did.clone())
                .expect("Could not make token.owner_did an issuer");

            // Share issuance is successful
            assert_ok!(Asset::create_token(
                Origin::signed(token_owner_acc),
                token_owner_did.clone(),
                token.name.clone(),
                token.name.clone(),
                token.total_supply,
                true
            ));
        });
    }
}
