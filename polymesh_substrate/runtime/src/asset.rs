use crate::general_tm;
use crate::identity;
use crate::percentage_tm;
use crate::utils;
use rstd::prelude::*;
//use parity_codec::Codec;
use runtime_primitives::traits::{As, CheckedAdd, CheckedSub, Convert};
use session;
use support::traits::{Currency, ExistenceRequirement, WithdrawReason};
use support::{decl_event, decl_module, decl_storage, dispatch::Result, ensure, StorageMap};
use system::{self, ensure_signed};

type FeeOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

/// The module's configuration trait.
pub trait Trait:
    system::Trait
    + general_tm::Trait
    + percentage_tm::Trait
    + utils::Trait
    + balances::Trait
    + identity::Trait
    + session::Trait
{
    // TODO: Add other types and constants required configure this module.

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    //type TokenBalance: Parameter + Member + SimpleArithmetic + Codec + Default + Copy + As<usize> + As<u64>;
    type Currency: Currency<Self::AccountId>;
    // Handler for the unbalanced decrease when charging fee
    type CurrencyToBalance: Convert<FeeOf<Self>, <Self as balances::Trait>::Balance>;
}

// struct to store the token details
#[derive(parity_codec::Encode, parity_codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct SecurityToken<U, V> {
    pub name: Vec<u8>,
    pub total_supply: U,
    pub owner: V,
    pub granularity: u128,
    pub decimals: u32,
}

decl_storage! {
    trait Store for Module<T: Trait> as Asset {
        FeeCollector get(fee_collector) config(): T::AccountId;
        // details of the token corresponding to the token ticker
        Tokens get(token_details): map Vec<u8> => SecurityToken<T::TokenBalance, T::AccountId>;
        // balances mapping for an account and token
        BalanceOf get(balance_of): map (Vec<u8>, T::AccountId) => T::TokenBalance;
        // allowance for an account and token
        Allowance get(allowance): map (Vec<u8>, T::AccountId, T::AccountId) => T::TokenBalance;
        // cost in base currency to create a token
        AssetCreationFee get(asset_creation_fee) config(): FeeOf<T>;
        // Checkpoints created per token
        TotalCheckpoints get(total_checkpoints_of): map (Vec<u8>) => u32;
        // Total supply of the token at the checkpoint
        CheckpointTotalSupply get(total_supply_at): map (Vec<u8>, u32) => T::TokenBalance;
        // Balance of a user at a checkpoint
        CheckpointBalance get(balance_at_checkpoint): map (Vec<u8>, T::AccountId, u32) => Option<T::TokenBalance>;
        // Last checkpoint updated for user balance
        LatestUserCheckpoint get(latest_user_checkpoint): map (Vec<u8>, T::AccountId) => u32;
    }
}

// public interface for this runtime module
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // initialize the default event for this module
        fn deposit_event<T>() = default;

        // initializes a new token
        // takes a name, ticker, total supply for the token
        // makes the initiating account the owner of the token
        // the balance of the owner is set to total supply
        fn issue_token(origin, name: Vec<u8>, _ticker: Vec<u8>, total_supply: T::TokenBalance, divisible: bool) -> Result {
            let ticker = utils::bytes_to_upper(_ticker.as_slice());
            let sender = ensure_signed(origin)?;
            ensure!(<identity::Module<T>>::is_issuer(sender.clone()),"user is not authorized");
            // Ensure the uniqueness of the ticker
            ensure!(!<Tokens<T>>::exists(ticker.clone()), "ticker is already issued");
            // checking max size for name and ticker
            // byte arrays (vecs) with no max size should be avoided
            ensure!(name.len() <= 64, "token name cannot exceed 64 bytes");
            ensure!(ticker.len() <= 32, "token ticker cannot exceed 32 bytes");

            let mut granularity  = 1 as u128;

            if !divisible {
                granularity = (10 as u128).pow(18);
            }

            ensure!(<T as utils::Trait>::as_u128(total_supply) % granularity == (0 as u128), "Invalid Total supply");

            // Alternative way to take a fee - fee is proportionaly paid to the validators and dust is burned
            let validators = <session::Module<T>>::validators();
            let fee = Self::asset_creation_fee();
            let validatorLen;
            if validators.len() < 1 {
                validatorLen = <FeeOf<T> as As<usize>>::sa(1);
            } else {
                validatorLen = <FeeOf<T> as As<usize>>::sa(validators.len());
            }
            let proportional_fee = fee / validatorLen;
            let proportional_fee_in_balance = <T::CurrencyToBalance as Convert<FeeOf<T>, T::Balance>>::convert(proportional_fee);
            for v in &validators {
                <balances::Module<T> as Currency<_>>::transfer(&sender, v, proportional_fee_in_balance)?;
            }
            let remainder_fee = fee - (proportional_fee * validatorLen);
            let _imbalance = T::Currency::withdraw(&sender, remainder_fee, WithdrawReason::Fee, ExistenceRequirement::KeepAlive)?;

            let token = SecurityToken {
                name,
                total_supply,
                owner:sender.clone(),
                granularity:granularity,
                decimals:18
            };

            <Tokens<T>>::insert(ticker.clone(), token);
            <BalanceOf<T>>::insert((ticker.clone(), sender.clone()), total_supply);
            Self::deposit_event(RawEvent::IssuedToken(ticker, total_supply, sender, granularity, 18));
            runtime_io::print("Initialized!!!");

            Ok(())
        }

        // transfer tokens from one account to another
        // origin is assumed as sender
        fn transfer(_origin, _ticker: Vec<u8>, to: T::AccountId, value: T::TokenBalance) -> Result {
            let ticker = utils::bytes_to_upper(_ticker.as_slice());
            let sender = ensure_signed(_origin)?;
            Self::_is_valid_transfer(ticker.clone(), sender.clone(), to.clone(), value)?;

            Self::_transfer(ticker.clone(), sender, to, value)
        }

        // transfer tokens from one account to another
        // origin is assumed as sender
        fn force_transfer(_origin, _ticker: Vec<u8>, from: T::AccountId, to: T::AccountId, value: T::TokenBalance) -> Result {
            let ticker = utils::bytes_to_upper(_ticker.as_slice());
            let sender = ensure_signed(_origin)?;
            ensure!(Self::is_owner(ticker.clone(), sender.clone()), "user is not authorized");

            Self::_transfer(ticker.clone(), from.clone(), to.clone(), value.clone())?;

            Self::deposit_event(RawEvent::ForcedTransfer(ticker.clone(), from, to, value));

            Ok(())
        }

        // approve token transfer from one account to another
        // once this is done, transfer_from can be called with corresponding values
        fn approve(_origin, _ticker: Vec<u8>, spender: T::AccountId, value: T::TokenBalance) -> Result {
            let ticker = utils::bytes_to_upper(_ticker.as_slice());
            let sender = ensure_signed(_origin)?;
            ensure!(<BalanceOf<T>>::exists((ticker.clone(), sender.clone())), "Account does not own this token");

            let allowance = Self::allowance((ticker.clone(), sender.clone(), spender.clone()));
            let updated_allowance = allowance.checked_add(&value).ok_or("overflow in calculating allowance")?;
            <Allowance<T>>::insert((ticker.clone(), sender.clone(), spender.clone()), updated_allowance);

            Self::deposit_event(RawEvent::Approval(ticker.clone(), sender.clone(), spender.clone(), value));

            Ok(())
        }

        // the ERC20 standard transfer_from function
        // implemented in the open-zeppelin way - increase/decrease allownace
        // if approved, transfer from an account to another account without owner's signature
        pub fn transfer_from(_origin, _ticker: Vec<u8>, from: T::AccountId, to: T::AccountId, value: T::TokenBalance) -> Result {
            let spender = ensure_signed(_origin)?;
            let ticker = utils::bytes_to_upper(_ticker.as_slice());
            ensure!(<Allowance<T>>::exists((ticker.clone(), from.clone(), spender.clone())), "Allowance does not exist");
            let allowance = Self::allowance((ticker.clone(), from.clone(), spender.clone()));
            ensure!(allowance >= value, "Not enough allowance");

            // using checked_sub (safe math) to avoid overflow
            let updated_allowance = allowance.checked_sub(&value).ok_or("overflow in calculating allowance")?;

            Self::_is_valid_transfer(ticker.clone(), from.clone(), to.clone(), value)?;

            Self::_transfer(ticker.clone(), from.clone(), to.clone(), value)?;

            // Change allowance afterwards
            <Allowance<T>>::insert((ticker.clone(), from.clone(), spender.clone()), updated_allowance);

            Self::deposit_event(RawEvent::Approval(ticker.clone(), from.clone(), spender.clone(), value));
            Ok(())
        }

        // called by issuer to create checkpoints
        pub fn create_checkpoint(_origin, _ticker: Vec<u8>) -> Result {
            let ticker = utils::bytes_to_upper(_ticker.as_slice());
            let sender = ensure_signed(_origin)?;

            ensure!(Self::is_owner(ticker.clone(), sender.clone()), "user is not authorized");
            Self::_create_checkpoint(ticker.clone())
        }

        // called by issuer to create checkpoints
        pub fn balance_at(_origin, _ticker: Vec<u8>, owner: T::AccountId, checkpoint: u32) -> Result {
            ensure_signed(_origin)?; //not needed
            let ticker = utils::bytes_to_upper(_ticker.as_slice());
            Self::deposit_event(RawEvent::BalanceAt(ticker.clone(), owner.clone(), checkpoint, Self::get_balance_at(ticker.clone(), owner, checkpoint)));
            Ok(())
        }

        pub fn mint(_origin, _ticker: Vec<u8>, to: T::AccountId, value: T::TokenBalance) -> Result {
            let ticker = utils::bytes_to_upper(_ticker.clone().as_slice());
            let sender = ensure_signed(_origin)?;

            ensure!(Self::is_owner(ticker.clone(), sender.clone()), "user is not authorized");
            Self::_mint(ticker,to,value)
        }

        pub fn burn(_origin, _ticker: Vec<u8>, value: T::TokenBalance) -> Result {
            let ticker = utils::bytes_to_upper(_ticker.as_slice());
            let sender = ensure_signed(_origin)?;

            ensure!(<BalanceOf<T>>::exists((ticker.clone(), sender.clone())), "Account does not own this token");
            let burner_balance = Self::balance_of((ticker.clone(), sender.clone()));
            ensure!(burner_balance >= value, "Not enough balance.");

            // Reduce sender's balance
            let updated_burner_balance = burner_balance
                .checked_sub(&value)
                .ok_or("overflow in calculating balance")?;

            //PABLO: TODO: Add verify transfer check

            //Decrease total suply
            let mut token = Self::token_details(ticker.clone());
            token.total_supply = token.total_supply.checked_sub(&value).ok_or("overflow in calculating balance")?;

            Self::_update_checkpoint(ticker.clone(), sender.clone(), burner_balance)?;

            <BalanceOf<T>>::insert((ticker.clone(), sender.clone()), updated_burner_balance);
            <Tokens<T>>::insert(ticker.clone(), token);

            Self::deposit_event(RawEvent::Burned(ticker.clone(), sender, value));

            Ok(())

        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        Balance = <T as utils::Trait>::TokenBalance,
    {
        // event for transfer of tokens
        // ticker, from, to, value
        Transfer(Vec<u8>, AccountId, AccountId, Balance),
        // event when an approval is made
        // ticker, owner, spender, value
        Approval(Vec<u8>, AccountId, AccountId, Balance),
        // event - used for testing in the absence of custom getters
        // ticker, owner, checkpoint, balance
        BalanceAt(Vec<u8>, AccountId, u32, Balance),
        // event mint
        // ticker, account, value
        Minted(Vec<u8>, AccountId, Balance),
        // event burn
        // ticker, account, value
        Burned(Vec<u8>, AccountId, Balance),
        // event for forced transfer of tokens
        // ticker, from, to, value
        ForcedTransfer(Vec<u8>, AccountId, AccountId, Balance),
        // Event for creation of the asset
        // ticker, total supply, owner, granularity, decimal
        IssuedToken(Vec<u8>, Balance, AccountId, u128, u32),
    }
);

pub trait AssetTrait<T, V> {
    fn _mint_from_sto(ticker: Vec<u8>, sender: T, tokens_purchased: V) -> Result;

    fn is_owner(_ticker: Vec<u8>, who: T) -> bool;
}

impl<T: Trait> AssetTrait<T::AccountId, T::TokenBalance> for Module<T> {
    fn _mint_from_sto(
        ticker: Vec<u8>,
        sender: T::AccountId,
        tokens_purchased: T::TokenBalance,
    ) -> Result {
        let _ticker = utils::bytes_to_upper(ticker.as_slice());
        Self::_mint(_ticker, sender, tokens_purchased)
    }

    fn is_owner(_ticker: Vec<u8>, sender: T::AccountId) -> bool {
        let token = Self::token_details(_ticker);
        token.owner == sender
    }
}

/// All functions in the decl_module macro become part of the public interface of the module
/// If they are there, they are accessible via extrinsics calls whether they are public or not
/// However, in the impl module section (this, below) the functions can be public and private
/// Private functions are internal to this module e.g.: _transfer
/// Public functions can be called from other modules e.g.: lock and unlock (being called from the tcr module)
/// All functions in the impl module section are not part of public interface because they are not part of the Call enum
impl<T: Trait> Module<T> {
    // Public immutables

    /// Get the asset `id` balance of `who`.
    pub fn balance(_ticker: Vec<u8>, who: T::AccountId) -> T::TokenBalance {
        let ticker = utils::bytes_to_upper(_ticker.as_slice());
        Self::balance_of((ticker, who))
    }

    // Get the total supply of an asset `id`
    pub fn total_supply(_ticker: Vec<u8>) -> T::TokenBalance {
        let ticker = utils::bytes_to_upper(_ticker.as_slice());
        Self::token_details(ticker).total_supply
    }

    pub fn get_balance_at(_ticker: Vec<u8>, _of: T::AccountId, mut _at: u32) -> T::TokenBalance {
        let ticker = utils::bytes_to_upper(_ticker.as_slice());
        let max = Self::total_checkpoints_of(ticker.clone());

        if _at > max {
            _at = max;
        }

        if <LatestUserCheckpoint<T>>::exists((ticker.clone(), _of.clone())) {
            let latest_checkpoint = Self::latest_user_checkpoint((ticker.clone(), _of.clone()));
            if _at <= latest_checkpoint {
                while _at > 0u32 {
                    match Self::balance_at_checkpoint((ticker.clone(), _of.clone(), _at)) {
                        Some(x) => return x,
                        None => _at -= 1,
                    }
                }
            }
        }

        return Self::balance_of((ticker, _of));
    }

    fn _is_valid_transfer(
        _ticker: Vec<u8>,
        from: T::AccountId,
        to: T::AccountId,
        value: T::TokenBalance,
    ) -> Result {
        let _verification_whitelist = <general_tm::Module<T>>::verify_restriction(
            _ticker.clone(),
            from.clone(),
            to.clone(),
            value,
        )?;
        let _verification_percentage = <percentage_tm::Module<T>>::verify_restriction(
            _ticker.clone(),
            from.clone(),
            to.clone(),
            value,
        )?;
        Ok(())
        // if !verification_whitelist.0 {verification_whitelist}
        // else if !verification_percentage.0 {verification_percentage}
        // else {(true,"")}
    }

    // the ERC20 standard transfer function
    // internal
    fn _transfer(
        _ticker: Vec<u8>,
        from: T::AccountId,
        to: T::AccountId,
        value: T::TokenBalance,
    ) -> Result {
        // Granularity check
        ensure!(
            Self::check_granularity(_ticker.clone(), value),
            "Invalid granularity"
        );
        ensure!(
            <BalanceOf<T>>::exists((_ticker.clone(), from.clone())),
            "Account does not own this token"
        );
        let sender_balance = Self::balance_of((_ticker.clone(), from.clone()));
        ensure!(sender_balance >= value, "Not enough balance.");

        let updated_from_balance = sender_balance
            .checked_sub(&value)
            .ok_or("overflow in calculating balance")?;
        let receiver_balance = Self::balance_of((_ticker.clone(), to.clone()));
        let updated_to_balance = receiver_balance
            .checked_add(&value)
            .ok_or("overflow in calculating balance")?;

        Self::_update_checkpoint(_ticker.clone(), from.clone(), sender_balance)?;
        Self::_update_checkpoint(_ticker.clone(), to.clone(), receiver_balance)?;
        // reduce sender's balance
        <BalanceOf<T>>::insert((_ticker.clone(), from.clone()), updated_from_balance);

        // increase receiver's balance
        <BalanceOf<T>>::insert((_ticker.clone(), to.clone()), updated_to_balance);

        Self::deposit_event(RawEvent::Transfer(_ticker.clone(), from, to, value));
        Ok(())
    }

    fn _create_checkpoint(ticker: Vec<u8>) -> Result {
        if <TotalCheckpoints<T>>::exists(ticker.clone()) {
            let mut checkpoint_count = Self::total_checkpoints_of(ticker.clone());
            checkpoint_count = checkpoint_count
                .checked_add(1)
                .ok_or("overflow in adding checkpoint")?;
            <TotalCheckpoints<T>>::insert(ticker.clone(), checkpoint_count);
            <CheckpointTotalSupply<T>>::insert(
                (ticker.clone(), checkpoint_count),
                Self::token_details(ticker.clone()).total_supply,
            );
        } else {
            <TotalCheckpoints<T>>::insert(ticker.clone(), 1);
            <CheckpointTotalSupply<T>>::insert(
                (ticker.clone(), 1),
                Self::token_details(ticker.clone()).total_supply,
            );
        }
        Ok(())
    }

    fn _update_checkpoint(
        ticker: Vec<u8>,
        user: T::AccountId,
        user_balance: T::TokenBalance,
    ) -> Result {
        if <TotalCheckpoints<T>>::exists(ticker.clone()) {
            let checkpoint_count = Self::total_checkpoints_of(ticker.clone());
            if !<CheckpointBalance<T>>::exists((ticker.clone(), user.clone(), checkpoint_count)) {
                <CheckpointBalance<T>>::insert(
                    (ticker.clone(), user.clone(), checkpoint_count),
                    user_balance,
                );
                <LatestUserCheckpoint<T>>::insert((ticker, user), checkpoint_count);
            }
        }
        Ok(())
    }

    fn is_owner(_ticker: Vec<u8>, sender: T::AccountId) -> bool {
        let token = Self::token_details(_ticker);
        token.owner == sender
    }

    pub fn _mint(ticker: Vec<u8>, to: T::AccountId, value: T::TokenBalance) -> Result {
        ensure!(
            Self::check_granularity(ticker.clone(), value),
            "Invalid granularity"
        );
        //Increase receiver balance
        let current_to_balance = Self::balance_of((ticker.clone(), to.clone()));
        let updated_to_balance = current_to_balance
            .checked_add(&value)
            .ok_or("overflow in calculating balance")?;

        //PABLO: TODO: Add verify transfer check
        // Read the token details
        let mut token = Self::token_details(ticker.clone());
        //Increase total suply
        token.total_supply = token
            .total_supply
            .checked_add(&value)
            .ok_or("overflow in calculating balance")?;

        Self::_update_checkpoint(ticker.clone(), to.clone(), current_to_balance)?;

        <BalanceOf<T>>::insert((ticker.clone(), to.clone()), updated_to_balance);
        <Tokens<T>>::insert(ticker.clone(), token);

        Self::deposit_event(RawEvent::Minted(ticker.clone(), to, value));

        Ok(())
    }

    fn check_granularity(ticker: Vec<u8>, value: T::TokenBalance) -> bool {
        // Read the token details
        let token = Self::token_details(ticker.clone());
        // Check the granularity
        <T as utils::Trait>::as_u128(value) % token.granularity == (0 as u128)
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use chrono::{prelude::*, Duration};
    use lazy_static::lazy_static;
    use primitives::{Blake2Hasher, H256};
    use runtime_io::with_externalities;
    use runtime_primitives::{
        testing::{Digest, DigestItem, Header, UintAuthorityId},
        traits::{BlakeTwo256, IdentityLookup},
        BuildStorage,
    };
    use support::{assert_noop, assert_ok, impl_outer_origin};
    use yaml_rust::{Yaml, YamlLoader};

    use std::{
        collections::HashMap,
        fs::read_to_string,
        path::PathBuf,
        sync::{Arc, Mutex},
    };

    use crate::identity::{Investor, InvestorList};

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;

    pub struct CurrencyToBalanceHandler;

    impl Convert<u128, u128> for CurrencyToBalanceHandler {
        fn convert(x: u128) -> u128 {
            x
        }
    }

    impl system::Trait for Test {
        type Origin = Origin;
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type Digest = Digest;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = ();
        type Log = DigestItem;
    }
    impl balances::Trait for Test {
        type Balance = u128;
        type DustRemoval = ();
        type Event = ();
        type OnFreeBalanceZero = ();
        type OnNewAccount = ();
        type TransactionPayment = ();
        type TransferPayment = ();
    }
    impl general_tm::Trait for Test {
        type Event = ();
        type Asset = Module<Test>;
    }
    impl identity::Trait for Test {
        type Event = ();
    }
    impl percentage_tm::Trait for Test {
        type Event = ();
        type Asset = Module<Test>;
    }
    impl timestamp::Trait for Test {
        type Moment = u64;
        type OnTimestampSet = ();
    }
    impl utils::Trait for Test {
        type TokenBalance = u128;
    }
    impl consensus::Trait for Test {
        type SessionKey = UintAuthorityId;
        type InherentOfflineReport = ();
        type Log = DigestItem;
    }
    impl session::Trait for Test {
        type ConvertAccountIdToSessionKey = ();
        type OnSessionChange = ();
        type Event = ();
    }
    impl Trait for Test {
        type Event = ();
        type Currency = balances::Module<Test>;
        type CurrencyToBalance = CurrencyToBalanceHandler;
    }
    type Asset = Module<Test>;

    type Balances = balances::Module<Test>;

    lazy_static! {
        static ref INVESTOR_MAP: Arc<
            Mutex<
                HashMap<
                    <Test as system::Trait>::AccountId,
                    Investor<<Test as system::Trait>::AccountId>,
                >,
            >,
        > = Arc::new(Mutex::new(HashMap::new()));
        static ref INVESTOR_MAP_OUTER_LOCK: Arc<Mutex<()>> = Arc::new(Mutex::new(()));
    }

    /// Build a genesis identity instance owned by account No. 1
    fn identity_owned_by_1() -> runtime_io::TestExternalities<Blake2Hasher> {
        let mut t = system::GenesisConfig::<Test>::default()
            .build_storage()
            .unwrap()
            .0;
        t.extend(
            identity::GenesisConfig::<Test> { owner: 1 }
                .build_storage()
                .unwrap()
                .0,
        );
        t.into()
    }

    /// Build a genesis identity instance owned by the specified account
    fn identity_owned_by(id: u64) -> runtime_io::TestExternalities<Blake2Hasher> {
        let mut t = system::GenesisConfig::<Test>::default()
            .build_storage()
            .unwrap()
            .0;
        t.extend(
            identity::GenesisConfig::<Test> { owner: id }
                .build_storage()
                .unwrap()
                .0,
        );
        t.into()
    }

    #[test]
    fn issuers_can_create_tokens() {
        with_externalities(&mut identity_owned_by_1(), || {
            // Expected token entry
            let token = SecurityToken {
                name: vec![0x01],
                owner: 1,
                total_supply: 1_000_000,
            };

            // Raise the owner's base currency balance
            Balances::make_free_balance_be(&token.owner, 1_000_000);

            identity::Module::<Test>::do_create_issuer(token.owner)
                .expect("Could not make token.owner an issuer");

            // Issuance is successful
            assert_ok!(Asset::issue_token(
                Origin::signed(token.owner),
                token.name.clone(),
                token.name.clone(),
                token.total_supply
            ));

            // A correct entry is added
            assert_eq!(Asset::token_details(token.name.clone()), token);
        });
    }

    #[test]
    fn non_issuers_cant_create_tokens() {
        with_externalities(&mut identity_owned_by_1(), || {
            // Expected token entry
            let token = SecurityToken {
                name: vec![0x01],
                owner: 1,
                total_supply: 1_000_000,
            };

            Balances::make_free_balance_be(&token.owner, 1_000_000);

            Balances::make_free_balance_be(&token.owner, 1_000_000);

            // Issuance is unsuccessful
            assert_noop!(
                Asset::issue_token(
                    Origin::signed(token.owner + 1),
                    token.name.clone(),
                    token.name.clone(),
                    token.total_supply
                ),
                "user is not authorized"
            );

            // Entry is not added
            assert_ne!(Asset::token_details(token.name.clone()), token);
        });
    }

    #[test]
    /// This test loads up a YAML of testcases and checks each of them
    fn transfer_scenarios_external() {
        let mut yaml_path_buf = PathBuf::new();
        yaml_path_buf.push(env!("CARGO_MANIFEST_DIR")); // This package's root
        yaml_path_buf.push("tests/asset_transfers.yml");

        println!("Loading YAML from {:?}", yaml_path_buf);

        let yaml_string = read_to_string(yaml_path_buf.as_path())
            .expect("Could not load the YAML file to a string");

        // Parse the YAML
        let yaml = YamlLoader::load_from_str(&yaml_string).expect("Could not parse the YAML file");

        let yaml = &yaml[0];

        let now = Utc::now();

        for case in yaml["test_cases"]
            .as_vec()
            .expect("Could not reach test_cases")
        {
            println!("Case: {:#?}", case);

            let accounts = case["named_accounts"]
                .as_hash()
                .expect("Could not view named_accounts as a hashmap");

            let mut externalities = if let Some(identity_owner) =
                accounts.get(&Yaml::String("identity-owner".to_owned()))
            {
                identity_owned_by(
                    identity_owner["id"]
                        .as_i64()
                        .expect("Could not get identity owner's ID") as u64,
                )
            } else {
                system::GenesisConfig::<Test>::default()
                    .build_storage()
                    .unwrap()
                    .0
                    .into()
            };

            with_externalities(&mut externalities, || {
                // Instantiate accounts
                for (name, account) in accounts {
                    <timestamp::Module<Test>>::set_timestamp(now.timestamp() as u64);
                    let name = name
                        .as_str()
                        .expect("Could not take named_accounts key as string");
                    let id = account["id"].as_i64().expect("id is not a number") as u64;
                    let balance = account["balance"]
                        .as_i64()
                        .expect("balance is not a number");

                    println!("Preparing account {}", name);

                    Balances::make_free_balance_be(&id, balance.clone() as u128);
                    println!("{}: gets {} initial balance", name, balance);
                    if account["issuer"]
                        .as_bool()
                        .expect("Could not check if account is an issuer")
                    {
                        assert_ok!(identity::Module::<Test>::do_create_issuer(id));
                        println!("{}: becomes issuer", name);
                    }
                    if account["investor"]
                        .as_bool()
                        .expect("Could not check if account is an investor")
                    {
                        assert_ok!(identity::Module::<Test>::do_create_investor(id));
                        println!("{}: becomes investor", name);
                    }
                }

                // Issue tokens
                let tokens = case["tokens"]
                    .as_hash()
                    .expect("Could not view tokens as a hashmap");

                for (ticker, token) in tokens {
                    let ticker = ticker.as_str().expect("Can't parse ticker as string");
                    println!("Preparing token {}:", ticker);

                    let owner = token["owner"]
                        .as_str()
                        .expect("Can't parse owner as string");

                    let owner_id = accounts
                        .get(&Yaml::String(owner.to_owned()))
                        .expect("Can't get owner record")["id"]
                        .as_i64()
                        .expect("Can't parse owner id as i64")
                        as u64;
                    let total_supply = token["total_supply"]
                        .as_i64()
                        .expect("Can't parse the total supply as i64")
                        as u128;

                    let token_struct = SecurityToken {
                        name: ticker.to_owned().into_bytes(),
                        owner: owner_id,
                        total_supply,
                    };
                    println!("{:#?}", token_struct);

                    // Check that issuing succeeds/fails as expected
                    if token["issuance_succeeds"]
                        .as_bool()
                        .expect("Could not check if issuance should succeed")
                    {
                        assert_ok!(Asset::issue_token(
                            Origin::signed(token_struct.owner),
                            token_struct.name.clone(),
                            token_struct.name.clone(),
                            token_struct.total_supply,
                        ));

                        // Also check that the new token matches what we asked to create
                        assert_eq!(
                            Asset::token_details(token_struct.name.clone()),
                            token_struct
                        );

                        // Check that the issuer's balance corresponds to total supply
                        assert_eq!(
                            Asset::balance_of((token_struct.name, token_struct.owner)),
                            token_struct.total_supply
                        );

                        // Add specified whitelist entries
                        let whitelists = token["whitelist_entries"]
                            .as_vec()
                            .expect("Could not view token whitelist entries as vec");

                        for wl_entry in whitelists {
                            let investor = wl_entry["investor"]
                                .as_str()
                                .expect("Can't parse investor as string");
                            let investor_id = accounts
                                .get(&Yaml::String(investor.to_owned()))
                                .expect("Can't get investor account record")["id"]
                                .as_i64()
                                .expect("Can't parse investor id as i64")
                                as u64;

                            let expiry = wl_entry["expiry"]
                                .as_i64()
                                .expect("Can't parse expiry as i64");

                            let wl_id = wl_entry["whitelist_id"]
                                .as_i64()
                                .expect("Could not parse whitelist_id as i64")
                                as u32;

                            println!(
                                "Token {}: processing whitelist entry for {}",
                                ticker, investor
                            );

                            general_tm::Module::<Test>::add_to_whitelist(
                                Origin::signed(owner_id),
                                ticker.to_owned().into_bytes(),
                                wl_id,
                                investor_id,
                                (now + Duration::hours(expiry)).timestamp() as u64,
                            )
                            .expect("Could not create whitelist entry");
                        }
                    } else {
                        assert!(Asset::issue_token(
                            Origin::signed(token_struct.owner),
                            token_struct.name.clone(),
                            token_struct.name.clone(),
                            token_struct.total_supply,
                        )
                        .is_err());
                    }
                }

                // Set up allowances
                let allowances = case["allowances"]
                    .as_vec()
                    .expect("Could not view allowances as a vec");

                for allowance in allowances {
                    let sender = allowance["sender"]
                        .as_str()
                        .expect("Could not view sender as str");
                    let sender_id = case["named_accounts"][sender]["id"]
                        .as_i64()
                        .expect("Could not view sender id as i64")
                        as u64;
                    let spender = allowance["spender"]
                        .as_str()
                        .expect("Could not view spender as str");
                    let spender_id = case["named_accounts"][spender]["id"]
                        .as_i64()
                        .expect("Could not view sender id as i64")
                        as u64;
                    let amount = allowance["amount"]
                        .as_i64()
                        .expect("Could not view amount as i64")
                        as u128;
                    let ticker = allowance["ticker"]
                        .as_str()
                        .expect("Could not view ticker as str");
                    let succeeds = allowance["succeeds"]
                        .as_bool()
                        .expect("Could not determine if allowance should succeed");

                    if succeeds {
                        assert_ok!(Asset::approve(
                            Origin::signed(sender_id),
                            ticker.to_owned().into_bytes(),
                            spender_id,
                            amount,
                        ));
                    } else {
                        assert!(Asset::approve(
                            Origin::signed(sender_id),
                            ticker.to_owned().into_bytes(),
                            spender_id,
                            amount,
                        )
                        .is_err())
                    }
                }

                println!("Transfers:");
                // Perform regular transfers
                let transfers = case["transfers"]
                    .as_vec()
                    .expect("Could not view transfers as vec");
                for transfer in transfers {
                    let from = transfer["from"]
                        .as_str()
                        .expect("Could not view from as str");
                    let from_id = case["named_accounts"][from]["id"]
                        .as_i64()
                        .expect("Could not view from_id as i64")
                        as u64;
                    let to = transfer["to"].as_str().expect("Could not view to as str");
                    let to_id = case["named_accounts"][to]["id"]
                        .as_i64()
                        .expect("Could not view to_id as i64")
                        as u64;
                    let amount = transfer["amount"]
                        .as_i64()
                        .expect("Could not view amount as i64")
                        as u128;
                    let ticker = transfer["ticker"]
                        .as_str()
                        .expect("Coule not view ticker as str")
                        .to_owned();
                    let succeeds = transfer["succeeds"]
                        .as_bool()
                        .expect("Could not view succeeds as bool");

                    println!("{} of token {} from {} to {}", amount, ticker, from, to);
                    let ticker = ticker.into_bytes();

                    // Get sender's investor data
                    let investor_data = <InvestorList<Test>>::get(from_id);

                    println!("{}'s investor data: {:#?}", from, investor_data);

                    if succeeds {
                        assert_ok!(Asset::transfer(
                            Origin::signed(from_id),
                            ticker,
                            to_id,
                            amount
                        ));
                    } else {
                        assert!(
                            Asset::transfer(Origin::signed(from_id), ticker, to_id, amount)
                                .is_err()
                        );
                    }
                }

                println!("Approval-basedt transfers:");
                // Perform regular transfers
                let transfer_froms = case["transfer_froms"]
                    .as_vec()
                    .expect("Could not view transfer_froms as vec");
                for transfer_from in transfer_froms {
                    let from = transfer_from["from"]
                        .as_str()
                        .expect("Could not view from as str");
                    let from_id = case["named_accounts"][from]["id"]
                        .as_i64()
                        .expect("Could not view from_id as i64")
                        as u64;
                    let spender = transfer_from["spender"]
                        .as_str()
                        .expect("Could not view spender as str");
                    let spender_id = case["named_accounts"][spender]["id"]
                        .as_i64()
                        .expect("Could not view spender_id as i64")
                        as u64;
                    let to = transfer_from["to"]
                        .as_str()
                        .expect("Could not view to as str");
                    let to_id = case["named_accounts"][to]["id"]
                        .as_i64()
                        .expect("Could not view to_id as i64")
                        as u64;
                    let amount = transfer_from["amount"]
                        .as_i64()
                        .expect("Could not view amount as i64")
                        as u128;
                    let ticker = transfer_from["ticker"]
                        .as_str()
                        .expect("Coule not view ticker as str")
                        .to_owned();
                    let succeeds = transfer_from["succeeds"]
                        .as_bool()
                        .expect("Could not view succeeds as bool");

                    println!(
                        "{} of token {} from {} to {} spent by {}",
                        amount, ticker, from, to, spender
                    );
                    let ticker = ticker.into_bytes();

                    // Get sender's investor data
                    let investor_data = <InvestorList<Test>>::get(spender_id);

                    println!("{}'s investor data: {:#?}", from, investor_data);

                    if succeeds {
                        assert_ok!(Asset::transfer_from(
                            Origin::signed(spender_id),
                            ticker,
                            from_id,
                            to_id,
                            amount
                        ));
                    } else {
                        assert!(Asset::transfer_from(
                            Origin::signed(from_id),
                            ticker,
                            from_id,
                            to_id,
                            amount
                        )
                        .is_err());
                    }
                }
            });
        }
    }
}
