use crate::general_tm;
use crate::identity;
use crate::percentage_tm;
use crate::utils;
use rstd::prelude::*;
//use parity_codec::Codec;
use runtime_primitives::traits::{As, CheckedAdd, CheckedSub};
use support::traits::{Currency, ExistenceRequirement, Imbalance, OnUnbalanced, WithdrawReason};
use support::{
    decl_event, decl_module, decl_storage, dispatch::Result, ensure, StorageMap, StorageValue,
};
use system::{self, ensure_signed};

type FeeOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;
type NegativeImbalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;

/// The module's configuration trait.
pub trait Trait:
    system::Trait
    + general_tm::Trait
    + percentage_tm::Trait
    + utils::Trait
    + balances::Trait
    + identity::Trait
{
    // TODO: Add other types and constants required configure this module.

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    //type TokenBalance: Parameter + Member + SimpleArithmetic + Codec + Default + Copy + As<usize> + As<u64>;
    type Currency: Currency<Self::AccountId>;
    // Handler for the unbalanced decrease when charging fee
    type TokenFeeCharge: OnUnbalanced<NegativeImbalanceOf<Self>>;
}

// struct to store the token details
#[derive(parity_codec::Encode, parity_codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct Erc20Token<U, V> {
    name: Vec<u8>,
    ticker: Vec<u8>,
    total_supply: U,
    pub owner: V,
}

decl_storage! {
    trait Store for Module<T: Trait> as Asset {

        FeeCollector get(fee_collector) config(): T::AccountId;
        // token id nonce for storing the next token id available for token initialization
        // inspired by the AssetId in the SRML assets module
        TokenId get(token_id): u32;
        // details of the token corresponding to a token id
        Tokens get(token_details): map u32 => Erc20Token<T::TokenBalance, T::AccountId>;
        // balances mapping for an account and token
        BalanceOf get(balance_of): map (u32, T::AccountId) => T::TokenBalance;
        // allowance for an account and token
        Allowance get(allowance): map (u32, T::AccountId, T::AccountId) => T::TokenBalance;
        // cost in base currency to create a token
        AssetCreationFee get(asset_creation_fee) config(): FeeOf<T>;
        // Checkpoints created per token
        TotalCheckpoints get(total_checkpoints_of): map (u32) => u32;
        // Total supply of the token at the checkpoint
        CheckpointTotalSupply get(total_supply_at): map (u32, u32) => T::TokenBalance;
        // Balance of a user at a checkpoint
        CheckpointBalance get(balance_at_checkpoint): map (u32, T::AccountId, u32) => T::TokenBalance;
    }
}

// public interface for this runtime module
decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {
      // initialize the default event for this module
      fn deposit_event<T>() = default;

      // initializes a new token
      // generates an integer token_id so that all tokens are unique
      // takes a name, ticker, total supply for the token
      // makes the initiating account the owner of the token
      // the balance of the owner is set to total supply
      fn issue_token(origin, name: Vec<u8>, ticker: Vec<u8>, total_supply: T::TokenBalance) -> Result {
          let sender = ensure_signed(origin)?;
          ensure!(<identity::Module<T>>::is_issuer(sender.clone()),"user is not authorized");

          // Fee is burnt (could override the on_unbalanced function to instead distribute to stakers / validators)
          let imbalance = T::Currency::withdraw(&sender, Self::asset_creation_fee(), WithdrawReason::Fee, ExistenceRequirement::KeepAlive)?;

          // Alternative way to take a fee - fee is paid to `fee_collector`
          let my_fee = <T::Balance as As<u64>>::sa(1337);
          <balances::Module<T> as Currency<_>>::transfer(&sender, &Self::fee_collector(), my_fee)?;
          T::TokenFeeCharge::on_unbalanced(imbalance);

          // checking max size for name and ticker
          // byte arrays (vecs) with no max size should be avoided
          ensure!(name.len() <= 64, "token name cannot exceed 64 bytes");
          ensure!(ticker.len() <= 32, "token ticker cannot exceed 32 bytes");

          // take fee for creating asset
          let token_id = Self::token_id();
          let next_token_id = token_id.checked_add(1).ok_or("overflow in calculating next token id")?;
          <TokenId<T>>::put(next_token_id);

          let token = Erc20Token {
              name,
              ticker,
              total_supply,
              owner:sender.clone()
          };

          <Tokens<T>>::insert(token_id, token);
          <BalanceOf<T>>::insert((token_id, sender), total_supply);

          runtime_io::print("Initialized!!!");

          Ok(())
      }

      // transfer tokens from one account to another
      // origin is assumed as sender
      fn transfer(_origin, token_id: u32, to: T::AccountId, value: T::TokenBalance) -> Result {
          let sender = ensure_signed(_origin)?;
          //Self::_isValidTransfer(token_id, sender.clone(), to.clone(), value);
          Self::_transfer(token_id, sender, to, value)
      }

      // approve token transfer from one account to another
      // once this is done, transfer_from can be called with corresponding values
      fn approve(_origin, token_id: u32, spender: T::AccountId, value: T::TokenBalance) -> Result {
          let sender = ensure_signed(_origin)?;
          ensure!(<BalanceOf<T>>::exists((token_id, sender.clone())), "Account does not own this token");

          let allowance = Self::allowance((token_id, sender.clone(), spender.clone()));
          let updated_allowance = allowance.checked_add(&value).ok_or("overflow in calculating allowance")?;
          <Allowance<T>>::insert((token_id, sender.clone(), spender.clone()), updated_allowance);

          Self::deposit_event(RawEvent::Approval(token_id, sender.clone(), spender.clone(), value));

          Ok(())
      }

      // the ERC20 standard transfer_from function
      // implemented in the open-zeppelin way - increase/decrease allownace
      // if approved, transfer from an account to another account without owner's signature
      pub fn transfer_from(_origin, token_id: u32, from: T::AccountId, to: T::AccountId, value: T::TokenBalance) -> Result {
        ensure!(<Allowance<T>>::exists((token_id, from.clone(), to.clone())), "Allowance does not exist.");
        let allowance = Self::allowance((token_id, from.clone(), to.clone()));
        ensure!(allowance >= value, "Not enough allowance.");
          
        // using checked_sub (safe math) to avoid overflow
        let updated_allowance = allowance.checked_sub(&value).ok_or("overflow in calculating allowance")?;
        <Allowance<T>>::insert((token_id, from.clone(), to.clone()), updated_allowance);

        Self::deposit_event(RawEvent::Approval(token_id, from.clone(), to.clone(), value));
        Self::_transfer(token_id, from, to, value)
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
        // tokenid, from, to, value
        Transfer(u32, AccountId, AccountId, Balance),
        // event when an approval is made
        // tokenid, owner, spender, value
        Approval(u32, AccountId, AccountId, Balance),
        // event - used for testing in the absence of custom getters
        // tokenid, owner, checkpoint, balance
        BalanceAt(u32, AccountId, u32, Balance),
    }
);

pub trait HasOwner<T> {
    fn is_owner(
        token_id: u32,
        who: T,
    ) -> bool;
}

impl<T: Trait> HasOwner<T::AccountId> for Module<T> {
    fn is_owner(
        token_id: u32,
        who: T::AccountId,
    ) -> bool {
        let erc20 = Self::token_details(token_id);
        erc20.owner == who
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
	pub fn balance(token_id: u32, who: T::AccountId) -> T::TokenBalance {
        Self::balance_of((token_id, who))
	}

	// Get the total supply of an asset `id`
	pub fn total_supply(token_id: u32) -> T::TokenBalance {
        Self::token_details(token_id).total_supply
	}

    pub fn get_balance_at(token_id: u32, _of: T::AccountId, mut _at: u32) -> T::TokenBalance {
        let max = Self::total_checkpoints_of(token_id);

        if _at > max {
            _at = max;
        }

        while _at > 0u32 {
            if <CheckpointBalance<T>>::exists((token_id, _of.clone(), _at)) {
                return Self::balance_at_checkpoint((token_id, _of.clone(), _at));
            }
            _at -= 1;
        }

        return Self::balance_of((token_id, _of.clone()));
    }


    fn _is_valid_transfer(
        token_id: u32,
        from: T::AccountId,
        to: T::AccountId,
        value: T::TokenBalance,
    ) -> Result {
        let verification_whitelist = <general_tm::Module<T>>::verify_restriction(token_id, from.clone(), to.clone(), value)?;
        let verification_percentage = <percentage_tm::Module<T>>::verify_restriction(token_id, from.clone(), to.clone(), value)?;
        Ok(())
        // if !verification_whitelist.0 {verification_whitelist}
        // else if !verification_percentage.0 {verification_percentage}
        // else {(true,"")}
    }

    // the ERC20 standard transfer function
    // internal
    fn _transfer(
        token_id: u32,
        from: T::AccountId,
        to: T::AccountId,
        value: T::TokenBalance,
    ) -> Result {
        ensure!(<BalanceOf<T>>::exists((token_id, from.clone())), "Account does not own this token");
        let sender_balance = Self::balance_of((token_id, from.clone()));
        ensure!(sender_balance >= value, "Not enough balance.");

        let updated_from_balance = sender_balance
            .checked_sub(&value)
            .ok_or("overflow in calculating balance")?;
        let receiver_balance = Self::balance_of((token_id, to.clone()));
        let updated_to_balance = receiver_balance
            .checked_add(&value)
            .ok_or("overflow in calculating balance")?;

        Self::_update_checkpoint(token_id, from.clone(), sender_balance)?;
        Self::_update_checkpoint(token_id, to.clone(), receiver_balance)?;
        // reduce sender's balance
        <BalanceOf<T>>::insert((token_id, from.clone()), updated_from_balance);

        // increase receiver's balance
        <BalanceOf<T>>::insert((token_id, to.clone()), updated_to_balance);

        Self::deposit_event(RawEvent::Transfer(token_id, from, to, value));
        Ok(())
    }

    fn _create_checkpoint(token_id: u32) -> Result {
        if <TotalCheckpoints<T>>::exists(token_id) {
            let mut checkpoint_count = Self::total_checkpoints_of(token_id);
            checkpoint_count = checkpoint_count
                .checked_add(1)
                .ok_or("overflow in adding checkpoint")?;
            <TotalCheckpoints<T>>::insert(token_id, checkpoint_count);
            <CheckpointTotalSupply<T>>::insert((token_id, checkpoint_count), Self::token_details(token_id).total_supply);
        } else {
            <TotalCheckpoints<T>>::insert(token_id, 1);
            <CheckpointTotalSupply<T>>::insert((token_id, 1), Self::token_details(token_id).total_supply);
        }
        Ok(())
    }

    fn _update_checkpoint(
        token_id: u32,
        user: T::AccountId,
        user_balance: T::TokenBalance,
    ) -> Result {
        if <TotalCheckpoints<T>>::exists(token_id) {
            let checkpoint_count = Self::total_checkpoints_of(token_id);
            if !<CheckpointBalance<T>>::exists((token_id, user.clone(), checkpoint_count)) {
                <CheckpointBalance<T>>::insert((token_id, user, checkpoint_count), user_balance);
            }
        }
        Ok(())
    }

    fn is_owner(token_id:u32, sender: T::AccountId) -> bool {
        let token = Self::token_details(token_id);
        token.owner == sender
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use primitives::{Blake2Hasher, H256};
    use runtime_io::with_externalities;
    use runtime_primitives::{
        testing::{Digest, DigestItem, Header},
        traits::{BlakeTwo256, IdentityLookup},
        BuildStorage,
    };
    use support::{assert_ok, impl_outer_origin};

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
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
    impl Trait for Test {
        type Event = ();
    }
    type asset = Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
        system::GenesisConfig::<Test>::default()
            .build_storage()
            .unwrap()
            .0
            .into()
    }

    //#[test]
    // fn it_works_for_default_value() {
    // 	with_externalities(&mut new_test_ext(), || {
    // 		// Just a dummy test for the dummy funtion `do_something`
    // 		// calling the `do_something` function with a value 42
    // 		assert_ok!(asset::do_something(Origin::signed(1), 42));
    // 		// asserting that the stored value is equal to what we stored
    // 		assert_eq!(asset::something(), Some(42));
    // 	});
    // }
}
