use crate::balances;
use codec::Codec;
use rstd::prelude::*;
use session;
use sr_primitives::traits::{Member, SimpleArithmetic};
use srml_support::Parameter;
use system;

/// The module's configuration trait.
pub trait Trait: system::Trait + balances::Trait + session::Trait {
    type TokenBalance: Parameter + Member + SimpleArithmetic + Codec + Default + Copy;
    fn as_u128(v: Self::TokenBalance) -> u128;
    fn as_tb(v: u128) -> Self::TokenBalance;
    fn token_balance_to_balance(v: Self::TokenBalance) -> <Self as balances::Trait>::Balance;
    fn balance_to_token_balance(v: <Self as balances::Trait>::Balance) -> Self::TokenBalance;
    fn validator_id_to_account_id(v: <Self as session::Trait>::ValidatorId) -> Self::AccountId;
}

// Other utility functions
#[inline]
/// Convert all letter characters of a slice to their upper case counterparts.
/// # TODO
/// This functions is always called on `ticket`, maybe we could create a type for `ticket` to
/// ensure that type is UPPER case, and **avoid vector clone** (using `collect`).
pub fn bytes_to_upper(v: &[u8]) -> Vec<u8> {
    v.iter()
        .map(|chr| match chr {
            97..=122 => chr - 32,
            other => *other,
        })
        .collect()
}

/*
use rstd::borrow::Borrow;
use srml_support::{ Parameter, StorageMap};
pub trait StorageMapExt<K: Codec, V: Codec> {
    type Query;

    fn get_option<KeyArg: Borrow<K>>( key: KeyArg) -> Option<Self::Query>;
}

impl<K,V> StorageMapExt<K,V> for StorageMap<K,V> {
    type Query = StorageMap<K,V>::Query;

    fn get_option<KeyArg: Borrow<K>>( key: KeyArg) -> Option<Self::Query>{
        if Self::exists(key) {
            Some( Self::get(key))
        } else {
            None
        }
    }
}*/
