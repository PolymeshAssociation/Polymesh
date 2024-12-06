#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};

use codec::{Decode, DecodeLimit, Encode};
use frame_support::dispatch::{DispatchError, Dispatchable, GetDispatchInfo};
use frame_support::ensure;
use frame_support::log::trace;
use frame_support::storage::unhashed;
use frame_support::traits::{Get, GetCallMetadata};
use frame_system::RawOrigin;
use scale_info::prelude::format;
use scale_info::prelude::string::String;
use scale_info::TypeInfo;
use sp_core::crypto::UncheckedFrom;

use pallet_contracts::chain_extension as ce;
use pallet_contracts::Config as BConfig;
use pallet_permissions::with_call_metadata;
use polymesh_common_utilities::Context;

use super::*;

type Identity<T> = pallet_identity::Module<T>;

/// Maximum decoding depth.
const MAX_DECODE_DEPTH: u32 = 10;

/// ExtrinsicId
#[derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExtrinsicId(u8, u8);

impl From<ExtrinsicId> for [u8; 2] {
    fn from(ExtrinsicId(pallet_id, extrinsic_id): ExtrinsicId) -> Self {
        [pallet_id, extrinsic_id]
    }
}

impl From<[u8; 2]> for ExtrinsicId {
    fn from(ext_id: [u8; 2]) -> Self {
        Self(ext_id[0], ext_id[1])
    }
}

impl ExtrinsicId {
    fn try_from(input: &[u8]) -> Option<Self> {
        if input.len() >= 2 {
            Some(Self(input[0], input[1]))
        } else {
            None
        }
    }
}

/// KeyHasher
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyHasher {
    Twox,
}

/// HashSize
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HashSize {
    B64,
    B128,
    B256,
}

/// Polymesh ChainExtension callable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FuncId {
    /// No operation -- Used for benchmarking the ChainExtension.
    #[cfg(feature = "runtime-benchmarks")]
    NOP,
    CallRuntime,
    ReadStorage,
    GetSpecVersion,
    GetTransactionVersion,
    GetKeyDid,
    KeyHasher(KeyHasher, HashSize),
    GetLatestApiUpgrade,
    CallRuntimeWithError,
    GetNextAssetId,
}

impl FuncId {
    fn try_from(id: u32) -> Option<Self> {
        let ext_id = (id >> 16) as u16;
        let func_id = (id & 0x0000FFFF) as u16;
        match ext_id {
            0x00 => match func_id {
                #[cfg(feature = "runtime-benchmarks")]
                0x00 => Some(Self::NOP),
                0x01 => Some(Self::CallRuntime),
                0x02 => Some(Self::ReadStorage),
                0x03 => Some(Self::GetSpecVersion),
                0x04 => Some(Self::GetTransactionVersion),
                0x05 => Some(Self::GetKeyDid),
                0x10 => Some(Self::KeyHasher(KeyHasher::Twox, HashSize::B64)),
                0x11 => Some(Self::KeyHasher(KeyHasher::Twox, HashSize::B128)),
                0x12 => Some(Self::KeyHasher(KeyHasher::Twox, HashSize::B256)),
                0x13 => Some(Self::GetLatestApiUpgrade),
                0x14 => Some(Self::CallRuntimeWithError),
                0x15 => Some(Self::GetNextAssetId),
                _ => None,
            },
            _ => None,
        }
    }
}

impl Into<u32> for FuncId {
    fn into(self) -> u32 {
        let (ext_id, func_id) = match self {
            #[cfg(feature = "runtime-benchmarks")]
            Self::NOP => (0x0000, 0x0000),
            Self::CallRuntime => (0x0000, 0x01),
            Self::ReadStorage => (0x0000, 0x02),
            Self::GetSpecVersion => (0x0000, 0x03),
            Self::GetTransactionVersion => (0x0000, 0x04),
            Self::GetKeyDid => (0x0000, 0x05),
            Self::KeyHasher(KeyHasher::Twox, HashSize::B64) => (0x0000, 0x10),
            Self::KeyHasher(KeyHasher::Twox, HashSize::B128) => (0x0000, 0x11),
            Self::KeyHasher(KeyHasher::Twox, HashSize::B256) => (0x0000, 0x12),
            Self::GetLatestApiUpgrade => (0x0000, 0x13),
            Self::CallRuntimeWithError => (0x0000, 0x14),
            Self::GetNextAssetId => (0x0000, 0x15),
        };
        (ext_id << 16) + func_id
    }
}

impl Into<i32> for FuncId {
    fn into(self) -> i32 {
        let id: u32 = self.into();
        id as i32
    }
}

/// Run `with` while the current Payer is temporarily set to the given one.
fn with_payer<T: Config, W: FnOnce() -> R, R>(payer: T::AccountId, with: W) -> R {
    let old_payer = Context::current_payer::<Identity<T>>();
    Context::set_current_payer::<Identity<T>>(Some(payer));
    let result = with();
    Context::set_current_payer::<Identity<T>>(old_payer);
    result
}

fn read_storage<T, E>(env: ce::Environment<E, ce::InitState>) -> ce::Result<ce::RetVal>
where
    T: Config,
    T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
    E: ce::Ext<T = T>,
{
    let mut env = env.buf_in_buf_out();
    let key_len = env.in_len();

    // Limit `key_len` to a maximum.
    ensure!(
        key_len <= <T as Config>::MaxInLen::get(),
        Error::<T>::InLenTooLarge
    );

    // Charge weight based on storage value length `MaxOutLen`.
    let max_len = T::MaxOutLen::get() as u32;
    let charged_amount =
        env.charge_weight(<T as Config>::WeightInfo::read_storage(key_len, max_len))?;

    let key = env.read(key_len)?;
    trace!(
        target: "runtime",
        "PolymeshExtension contract ReadStorage: key={:x?}",
        key
    );
    let value = unhashed::get_raw(key.as_slice());
    let value_len = value.as_ref().map(|v| v.len() as u32).unwrap_or_default();
    trace!(
        target: "runtime",
        "PolymeshExtension contract ReadStorage: value length={:?}",
        value_len
    );

    // Limit `value_len` to a maximum.
    ensure!(
        value_len <= <T as Config>::MaxOutLen::get(),
        Error::<T>::OutLenTooLarge
    );

    // Adjust charged weight based on the actual value length.
    if value_len < max_len {
        env.adjust_weight(
            charged_amount,
            <T as Config>::WeightInfo::read_storage(key_len, value_len),
        );
    }

    trace!(
        target: "runtime",
        "PolymeshExtension contract ReadStorage: value={:x?}",
        value
    );
    let encoded = value.encode();
    env.write(&encoded, false, None).map_err(|err| {
        trace!(
            target: "runtime",
            "PolymeshExtension failed to write storage value into contract memory:{:?}",
            err
        );
        Error::<T>::ReadStorageFailed
    })?;

    Ok(ce::RetVal::Converging(0))
}

fn get_version<T, E>(
    env: ce::Environment<E, ce::InitState>,
    get_spec: bool,
) -> ce::Result<ce::RetVal>
where
    T: Config,
    T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
    E: ce::Ext<T = T>,
{
    let mut env = env.prim_in_buf_out();

    // Charge weight.
    env.charge_weight(<T as Config>::WeightInfo::get_version())?;

    let runtime_version = <T as frame_system::Config>::Version::get();
    let version = if get_spec {
        runtime_version.spec_version
    } else {
        runtime_version.transaction_version
    }
    .encode();
    env.write(&version, false, None).map_err(|err| {
        trace!(
            target: "runtime",
            "PolymeshExtension failed to write value into contract memory:{:?}",
            err
        );
        Error::<T>::ReadStorageFailed
    })?;

    Ok(ce::RetVal::Converging(0))
}

fn get_key_did<T, E>(env: ce::Environment<E, ce::InitState>) -> ce::Result<ce::RetVal>
where
    T: Config,
    T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
    E: ce::Ext<T = T>,
{
    let mut env = env.buf_in_buf_out();

    // Charge weight.
    env.charge_weight(<T as Config>::WeightInfo::get_key_did())?;

    let key: T::AccountId = env.read_as()?;
    trace!(
        target: "runtime",
        "PolymeshExtension contract GetKeyDid: key={key:?}",
    );
    let did = Identity::<T>::get_identity(&key);
    trace!(
        target: "runtime",
        "PolymeshExtension contract GetKeyDid: did={did:?}",
    );
    let encoded = did.encode();
    env.write(&encoded, false, None).map_err(|err| {
        trace!(
            target: "runtime",
            "PolymeshExtension failed to write identity value into contract memory:{:?}",
            err
        );
        Error::<T>::ReadStorageFailed
    })?;

    Ok(ce::RetVal::Converging(0))
}

fn key_hasher<T, E>(
    env: ce::Environment<E, ce::InitState>,
    hasher: KeyHasher,
    size: HashSize,
) -> ce::Result<ce::RetVal>
where
    T: Config,
    T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
    E: ce::Ext<T = T>,
{
    use sp_io::hashing;
    let mut env = env.buf_in_buf_out();
    let in_len = env.in_len();

    // Charge weight as a linear function of `in_len`.
    let weight = match size {
        HashSize::B64 => <T as Config>::WeightInfo::hash_twox_64(in_len),
        HashSize::B128 => <T as Config>::WeightInfo::hash_twox_64(in_len),
        HashSize::B256 => <T as Config>::WeightInfo::hash_twox_64(in_len),
    };
    env.charge_weight(weight)?;

    let data = env.read(in_len)?;
    let hash = match (hasher, size) {
        (KeyHasher::Twox, HashSize::B64) => hashing::twox_64(data.as_slice()).encode(),
        (KeyHasher::Twox, HashSize::B128) => hashing::twox_128(data.as_slice()).encode(),
        (KeyHasher::Twox, HashSize::B256) => hashing::twox_256(data.as_slice()).encode(),
    };
    trace!(
        target: "runtime",
        "PolymeshExtension contract KeyHasher: hash={hash:x?}",
    );
    env.write(&hash, false, None).map_err(|err| {
        trace!(
            target: "runtime",
            "PolymeshExtension failed to write hash into contract memory:{:?}",
            err
        );
        Error::<T>::ReadStorageFailed
    })?;

    Ok(ce::RetVal::Converging(0))
}

fn get_latest_api_upgrade<T, E>(env: ce::Environment<E, ce::InitState>) -> ce::Result<ce::RetVal>
where
    T: Config,
    T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
    E: ce::Ext<T = T>,
{
    let mut env = env.buf_in_buf_out();
    env.charge_weight(<T as Config>::WeightInfo::get_latest_api_upgrade())?;
    let api: Api = env.read_as()?;

    let spec_version = T::Version::get().spec_version;
    let tx_version = T::Version::get().transaction_version;
    let current_chain_version = ChainVersion::new(spec_version, tx_version);

    let current_api_hash: Option<ApiCodeHash<T>> = CurrentApiHash::<T>::get(&api);
    let next_upgrade: Option<NextUpgrade<T>> = ApiNextUpgrade::<T>::get(&api);
    let latest_api_hash = {
        match next_upgrade {
            Some(next_upgrade) => {
                if next_upgrade.chain_version <= current_chain_version {
                    CurrentApiHash::<T>::insert(&api, &next_upgrade.api_hash);
                    ApiNextUpgrade::<T>::remove(&api);
                    Some(next_upgrade.api_hash)
                } else {
                    current_api_hash
                }
            }
            None => current_api_hash,
        }
    };

    // If there are no upgrades found, return an error
    if latest_api_hash.is_none() {
        return Err(Error::<T>::NoUpgradesSupported.into());
    }

    trace!(
        target: "runtime",
        "PolymeshExtension contract GetLatestApiUpgrade: {latest_api_hash:?}",
    );
    let encoded_api_hash = latest_api_hash.unwrap_or_default().encode();
    env.write(&encoded_api_hash, false, None).map_err(|err| {
        trace!(
            target: "runtime",
            "PolymeshExtension failed to write api code hash value into contract memory:{err:?}",
        );
        Error::<T>::ReadStorageFailed
    })?;

    Ok(ce::RetVal::Converging(0))
}

fn call_runtime<T, E>(
    env: ce::Environment<E, ce::InitState>,
    write_error: bool,
) -> ce::Result<ce::RetVal>
where
    <T as BConfig>::RuntimeCall: GetDispatchInfo + GetCallMetadata,
    T: Config,
    T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
    E: ce::Ext<T = T>,
{
    let mut env = env.buf_in_buf_out();
    let in_len = env.in_len();

    // Limit `in_len` to a maximum.
    ensure!(
        in_len <= <T as Config>::MaxInLen::get(),
        Error::<T>::InLenTooLarge
    );

    // Charge weight as a linear function of `in_len`.
    env.charge_weight(<T as Config>::WeightInfo::call_runtime(in_len))?;

    // Decide what to call in the runtime.
    let (call, extrinsic_id) = {
        let input = env.read(in_len)?;
        // Decode the pallet_id & extrinsic_id.
        let ext_id =
            ExtrinsicId::try_from(input.as_slice()).ok_or(Error::<T>::InvalidRuntimeCall)?;
        // Check if the extrinsic is allowed to be called.
        Module::<T>::ensure_call_runtime(ext_id)?;
        (
            <<T as BConfig>::RuntimeCall>::decode_all_with_depth_limit(
                MAX_DECODE_DEPTH,
                &mut input.as_slice(),
            )
            .map_err(|_| Error::<T>::InvalidRuntimeCall)?,
            ext_id,
        )
    };

    // Charge weight for the call.
    let di = call.get_dispatch_info();
    let charged_amount = env.charge_weight(di.weight)?;

    // Execute call requested by contract, with current DID set to the contract owner.
    let addr = env.ext().address().clone();
    // Emit event for calling into the runtime
    Module::<T>::deposit_event(Event::<T>::SCRuntimeCall(addr.clone(), extrinsic_id));
    // Dispatch call
    let result = with_payer::<T, _, _>(addr.clone(), || {
        with_call_metadata(call.get_call_metadata(), || {
            // Dispatch the call, avoiding use of `ext.call_runtime()`,
            // as that uses `CallFilter = Nothing`, which would case a problem for us.
            call.dispatch(RawOrigin::Signed(addr).into())
        })
    });

    // Refund unspent weight.
    let post_di = result.unwrap_or_else(|e| e.post_info);
    // This check isn't necessary but avoids some work.
    if post_di.actual_weight.is_some() {
        let actual_weight = post_di.calc_actual_weight(&di);
        env.adjust_weight(charged_amount, actual_weight);
    }

    // Ensure the call was successful
    if let Err(e) = result {
        if write_error {
            if let DispatchError::Module(e) = e.error {
                let error_string: Result<(), String> = Err(format!("{e:?}"));
                env.write(&error_string.encode(), false, None)
                    .map_err(|_| Error::<T>::ReadStorageFailed)?;
                return Ok(ce::RetVal::Converging(0));
            }
        }
        return Err(e.error);
    }

    if write_error {
        env.write(&Ok::<(), String>(()).encode(), false, None)
            .map_err(|_| Error::<T>::ReadStorageFailed)?;
    }
    // Done; continue with smart contract execution when returning.
    Ok(ce::RetVal::Converging(0))
}

fn get_next_asset_id<T, E>(env: ce::Environment<E, ce::InitState>) -> ce::Result<ce::RetVal>
where
    T: Config,
    T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
    E: ce::Ext<T = T>,
{
    let mut env = env.buf_in_buf_out();

    // Charge weight.
    env.charge_weight(<T as Config>::WeightInfo::get_next_asset_id())?;

    let caller_account: T::AccountId = env.read_as()?;
    trace!(
        target: "runtime",
        "PolymeshExtension contract GetNextAssetId: caller_account={caller_account:?}",
    );
    let asset_id = T::Asset::generate_asset_id(caller_account);
    trace!(
        target: "runtime",
        "PolymeshExtension contract GetNextAssetId: asset_id={asset_id:?}",
    );
    let encoded = asset_id.encode();
    env.write(&encoded, false, None).map_err(|err| {
        trace!(
            target: "runtime",
            "PolymeshExtension failed to write asset_id value into contract memory:{:?}",
            err
        );
        Error::<T>::ReadStorageFailed
    })?;

    Ok(ce::RetVal::Converging(0))
}

#[derive(Clone, Copy, Default)]
pub struct PolymeshExtension;

/// A chain extension allowing calls to polymesh pallets
/// and using the contract's DID instead of the caller's DID.
impl<T> ce::ChainExtension<T> for PolymeshExtension
where
    <T as BConfig>::RuntimeCall: GetDispatchInfo + GetCallMetadata,
    T: Config,
    T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
{
    fn enabled() -> bool {
        true
    }

    fn call<E: ce::Ext<T = T>>(
        &mut self,
        env: ce::Environment<E, ce::InitState>,
    ) -> ce::Result<ce::RetVal> {
        let ext_id = ((env.ext_id() as u32) << 16) + env.func_id() as u32;
        // Decode chain extension id.
        let func_id = FuncId::try_from(ext_id);

        trace!(
            target: "runtime",
            "PolymeshExtension contract calling: {func_id:?}",
        );
        let res = match func_id {
            // `FuncId::NOP` is only used to benchmark the cost of:
            // 1. Calling a contract.
            // 2. Calling `seal_call_chain_extension` from the contract.
            #[cfg(feature = "runtime-benchmarks")]
            Some(FuncId::NOP) => {
                // Return without doing any work.
                Ok(ce::RetVal::Converging(0))
            }
            Some(FuncId::ReadStorage) => read_storage(env),
            Some(FuncId::CallRuntime) => call_runtime(env, false),
            Some(FuncId::GetSpecVersion) => get_version(env, true),
            Some(FuncId::GetTransactionVersion) => get_version(env, false),
            Some(FuncId::GetKeyDid) => get_key_did(env),
            Some(FuncId::KeyHasher(hasher, size)) => key_hasher(env, hasher, size),
            Some(FuncId::GetLatestApiUpgrade) => get_latest_api_upgrade(env),
            Some(FuncId::CallRuntimeWithError) => call_runtime(env, true),
            Some(FuncId::GetNextAssetId) => get_next_asset_id(env),
            None => {
                trace!(
                    target: "runtime",
                    "PolymeshExtension contract calling invalid ext_id={ext_id:?}",
                );
                Err(Error::<T>::InvalidFuncId)?
            }
        };
        if let Err(err) = &res {
            trace!(
                target: "runtime",
                "PolymeshExtension: err={err:?}",
            );
        }

        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_func_id() {
        let test_func_id = |id: FuncId| {
            let id_u32: u32 = id.into();
            let id2 = FuncId::try_from(id_u32).expect("Failed to convert back to FuncId");
            assert_eq!(id, id2);
        };
        #[cfg(feature = "runtime-benchmarks")]
        test_func_id(FuncId::NOP);
        test_func_id(FuncId::CallRuntime);
        test_func_id(FuncId::ReadStorage);
        test_func_id(FuncId::GetSpecVersion);
        test_func_id(FuncId::GetTransactionVersion);
        test_func_id(FuncId::GetKeyDid);
        test_func_id(FuncId::KeyHasher(KeyHasher::Twox, HashSize::B64));
        test_func_id(FuncId::KeyHasher(KeyHasher::Twox, HashSize::B128));
        test_func_id(FuncId::KeyHasher(KeyHasher::Twox, HashSize::B256));
        test_func_id(FuncId::GetLatestApiUpgrade);
        test_func_id(FuncId::CallRuntimeWithError);
    }
}
