use codec::{Decode, Encode};
use frame_support::{
    dispatch::{DispatchError, Dispatchable, GetDispatchInfo},
    ensure,
    log::trace,
    storage::unhashed,
    traits::{Get, GetCallMetadata},
};
use frame_system::RawOrigin;
use pallet_contracts::chain_extension as ce;
use pallet_contracts::Config as BConfig;
use pallet_permissions::with_call_metadata;
use polymesh_common_utilities::Context;
use polymesh_primitives::IdentityId;
use scale_info::TypeInfo;
use sp_core::crypto::UncheckedFrom;
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};

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

    /// Deprecated Polymesh (<=5.0) chain extensions.
    OldCallRuntime(ExtrinsicId),
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
                _ => None,
            },
            0x1A => match func_id {
                0x00_00 | 0x01_00 | 0x02_00 | 0x03_00 | 0x11_00 => Some(Self::OldCallRuntime(
                    ExtrinsicId(0x1A, (func_id >> 8) as u8),
                )),
                _ => None,
            },
            0x2F => match func_id {
                0x01_00 => Some(Self::OldCallRuntime(ExtrinsicId(0x2F, 0x01))),
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
            Self::OldCallRuntime(ExtrinsicId(ext_id, func_id)) => {
                (ext_id as u32, (func_id as u32) << 8)
            }
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

/// Returns the `contract`'s DID or errors.
fn contract_did<T: Config>(contract: &T::AccountId) -> Result<IdentityId, DispatchError>
where
    T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
{
    // N.B. it might be the case that the contract is a primary key due to rotation.
    Ok(Identity::<T>::get_identity(contract).ok_or(Error::<T>::InstantiatorWithNoIdentity)?)
}

/// Run `with` while the current DID and Payer is temporarily set to the given one.
fn with_did_and_payer<T: Config, W: FnOnce() -> R, R>(
    did: IdentityId,
    payer: T::AccountId,
    with: W,
) -> R {
    let old_payer = Context::current_payer::<Identity<T>>();
    let old_did = Context::current_identity::<Identity<T>>();
    Context::set_current_payer::<Identity<T>>(Some(payer));
    Context::set_current_identity::<Identity<T>>(Some(did));
    let result = with();
    Context::set_current_payer::<Identity<T>>(old_payer);
    Context::set_current_identity::<Identity<T>>(old_did);
    result
}

// This is used to convert the `OldCallRuntime` to `CallRuntime`.
struct ChainInput<'a, 'b> {
    input1: &'a [u8],
    input2: &'b [u8],
}

impl<'a, 'b> ChainInput<'a, 'b> {
    pub fn new(input1: &'a [u8], input2: &'b [u8]) -> Self {
        Self { input1, input2 }
    }

    pub fn len(&self) -> usize {
        self.input1.len() + self.input2.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a, 'b> codec::Input for ChainInput<'a, 'b> {
    fn remaining_len(&mut self) -> Result<Option<usize>, codec::Error> {
        Ok(Some(self.len()))
    }

    fn read(&mut self, into: &mut [u8]) -> Result<(), codec::Error> {
        let len = into.len();
        let in1_len = self.input1.len();
        let in2_len = self.input2.len();
        if len > (in1_len + in2_len) {
            return Err("Not enough data to fill buffer".into());
        }
        // `input1` still has bytes, read from it first.
        if in1_len > 0 {
            let off = in1_len.min(len);
            // Split `into` buffer into two parts.
            let (into1, into2) = into.split_at_mut(off);
            // Read from `input1`.
            let len = into1.len();
            into1.copy_from_slice(&self.input1[..len]);
            self.input1 = &self.input1[len..];
            // Read from `input2`.
            let len = into2.len();
            into2.copy_from_slice(&self.input2[..len]);
            self.input2 = &self.input2[len..];
        } else {
            // `input1` is empty, only read from `input2`.
            let len = into.len();
            into.copy_from_slice(&self.input2[..len]);
            self.input2 = &self.input2[len..];
        }
        Ok(())
    }
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
    env.charge_weight(<T as Config>::WeightInfo::get_version())?;

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

fn call_runtime<T, E>(
    env: ce::Environment<E, ce::InitState>,
    old_call: Option<ExtrinsicId>,
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
    use codec::DecodeLimit;
    let call = match old_call {
        None => {
            let input = env.read(in_len)?;
            // Decode the pallet_id & extrinsic_id.
            let ext_id =
                ExtrinsicId::try_from(input.as_slice()).ok_or(Error::<T>::InvalidRuntimeCall)?;
            // Check if the extrinsic is allowed to be called.
            Module::<T>::ensure_call_runtime(ext_id)?;
            <<T as BConfig>::RuntimeCall>::decode_all_with_depth_limit(
                MAX_DECODE_DEPTH,
                &mut input.as_slice(),
            )
            .map_err(|_| Error::<T>::InvalidRuntimeCall)?
        }
        Some(ext_id) => {
            // Check if the extrinsic is allowed to be called.
            Module::<T>::ensure_call_runtime(ext_id)?;
            // Convert old ChainExtension runtime calls into `Call` format.
            let extrinsic: [u8; 2] = ext_id.into();
            let params = env.read(in_len)?;
            let mut input = ChainInput::new(&extrinsic, params.as_slice());
            let call = <<T as BConfig>::RuntimeCall>::decode_with_depth_limit(
                MAX_DECODE_DEPTH,
                &mut input,
            )
            .map_err(|_| Error::<T>::InvalidRuntimeCall)?;
            ensure!(input.is_empty(), Error::<T>::DataLeftAfterDecoding);
            call
        }
    };

    // Charge weight for the call.
    let di = call.get_dispatch_info();
    let charged_amount = env.charge_weight(di.weight)?;

    // Execute call requested by contract, with current DID set to the contract owner.
    let addr = env.ext().address().clone();
    let result = with_did_and_payer::<T, _, _>(contract_did::<T>(&addr)?, addr.clone(), || {
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

    // Ensure the call was successful.
    result.map_err(|e| e.error)?;

    // Done; continue with smart contract execution when returning.
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
        match func_id {
            // `FuncId::NOP` is only used to benchmark the cost of:
            // 1. Calling a contract.
            // 2. Calling `seal_call_chain_extension` from the contract.
            #[cfg(feature = "runtime-benchmarks")]
            Some(FuncId::NOP) => {
                // Return without doing any work.
                return Ok(ce::RetVal::Converging(0));
            }
            Some(FuncId::ReadStorage) => read_storage(env),
            Some(FuncId::CallRuntime) => call_runtime(env, None),
            Some(FuncId::GetSpecVersion) => get_version(env, true),
            Some(FuncId::GetTransactionVersion) => get_version(env, false),
            Some(FuncId::GetKeyDid) => get_key_did(env),
            Some(FuncId::KeyHasher(hasher, size)) => key_hasher(env, hasher, size),
            Some(FuncId::OldCallRuntime(p)) => call_runtime(env, Some(p)),
            None => {
                trace!(
                    target: "runtime",
                    "PolymeshExtension contract calling invalid ext_id={ext_id:?}",
                );
                Err(Error::<T>::InvalidFuncId)?
            }
        }
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
        test_func_id(FuncId::OldCallRuntime(ExtrinsicId(0x1A, 0x00)));
        test_func_id(FuncId::OldCallRuntime(ExtrinsicId(0x1A, 0x01)));
        test_func_id(FuncId::OldCallRuntime(ExtrinsicId(0x1A, 0x02)));
        test_func_id(FuncId::OldCallRuntime(ExtrinsicId(0x1A, 0x03)));
        test_func_id(FuncId::OldCallRuntime(ExtrinsicId(0x1A, 0x11)));
        test_func_id(FuncId::OldCallRuntime(ExtrinsicId(0x2F, 0x01)));
    }
}
