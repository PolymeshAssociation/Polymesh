//! # General Transfer Manager Module
//!
//! The GTM module provides functionality for setting whitelisting rules for transfers
//!
//! ## Overview
//!
//! The GTM module provides functions for:
//!
//! - Adding rules for allowing transfers
//! - Removing rules that allow transfers
//! - Resetting all rules
//!
//! ### Use case
//!
//! This module is very versatile and offers infinite possibilities.
//! The rules can dictate various requirements like:
//!
//! - Only accredited investors should be able to trade
//! - Only valid KYC holders should be able to trade
//! - Only those with credit score of greater than 800 should be able to purchase this token
//! - People from Wakanda should only be able to trade with people from Wakanda
//! - People from Gryffindor should not be able to trade with people from Slytherin (But allowed to trade with anyone else)
//! - Only marvel supporters should be allowed to buy avengers token
//!
//! ### Terminology
//!
//! - **Active rules:** It is an array of Asset rules that are currently enforced for a ticker
//! - **Asset rule:** Every asset rule contains an array for sender rules and an array for receiver rules
//! - **sender rules:** These are rules that the sender of security tokens must follow
//! - **receiver rules:** These are rules that the receiver of security tokens must follow
//! - **Valid transfer:** For a transfer to be valid,
//!     All reciever and sender rules of any of the active asset rule must be followed.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `add_active_rule` - Adds a new asset rule to ticker's active rules
//! - `remove_active_rule` - Removes an asset rule from ticker's active rules
//! - `reset_active_rules` - Reset(remove) all active rules of a tikcer
//!
//! ### Public Functions
//!
//! - `verify_restriction` - Checks if a transfer is a valid transfer and returns the result

use crate::{
    asset::{self, AssetTrait},
    balances,
    constants::*,
    identity, utils,
};
use codec::Encode;
use core::result::Result as StdResult;
use frame_support::{decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure};
use frame_system::{self as system, ensure_signed};
use identity::ClaimValue;
use primitives::{AccountKey, IdentityId, Signer, Ticker};
use sp_std::{convert::TryFrom, prelude::*};

/// Type of operators that a rule can have
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
pub trait Trait:
    pallet_timestamp::Trait + frame_system::Trait + balances::Trait + utils::Trait + identity::Trait
{
    /// The overarching event type.
    type Event: From<Event> + Into<<Self as frame_system::Trait>::Event>;

    /// Asset module
    type Asset: asset::AssetTrait<Self::Balance>;
}

/// An asset rule.
/// All sender and receiver rules of the same asset rule must be true for tranfer to be valid
#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct AssetRule {
    pub sender_rules: Vec<RuleData>,
    pub receiver_rules: Vec<RuleData>,
}

/// Details about individual rules
#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct RuleData {
    /// Claim key
    key: Vec<u8>,

    /// Claim target value. (RHS of operatior)
    value: Vec<u8>,

    /// Array of trusted claim issuers
    trusted_issuers: Vec<IdentityId>,

    /// Operator. The rule is "Actual claim value" Operator "Rule value defined in this struct"
    /// Example: If the actual claim value is 5, value defined here is 10 and operator is NotEqualTo
    /// Then the rule will be resolved as 5 != 10 which is true and hence the rule will pass
    operator: Operators,
}

decl_storage! {
    trait Store for Module<T: Trait> as GeneralTM {
        /// List of active rules for a ticker (Ticker -> Array of AssetRules)
        pub ActiveRules get(fn active_rules): map Ticker => Vec<AssetRule>;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Adds an asset rule to active rules for a ticker
        pub fn add_active_rule(origin, did: IdentityId, ticker: Ticker, asset_rule: AssetRule) -> DispatchResult {
            let sender = Signer::AccountKey(AccountKey::try_from(ensure_signed(origin)?.encode())?);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signer_authorized(did, &sender), "sender must be a signing key for DID");
            ticker.canonize();
            ensure!(Self::is_owner(&ticker, did), "user is not authorized");

            <ActiveRules>::mutate(ticker, |old_asset_rules| {
                if !old_asset_rules.contains(&asset_rule) {
                    old_asset_rules.push(asset_rule.clone());
                }
            });

            Self::deposit_event(Event::NewAssetRule(ticker, asset_rule));

            Ok(())
        }

        /// Removes a rule from active asset rules
        pub fn remove_active_rule(origin, did: IdentityId, ticker: Ticker, asset_rule: AssetRule) -> DispatchResult {
            let sender = Signer::AccountKey(AccountKey::try_from( ensure_signed(origin)?.encode())?);

            ensure!(<identity::Module<T>>::is_signer_authorized(did, &sender), "sender must be a signing key for DID");
            ticker.canonize();
            ensure!(Self::is_owner(&ticker, did), "user is not authorized");

            <ActiveRules>::mutate(ticker, |old_asset_rules| {
                *old_asset_rules = old_asset_rules
                    .iter()
                    .cloned()
                    .filter(|an_asset_rule| *an_asset_rule != asset_rule)
                    .collect();
            });

            Self::deposit_event(Event::RemoveAssetRule(ticker, asset_rule));

            Ok(())
        }

        /// Removes all active rules of a ticker
        pub fn reset_active_rules(origin, did: IdentityId, ticker: Ticker) -> DispatchResult {
            let sender = Signer::AccountKey(AccountKey::try_from(ensure_signed(origin)?.encode())?);

            ensure!(<identity::Module<T>>::is_signer_authorized(did, &sender), "sender must be a signing key for DID");
            ticker.canonize();
            ensure!(Self::is_owner(&ticker, did), "user is not authorized");

            <ActiveRules>::remove(ticker);

            Self::deposit_event(Event::ResetAssetRules(ticker));

            Ok(())
        }
    }
}

decl_event!(
    pub enum Event {
        NewAssetRule(Ticker, AssetRule),
        RemoveAssetRule(Ticker, AssetRule),
        ResetAssetRules(Ticker),
    }
);

impl<T: Trait> Module<T> {
    fn is_owner(ticker: &Ticker, sender_did: IdentityId) -> bool {
        T::Asset::is_owner(ticker, sender_did)
    }

    fn fetch_value(
        did: IdentityId,
        key: Vec<u8>,
        trusted_issuers: Vec<IdentityId>,
    ) -> Option<ClaimValue> {
        <identity::Module<T>>::fetch_claim_value_multiple_issuers(did, key, trusted_issuers)
    }

    ///  Sender restriction verification
    pub fn verify_restriction(
        ticker: &Ticker,
        from_did_opt: Option<IdentityId>,
        to_did_opt: Option<IdentityId>,
        _value: T::Balance,
    ) -> StdResult<u8, &'static str> {
        // Transfer is valid if All reciever and sender rules of any asset rule are valid.
        let active_rules = Self::active_rules(ticker);
        for active_rule in active_rules {
            let mut rule_broken = false;

            if let Some(from_did) = from_did_opt {
                for sender_rule in active_rule.sender_rules {
                    let identity_value = Self::fetch_value(
                        from_did.clone(),
                        sender_rule.key,
                        sender_rule.trusted_issuers,
                    );
                    rule_broken = match identity_value {
                        None => true,
                        Some(x) => utils::is_rule_broken(
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
            }

            if let Some(to_did) = to_did_opt {
                for receiver_rule in active_rule.receiver_rules {
                    let identity_value = Self::fetch_value(
                        to_did.clone(),
                        receiver_rule.key,
                        receiver_rule.trusted_issuers,
                    );
                    rule_broken = match identity_value {
                        None => true,
                        Some(x) => utils::is_rule_broken(
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
            }

            if !rule_broken {
                return Ok(ERC1400_TRANSFER_SUCCESS);
            }
        }

        sp_runtime::print("Identity TM restrictions not satisfied");
        Ok(ERC1400_TRANSFER_FAILURE)
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::prelude::*;
    use frame_support::traits::Currency;
    use frame_support::{assert_ok, dispatch::DispatchResult, impl_outer_origin, parameter_types};
    use frame_system::EnsureSignedBy;
    use primitives::IdentityId;
    use sp_core::{crypto::key_types, H256};
    use sp_runtime::{
        testing::{Header, UintAuthorityId},
        traits::{BlakeTwo256, ConvertInto, IdentityLookup, OpaqueKeys, Verify},
        AnySignature, KeyTypeId, Perbill,
    };
    use sp_std::result::Result;
    use test_client::{self, AccountKeyring};

    use crate::{
        asset::{AssetType, SecurityToken, TickerRegistrationConfig},
        balances, exemption, group, identity,
        identity::DataTypes,
        percentage_tm, statistics,
    };

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
        pub const MaximumBlockWeight: u32 = 4096;
        pub const MaximumBlockLength: u32 = 4096;
        pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    }

    impl frame_system::Trait for Test {
        type Origin = Origin;
        type Index = u64;
        type BlockNumber = u64;
        type Call = ();
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = AccountId;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type MaximumBlockLength = MaximumBlockLength;
        type AvailableBlockRatio = AvailableBlockRatio;
        type Version = ();
        type ModuleToIndex = ();
    }

    parameter_types! {
        pub const ExistentialDeposit: u64 = 0;
        pub const TransferFee: u64 = 0;
        pub const CreationFee: u64 = 0;
        pub const TransactionBaseFee: u64 = 0;
        pub const TransactionByteFee: u64 = 0;
    }

    impl balances::Trait for Test {
        type Balance = u128;
        type OnFreeBalanceZero = ();
        type OnNewAccount = ();
        type Event = ();
        type DustRemoval = ();
        type TransferPayment = ();
        type ExistentialDeposit = ExistentialDeposit;
        type TransferFee = TransferFee;
        type CreationFee = CreationFee;
        type Identity = crate::identity::Module<Test>;
    }

    parameter_types! {
        pub const MinimumPeriod: u64 = 3;
    }

    type SessionIndex = u32;
    type AuthorityId = <AnySignature as Verify>::Signer;
    type BlockNumber = u64;
    type AccountId = <AnySignature as Verify>::Signer;
    type OffChainSignature = AnySignature;

    impl pallet_timestamp::Trait for Test {
        type Moment = u64;
        type OnTimestampSet = ();
        type MinimumPeriod = MinimumPeriod;
    }

    impl utils::Trait for Test {
        type Public = AccountId;
        type OffChainSignature = OffChainSignature;
        fn validator_id_to_account_id(
            v: <Self as pallet_session::Trait>::ValidatorId,
        ) -> Self::AccountId {
            v
        }
    }

    pub struct TestOnSessionEnding;
    impl pallet_session::OnSessionEnding<AuthorityId> for TestOnSessionEnding {
        fn on_session_ending(_: SessionIndex, _: SessionIndex) -> Option<Vec<AuthorityId>> {
            None
        }
    }

    pub struct TestSessionHandler;
    impl pallet_session::SessionHandler<AuthorityId> for TestSessionHandler {
        const KEY_TYPE_IDS: &'static [KeyTypeId] = &[key_types::DUMMY];
        fn on_new_session<Ks: OpaqueKeys>(
            _changed: bool,
            _validators: &[(AuthorityId, Ks)],
            _queued_validators: &[(AuthorityId, Ks)],
        ) {
        }

        fn on_disabled(_validator_index: usize) {}

        fn on_genesis_session<Ks: OpaqueKeys>(_validators: &[(AuthorityId, Ks)]) {}

        fn on_before_session_ending() {}
    }

    parameter_types! {
        pub const Period: BlockNumber = 1;
        pub const Offset: BlockNumber = 0;
        pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(33);
    }

    impl pallet_session::Trait for Test {
        type OnSessionEnding = TestOnSessionEnding;
        type Keys = UintAuthorityId;
        type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
        type SessionHandler = TestSessionHandler;
        type Event = ();
        type ValidatorId = AuthorityId;
        type ValidatorIdOf = ConvertInto;
        type SelectInitialValidators = ();
        type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
    }

    impl pallet_session::historical::Trait for Test {
        type FullIdentification = ();
        type FullIdentificationOf = ();
    }

    parameter_types! {
        pub const One: AccountId = AccountId::from(AccountKeyring::Dave);
        pub const Two: AccountId = AccountId::from(AccountKeyring::Dave);
        pub const Three: AccountId = AccountId::from(AccountKeyring::Dave);
        pub const Four: AccountId = AccountId::from(AccountKeyring::Dave);
        pub const Five: AccountId = AccountId::from(AccountKeyring::Dave);
    }

    impl group::Trait<group::Instance2> for Test {
        type Event = ();
        type AddOrigin = EnsureSignedBy<One, AccountId>;
        type RemoveOrigin = EnsureSignedBy<Two, AccountId>;
        type SwapOrigin = EnsureSignedBy<Three, AccountId>;
        type ResetOrigin = EnsureSignedBy<Four, AccountId>;
        type MembershipInitialized = ();
        type MembershipChanged = ();
    }

    impl identity::Trait for Test {
        type Event = ();
        type Proposal = Call<Test>;
        type AcceptTransferTarget = asset::Module<Test>;
        type AddSignerMultiSigTarget = Test;
        type KYCServiceProviders = Test;
    }

    impl crate::group::GroupTrait for Test {
        fn get_members() -> Vec<IdentityId> {
            unimplemented!()
        }
        fn is_member(_did: &IdentityId) -> bool {
            unimplemented!()
        }
    }

    impl crate::multisig::AddSignerMultiSig for Test {
        fn accept_multisig_signer(_: Signer, _: u64) -> DispatchResult {
            unimplemented!()
        }
    }

    impl asset::Trait for Test {
        type Event = ();
        type Currency = balances::Module<Test>;
    }

    impl statistics::Trait for Test {}

    impl percentage_tm::Trait for Test {
        type Event = ();
    }

    impl exemption::Trait for Test {
        type Event = ();
        type Asset = asset::Module<Test>;
    }

    impl Trait for Test {
        type Event = ();
        type Asset = asset::Module<Test>;
    }

    type Identity = identity::Module<Test>;
    type GeneralTM = Module<Test>;
    type Balances = balances::Module<Test>;
    type Asset = asset::Module<Test>;

    /// Build a genesis identity instance owned by the specified account
    fn identity_owned_by_alice() -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        identity::GenesisConfig::<Test> {
            owner: AccountKeyring::Alice.public().into(),
            did_creation_fee: 250,
        }
        .assimilate_storage(&mut t)
        .unwrap();
        asset::GenesisConfig::<Test> {
            asset_creation_fee: 0,
            ticker_registration_fee: 0,
            ticker_registration_config: TickerRegistrationConfig {
                max_ticker_length: 12,
                registration_length: Some(10000),
            },
            fee_collector: AccountKeyring::Dave.public().into(),
        }
        .assimilate_storage(&mut t)
        .unwrap();
        sp_io::TestExternalities::new(t)
    }

    fn make_account(
        account_id: &AccountId,
    ) -> Result<(<Test as frame_system::Trait>::Origin, IdentityId), &'static str> {
        let signed_id = Origin::signed(account_id.clone());
        Balances::make_free_balance_be(&account_id, 1_000_000);
        Identity::register_did(signed_id.clone(), vec![]);
        let did = Identity::get_identity(&AccountKey::try_from(account_id.encode())?).unwrap();
        Ok((signed_id, did))
    }

    #[test]
    fn should_add_and_verify_assetrule() {
        identity_owned_by_alice().execute_with(|| {
            let token_owner_acc = AccountId::from(AccountKeyring::Alice);
            let (token_owner_signed, token_owner_did) = make_account(&token_owner_acc).unwrap();

            // A token representing 1M shares
            let token = SecurityToken {
                name: vec![0x01],
                owner_did: token_owner_did.clone(),
                total_supply: 1_000_000,
                divisible: true,
                asset_type: AssetType::default(),
            };
            let ticker = Ticker::from_slice(token.name.as_slice());
            Balances::make_free_balance_be(&token_owner_acc, 1_000_000);

            // Share issuance is successful
            assert_ok!(Asset::create_token(
                token_owner_signed.clone(),
                token_owner_did,
                token.name.clone(),
                ticker,
                token.total_supply,
                true,
                token.asset_type.clone(),
                vec![],
            ));
            let claim_issuer_acc = AccountId::from(AccountKeyring::Bob);
            Balances::make_free_balance_be(&claim_issuer_acc, 1_000_000);
            let (_claim_issuer, claim_issuer_did) =
                make_account(&claim_issuer_acc.clone()).unwrap();

            let claim_value = ClaimValue {
                data_type: DataTypes::VecU8,
                value: "some_value".as_bytes().to_vec(),
            };

            assert_ok!(Identity::add_claim(
                Origin::signed(claim_issuer_acc.clone()),
                token_owner_did,
                "some_key".as_bytes().to_vec(),
                claim_issuer_did,
                99999999999999999u64,
                claim_value.clone()
            ));

            let now = Utc::now();
            <pallet_timestamp::Module<Test>>::set_timestamp(now.timestamp() as u64);

            let sender_rule = RuleData {
                key: "some_key".as_bytes().to_vec(),
                value: "some_value".as_bytes().to_vec(),
                trusted_issuers: vec![claim_issuer_did],
                operator: Operators::EqualTo,
            };

            let x = vec![sender_rule];

            let asset_rule = AssetRule {
                sender_rules: x,
                receiver_rules: vec![],
            };

            // Allow all transfers
            assert_ok!(GeneralTM::add_active_rule(
                token_owner_signed.clone(),
                token_owner_did,
                ticker,
                asset_rule
            ));

            //Transfer tokens to investor
            assert_ok!(Asset::transfer(
                token_owner_signed.clone(),
                token_owner_did,
                ticker,
                token_owner_did,
                token.total_supply
            ));
        });
    }

    #[test]
    fn should_add_and_verify_complex_assetrule() {
        identity_owned_by_alice().execute_with(|| {
            let token_owner_acc = AccountId::from(AccountKeyring::Alice);
            let (token_owner_signed, token_owner_did) = make_account(&token_owner_acc).unwrap();

            // A token representing 1M shares
            let token = SecurityToken {
                name: vec![0x01],
                owner_did: token_owner_did.clone(),
                total_supply: 1_000_000,
                divisible: true,
                asset_type: AssetType::default(),
            };
            let ticker = Ticker::from_slice(token.name.as_slice());
            Balances::make_free_balance_be(&token_owner_acc, 1_000_000);

            // Share issuance is successful
            assert_ok!(Asset::create_token(
                token_owner_signed.clone(),
                token_owner_did,
                token.name.clone(),
                ticker,
                token.total_supply,
                true,
                token.asset_type.clone(),
                vec![],
            ));
            let claim_issuer_acc = AccountId::from(AccountKeyring::Bob);
            Balances::make_free_balance_be(&claim_issuer_acc, 1_000_000);
            let (_claim_issuer, claim_issuer_did) =
                make_account(&claim_issuer_acc.clone()).unwrap();

            let claim_value = ClaimValue {
                data_type: DataTypes::U8,
                value: 10u8.encode(),
            };

            assert_ok!(Identity::add_claim(
                Origin::signed(claim_issuer_acc.clone()),
                token_owner_did,
                "some_key".as_bytes().to_vec(),
                claim_issuer_did,
                99999999999999999u64,
                claim_value.clone()
            ));

            let now = Utc::now();
            <pallet_timestamp::Module<Test>>::set_timestamp(now.timestamp() as u64);

            let sender_rule = RuleData {
                key: "some_key".as_bytes().to_vec(),
                value: 5u8.encode(),
                trusted_issuers: vec![claim_issuer_did],
                operator: Operators::GreaterThan,
            };

            let receiver_rule = RuleData {
                key: "some_key".as_bytes().to_vec(),
                value: 15u8.encode(),
                trusted_issuers: vec![claim_issuer_did],
                operator: Operators::LessThan,
            };

            let x = vec![sender_rule];
            let y = vec![receiver_rule];

            let asset_rule = AssetRule {
                sender_rules: x,
                receiver_rules: y,
            };

            assert_ok!(GeneralTM::add_active_rule(
                token_owner_signed.clone(),
                token_owner_did,
                ticker,
                asset_rule
            ));

            //Transfer tokens to investor
            assert_ok!(Asset::transfer(
                token_owner_signed.clone(),
                token_owner_did,
                ticker,
                token_owner_did.clone(),
                token.total_supply
            ));
        });
    }

    #[test]
    fn should_reset_assetrules() {
        identity_owned_by_alice().execute_with(|| {
            let token_owner_acc = AccountId::from(AccountKeyring::Alice);
            let (token_owner_signed, token_owner_did) = make_account(&token_owner_acc).unwrap();

            // A token representing 1M shares
            let token = SecurityToken {
                name: vec![0x01],
                owner_did: token_owner_did,
                total_supply: 1_000_000,
                divisible: true,
                asset_type: AssetType::default(),
            };
            let ticker = Ticker::from_slice(token.name.as_slice());
            Balances::make_free_balance_be(&token_owner_acc, 1_000_000);

            // Share issuance is successful
            assert_ok!(Asset::create_token(
                token_owner_signed.clone(),
                token_owner_did,
                token.name.clone(),
                ticker,
                token.total_supply,
                true,
                token.asset_type.clone(),
                vec![],
            ));

            let asset_rule = AssetRule {
                sender_rules: vec![],
                receiver_rules: vec![],
            };

            assert_ok!(GeneralTM::add_active_rule(
                token_owner_signed.clone(),
                token_owner_did,
                ticker,
                asset_rule
            ));

            let asset_rules = GeneralTM::active_rules(ticker);
            assert_eq!(asset_rules.len(), 1);

            assert_ok!(GeneralTM::reset_active_rules(
                token_owner_signed.clone(),
                token_owner_did,
                ticker
            ));

            let asset_rules_new = GeneralTM::active_rules(ticker);
            assert_eq!(asset_rules_new.len(), 0);
        });
    }
}
