//! # Simple Token Module
//!
//! The Simple Token module provides functionality for issuing and managing tokens which do not have transfer restrictions.
//!
//! ## Overview
//!
//! The Simple Token module provides functions for:
//!
//! - Creating a simple token with an inital balance
//! - Transfering simple tokens between identities
//! - Approving simple tokens to be transferred on your behalf by another identity
//!
//! ### Use case
//!
//! In some cases the asset module may be unnecessary. For example a token representing USD may not need transfer restrictions
//! that are typically associated with securities.
//!
//! In other cases a simple token may be used to represent a wrapped asset that originates on a different chain, for example BTC,
//! which by its nature does not need transfer restrictions.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `create_token` - Creates a new simple token and mints a balance to the issuer
//! - `approve` - Approves another identity to transfer tokens on behalf of the caller
//! - `transfer` - Transfers simple tokens to another identity
//! - `transfer_from` - Transfers simple tokens to another identity using the approval process
//!
//! ### Public Functions
//!
//! - `balance_of` - Returns the simple token balance associated with an identity

use crate::utils;

use polymesh_primitives::{AccountKey, IdentityId, Signatory, Ticker};
use polymesh_runtime_common::{
    balances::Trait as BalancesTrait, constants::currency::MAX_SUPPLY,
    identity::Trait as IdentityTrait, CommonTrait,
};
use polymesh_runtime_identity as identity;

use codec::Encode;
use sp_std::{convert::TryFrom, prelude::*};

use frame_support::{decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure};
use frame_system::{self as system, ensure_signed};
use sp_runtime::traits::{CheckedAdd, CheckedSub};

/// The module's configuration trait.
pub trait Trait: frame_system::Trait + BalancesTrait + utils::Trait + IdentityTrait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

/// Struct to store the details of each simple token
#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct SimpleTokenRecord<U> {
    pub ticker: Ticker,
    pub total_supply: U,
    pub owner_did: IdentityId,
}

decl_storage! {
    trait Store for Module<T: Trait> as SimpleToken {
        /// Mapping from (ticker, owner DID, spender DID) to allowance amount
        Allowance get(fn allowance): map (Ticker, IdentityId, IdentityId) => T::Balance;
        /// Mapping from (ticker, owner DID) to their balance
        pub BalanceOf get(fn balance_of): map (Ticker, IdentityId) => T::Balance;
        /// The cost to create a new simple token
        CreationFee get(fn creation_fee) config(): T::Balance;
        /// The details associated with each simple token
        Tokens get(fn tokens): map Ticker => SimpleTokenRecord<T::Balance>;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event() = default;

        /// Create a new token and mint a balance to the issuing identity
        pub fn create_token(origin, did: IdentityId, ticker: Ticker, total_supply: T::Balance) -> DispatchResult {
            let sender = Signatory::AccountKey(AccountKey::try_from(ensure_signed(origin)?.encode())?);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signer_authorized(did, &sender), "sender must be a signing key for DID");
            ticker.canonize();
            ensure!(!<Tokens<T>>::exists(&ticker), "Ticker with this name already exists");
            ensure!(ticker.len() <= 32, "token ticker cannot exceed 32 bytes");
            ensure!(total_supply <= MAX_SUPPLY.into(), "Total supply above the limit");

            // TODO Charge proper fee
            // <identity::DidRecords<T>>::mutate( did, |record| -> Result {
            //     record.balance = record.balance.checked_sub(&Self::creation_fee()).ok_or("Could not charge for token issuance")?;
            //     Ok(())
            // })?;

            let new_token = SimpleTokenRecord {
                ticker: ticker,
                total_supply: total_supply.clone(),
                owner_did: did.clone(),
            };

            <Tokens<T>>::insert(&ticker, new_token);
            // Let the owner distribute the whole supply of the token
            <BalanceOf<T>>::insert((ticker, did.clone()), total_supply);

            sp_runtime::print("Initialized a new token");

            Self::deposit_event(RawEvent::TokenCreated(ticker, did, total_supply));

            Ok(())
        }

        /// Approve another identity to transfer tokens on behalf of the caller
        fn approve(origin, did: IdentityId, ticker: Ticker, spender_did: IdentityId, value: T::Balance) -> DispatchResult {
            let sender = Signatory::AccountKey(AccountKey::try_from(ensure_signed(origin)?.encode())?);
            ticker.canonize();
            let ticker_did = (ticker, did.clone());
            ensure!(<BalanceOf<T>>::exists(&ticker_did), "Account does not own this token");

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signer_authorized(did, &sender), "sender must be a signing key for DID");

            let ticker_did_spender_did = (ticker, did, spender_did);
            let allowance = Self::allowance(&ticker_did_spender_did);
            let updated_allowance = allowance.checked_add(&value).ok_or("overflow in calculating allowance")?;
            <Allowance<T>>::insert(&ticker_did_spender_did, updated_allowance);

            Self::deposit_event(RawEvent::Approval(ticker, did, spender_did, value));

            Ok(())
        }

        /// Transfer tokens to another identity
        pub fn transfer(origin, did: IdentityId, ticker: Ticker, to_did: IdentityId, amount: T::Balance) -> DispatchResult {
            ticker.canonize();
            let sender = Signatory::AccountKey(AccountKey::try_from(ensure_signed(origin)?.encode())?);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signer_authorized(did, &sender), "sender must be a signing key for DID");

            Self::_transfer(&ticker, did, to_did, amount)
        }

        /// Transfer tokens to another identity using the approval mechanic
        fn transfer_from(origin, did: IdentityId, ticker: Ticker, from_did: IdentityId, to_did: IdentityId, amount: T::Balance) -> DispatchResult {
            let spender = Signatory::AccountKey(AccountKey::try_from(ensure_signed(origin)?.encode())?);

            // Check that spender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signer_authorized(did, &spender), "spender must be a signing key for DID");
            ticker.canonize();
            let ticker_from_did_did = (ticker, from_did, did);
            ensure!(<Allowance<T>>::exists(&ticker_from_did_did), "Allowance does not exist.");
            let allowance = Self::allowance(&ticker_from_did_did);
            ensure!(allowance >= amount, "Not enough allowance.");

            // Needs to happen before allowance subtraction so that the from balance is checked in _transfer
            Self::_transfer(&ticker, from_did, to_did, amount)?;

            // using checked_sub (safe math) to avoid overflow
            let updated_allowance = allowance.checked_sub(&amount).ok_or("overflow in calculating allowance")?;
            <Allowance<T>>::insert((ticker, from_did.clone(), did.clone()), updated_allowance);

            Self::deposit_event(RawEvent::Approval(ticker, from_did, did, updated_allowance));

            Ok(())
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        Balance = <T as CommonTrait>::Balance,
    {
        /// ticker, from DID, spender DID, amount
        Approval(Ticker, IdentityId, IdentityId, Balance),
        /// ticker, owner DID, supply
        TokenCreated(Ticker, IdentityId, Balance),
        /// ticker, from DID, to DID, amount
        Transfer(Ticker, IdentityId, IdentityId, Balance),
    }
);

pub trait SimpleTokenTrait<V> {
    /// Tranfers tokens between two identities
    fn transfer(
        sender_did: IdentityId,
        ticker: &Ticker,
        to_did: IdentityId,
        amount: V,
    ) -> DispatchResult;
    /// Returns the balance associated with an identity and ticker
    fn balance_of(ticker: Ticker, owner_did: IdentityId) -> V;
}

impl<T: Trait> SimpleTokenTrait<T::Balance> for Module<T> {
    /// Tranfers tokens between two identities
    fn transfer(
        sender_did: IdentityId,
        ticker: &Ticker,
        to_did: IdentityId,
        amount: T::Balance,
    ) -> DispatchResult {
        Self::_transfer(ticker, sender_did, to_did, amount)
    }
    /// Returns the balance associated with an identity and ticker
    fn balance_of(ticker: Ticker, owner_did: IdentityId) -> T::Balance {
        Self::balance_of((ticker, owner_did))
    }
}

impl<T: Trait> Module<T> {
    fn _transfer(
        ticker: &Ticker,
        from_did: IdentityId,
        to_did: IdentityId,
        amount: T::Balance,
    ) -> DispatchResult {
        let ticker_from_did = (*ticker, from_did.clone());
        ensure!(
            <BalanceOf<T>>::exists(&ticker_from_did),
            "Sender doesn't own this token"
        );
        let from_balance = Self::balance_of(&ticker_from_did);
        ensure!(from_balance >= amount, "Insufficient balance");

        let new_from_balance = from_balance
            .checked_sub(&amount)
            .ok_or("overflow in calculating from balance")?;
        let ticker_to_did = (*ticker, to_did.clone());
        let to_balance = Self::balance_of(&ticker_to_did);
        let new_to_balance = to_balance
            .checked_add(&amount)
            .ok_or("overflow in calculating to balanc")?;

        <BalanceOf<T>>::insert(&ticker_from_did, new_from_balance);
        <BalanceOf<T>>::insert(&ticker_to_did, new_to_balance);

        Self::deposit_event(RawEvent::Transfer(*ticker, from_did, to_did, amount));
        Ok(())
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use core::result::Result as StdResult;
    use polymesh_primitives::{IdentityId, Signatory};
    use polymesh_runtime_balances as balances;
    use polymesh_runtime_common::traits::{
        asset::AcceptTransfer, multisig::AddSignerMultiSig, CommonTrait,
    };
    use polymesh_runtime_group as group;
    use polymesh_runtime_identity as identity;

    use frame_support::{
        assert_err, assert_ok, dispatch::DispatchResult, impl_outer_origin, parameter_types,
    };
    use sp_core::{crypto::key_types, H256};
    use sp_runtime::{
        testing::{Header, UintAuthorityId},
        traits::{BlakeTwo256, ConvertInto, IdentityLookup, OpaqueKeys, Verify},
        AnySignature, KeyTypeId, Perbill,
    };
    use test_client::{self, AccountKeyring};

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    type SessionIndex = u32;
    type AuthorityId = <AnySignature as Verify>::Signer;
    type BlockNumber = u64;
    type AccountId = <AnySignature as Verify>::Signer;
    type OffChainSignature = AnySignature;

    #[derive(Clone, Eq, PartialEq, Debug)]
    pub struct Test;

    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const MaximumBlockWeight: u32 = 1024;
        pub const MaximumBlockLength: u32 = 2 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::one();
    }

    impl frame_system::Trait for Test {
        type Origin = Origin;
        type Index = u64;
        type BlockNumber = BlockNumber;
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

    impl CommonTrait for Test {
        type Balance = u128;
        type CreationFee = CreationFee;
        type AcceptTransferTarget = Test;
        type BlockRewardsReserve = balances::Module<Test>;
    }

    impl balances::Trait for Test {
        type OnFreeBalanceZero = ();
        type OnNewAccount = ();
        type Event = ();
        type DustRemoval = ();
        type TransferPayment = ();
        type ExistentialDeposit = ExistentialDeposit;
        type TransferFee = TransferFee;
        type Identity = identity::Module<Test>;
    }

    impl identity::Trait for Test {
        type Event = ();
        type Proposal = Call<Test>;
        type AddSignerMultiSigTarget = Test;
        type KycServiceProviders = Test;
        type Balances = balances::Module<Test>;
    }

    impl group::GroupTrait for Test {
        fn get_members() -> Vec<IdentityId> {
            unimplemented!()
        }
        fn is_member(_did: &IdentityId) -> bool {
            unimplemented!()
        }
    }

    impl AddSignerMultiSig for Test {
        fn accept_multisig_signer(_: Signatory, _: u64) -> DispatchResult {
            unimplemented!()
        }
    }

    impl AcceptTransfer for Test {
        fn accept_ticker_transfer(_: IdentityId, _: u64) -> DispatchResult {
            unimplemented!()
        }
        fn accept_token_ownership_transfer(_: IdentityId, _: u64) -> DispatchResult {
            unimplemented!()
        }
    }

    parameter_types! {
        pub const MinimumPeriod: u64 = 3;
    }

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

    impl Trait for Test {
        type Event = ();
    }

    type Identity = identity::Module<Test>;
    type SimpleToken = Module<Test>;

    fn new_test_ext() -> sp_io::TestExternalities {
        let t = system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        t.into()
    }

    fn make_account(
        account_id: &AccountId,
    ) -> StdResult<(<Test as frame_system::Trait>::Origin, IdentityId), &'static str> {
        let signed_id = Origin::signed(account_id.clone());
        let _ = Identity::register_did(signed_id.clone(), vec![]);
        let did = Identity::get_identity(&AccountKey::try_from(account_id.encode())?).unwrap();
        Ok((signed_id, did))
    }

    #[test]
    fn create_token_works() {
        new_test_ext().execute_with(|| {
            let owner_acc = AccountId::from(AccountKeyring::Alice);
            let (owner_signed, owner_did) = make_account(&owner_acc).unwrap();

            let ticker = Ticker::from_slice(&[0x01]);
            let total_supply = 1_000_000;

            // Issuance is successful
            assert_ok!(SimpleToken::create_token(
                owner_signed.clone(),
                owner_did,
                ticker,
                total_supply
            ));

            assert_eq!(
                SimpleToken::tokens(ticker),
                SimpleTokenRecord {
                    ticker,
                    total_supply,
                    owner_did
                }
            );

            assert_err!(
                SimpleToken::create_token(owner_signed.clone(), owner_did, ticker, total_supply),
                "Ticker with this name already exists"
            );

            assert_ok!(SimpleToken::create_token(
                owner_signed.clone(),
                owner_did,
                Ticker::from_slice("1234567890123456789012345678901234567890".as_bytes()),
                total_supply,
            ));
            assert_eq!(
                SimpleToken::tokens(Ticker::from_slice(
                    "1234567890123456789012345678901234567890".as_bytes()
                )),
                SimpleTokenRecord {
                    ticker: Ticker::from_slice("123456789012".as_bytes()),
                    total_supply,
                    owner_did
                }
            );

            assert_err!(
                SimpleToken::create_token(
                    owner_signed.clone(),
                    owner_did,
                    Ticker::from_slice(&[0x02]),
                    MAX_SUPPLY + 1
                ),
                "Total supply above the limit"
            );
        });
    }

    #[test]
    fn transfer_works() {
        new_test_ext().execute_with(|| {
            let owner_acc = AccountId::from(AccountKeyring::Alice);
            let (owner_signed, owner_did) = make_account(&owner_acc).unwrap();

            let spender_acc = AccountId::from(AccountKeyring::Bob);
            let (spender_signed, spender_did) = make_account(&spender_acc).unwrap();

            let ticker = Ticker::from_slice(&[0x01]);
            let total_supply = 1_000_000;

            // Issuance is successful
            assert_ok!(SimpleToken::create_token(
                owner_signed.clone(),
                owner_did,
                ticker,
                total_supply
            ));

            let gift = 1000u128;
            assert_err!(
                SimpleToken::transfer(spender_signed.clone(), spender_did, ticker, owner_did, gift),
                "Sender doesn't own this token"
            );

            assert_ok!(SimpleToken::transfer(
                owner_signed.clone(),
                owner_did,
                ticker,
                spender_did,
                gift
            ));
            assert_eq!(
                SimpleToken::balance_of((ticker, owner_did)),
                total_supply - gift
            );
            assert_eq!(SimpleToken::balance_of((ticker, spender_did)), gift);
        });
    }

    #[test]
    fn approve_transfer_works() {
        new_test_ext().execute_with(|| {
            let owner_acc = AccountId::from(AccountKeyring::Alice);
            let (owner_signed, owner_did) = make_account(&owner_acc).unwrap();

            let spender_acc = AccountId::from(AccountKeyring::Bob);
            let (spender_signed, spender_did) = make_account(&spender_acc).unwrap();

            let agent_acc = AccountId::from(AccountKeyring::Bob);
            let (agent_signed, agent_did) = make_account(&agent_acc).unwrap();

            let ticker = Ticker::from_slice(&[0x01]);
            let total_supply = 1_000_000;

            // Issuance is successful
            assert_ok!(SimpleToken::create_token(
                owner_signed.clone(),
                owner_did,
                ticker,
                total_supply
            ));

            let allowance = 1000u128;

            assert_err!(
                SimpleToken::approve(
                    spender_signed.clone(),
                    spender_did,
                    ticker,
                    spender_did,
                    allowance
                ),
                "Account does not own this token"
            );

            assert_ok!(SimpleToken::approve(
                owner_signed.clone(),
                owner_did,
                ticker,
                spender_did,
                allowance
            ));
            assert_eq!(
                SimpleToken::allowance((ticker, owner_did, spender_did)),
                allowance
            );

            assert_err!(
                SimpleToken::approve(
                    owner_signed.clone(),
                    owner_did,
                    ticker,
                    spender_did,
                    std::u128::MAX
                ),
                "overflow in calculating allowance"
            );

            assert_err!(
                SimpleToken::transfer_from(
                    agent_signed.clone(),
                    agent_did,
                    ticker,
                    owner_did,
                    spender_did,
                    allowance + 1u128
                ),
                "Not enough allowance."
            );

            assert_ok!(SimpleToken::transfer_from(
                agent_signed.clone(),
                agent_did,
                ticker,
                owner_did,
                spender_did,
                allowance
            ));
            assert_eq!(
                SimpleToken::balance_of((ticker, owner_did)),
                total_supply - allowance
            );
            assert_eq!(SimpleToken::balance_of((ticker, spender_did)), allowance);
        });
    }
}
