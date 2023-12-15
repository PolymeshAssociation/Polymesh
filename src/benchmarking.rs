//! Setup code for [`super::command`] which would otherwise bloat that module.
//!
//! Should only be used for benchmarking as it may break in other contexts.

use crate::service::FullClient;
use polymesh_primitives::{AccountId, Balance, Signature};

use polymesh_runtime_common::BlockHashCount;
use polymesh_runtime_develop::runtime::{self, BalancesCall, SystemCall};
use sc_cli::Result;
use sc_client_api::BlockBackend;
use sc_executor::NativeExecutionDispatch;
use sp_core::{Encode, Pair};
use sp_inherents::{InherentData, InherentDataProvider};
use sp_keyring::Sr25519Keyring;
use sp_runtime::{OpaqueExtrinsic, SaturatedConversion};

use std::{sync::Arc, time::Duration};

/// Generates extrinsics for the `benchmark overhead` command.
///
/// Note: Should only be used for benchmarking.
pub struct RemarkBuilder<R, D: NativeExecutionDispatch + 'static> {
    client: Arc<FullClient<R, D>>,
}

impl<R, D: NativeExecutionDispatch + 'static> RemarkBuilder<R, D> {
    /// Creates a new [`Self`] from the given client.
    pub fn new(client: Arc<FullClient<R, D>>) -> Self {
        Self { client }
    }
}

impl<R, D: NativeExecutionDispatch + 'static> frame_benchmarking_cli::ExtrinsicBuilder
    for RemarkBuilder<R, D>
{
    fn pallet(&self) -> &str {
        "system"
    }

    fn extrinsic(&self) -> &str {
        "remark"
    }

    fn build(&self, nonce: u32) -> std::result::Result<OpaqueExtrinsic, &'static str> {
        let acc = Sr25519Keyring::Bob.pair();
        let extrinsic: OpaqueExtrinsic = create_benchmark_extrinsic(
            self.client.as_ref(),
            acc,
            SystemCall::remark { remark: vec![] }.into(),
            nonce,
        )
        .into();

        Ok(extrinsic)
    }
}

/// Generates `Balances::Transfer` extrinsics for the benchmarks.
///
/// Note: Should only be used for benchmarking.
pub struct TransferBuilder<R, D: NativeExecutionDispatch + 'static> {
    client: Arc<FullClient<R, D>>,
    dest: AccountId,
    value: Balance,
}

impl<R, D: NativeExecutionDispatch + 'static> TransferBuilder<R, D> {
    /// Creates a new [`Self`] from the given client.
    pub fn new(client: Arc<FullClient<R, D>>, dest: AccountId, value: Balance) -> Self {
        Self {
            client,
            dest,
            value,
        }
    }
}

impl<R, D: NativeExecutionDispatch + 'static> frame_benchmarking_cli::ExtrinsicBuilder
    for TransferBuilder<R, D>
{
    fn pallet(&self) -> &str {
        "balances"
    }

    fn extrinsic(&self) -> &str {
        "transfer"
    }

    fn build(&self, nonce: u32) -> std::result::Result<OpaqueExtrinsic, &'static str> {
        let acc = Sr25519Keyring::Bob.pair();
        let extrinsic: OpaqueExtrinsic = create_benchmark_extrinsic(
            self.client.as_ref(),
            acc,
            BalancesCall::transfer {
                dest: self.dest.clone().into(),
                value: self.value.into(),
            }
            .into(),
            nonce,
        )
        .into();

        Ok(extrinsic)
    }
}

/// Create a transaction using the given `call`.
///
/// Note: Should only be used for benchmarking.
pub fn create_benchmark_extrinsic<R, D: NativeExecutionDispatch + 'static>(
    client: &FullClient<R, D>,
    sender: sp_core::sr25519::Pair,
    call: runtime::RuntimeCall,
    nonce: u32,
) -> runtime::UncheckedExtrinsic {
    let genesis_hash = client
        .block_hash(0)
        .ok()
        .flatten()
        .expect("Genesis block exists; qed");
    let best_hash = client.chain_info().best_hash;
    let best_block = client.chain_info().best_number;

    let period = BlockHashCount::get()
        .checked_next_power_of_two()
        .map(|c| c / 2)
        .unwrap_or(2) as u64;
    let extra: runtime::SignedExtra = (
        frame_system::CheckSpecVersion::<runtime::Runtime>::new(),
        frame_system::CheckTxVersion::<runtime::Runtime>::new(),
        frame_system::CheckGenesis::<runtime::Runtime>::new(),
        frame_system::CheckEra::<runtime::Runtime>::from(sp_runtime::generic::Era::mortal(
            period,
            best_block.saturated_into(),
        )),
        frame_system::CheckNonce::<runtime::Runtime>::from(nonce),
        polymesh_extensions::CheckWeight::new(),
        pallet_transaction_payment::ChargeTransactionPayment::<runtime::Runtime>::from(0),
        pallet_permissions::StoreCallMetadata::new(),
    );

    let raw_payload = runtime::SignedPayload::from_raw(
        call.clone(),
        extra.clone(),
        (
            runtime::VERSION.spec_version,
            runtime::VERSION.transaction_version,
            genesis_hash,
            best_hash,
            (),
            (),
            (),
            (),
        ),
    );
    let signature = raw_payload.using_encoded(|e| sender.sign(e));

    runtime::UncheckedExtrinsic::new_signed(
        call.clone(),
        sp_runtime::AccountId32::from(sender.public()).into(),
        Signature::Sr25519(signature.clone()),
        extra.clone(),
    )
}

/// Generates inherent data for the `benchmark overhead` command.
///
/// Note: Should only be used for benchmarking.
pub fn inherent_benchmark_data() -> Result<InherentData> {
    let mut inherent_data = InherentData::new();
    let d = Duration::from_millis(0);
    let timestamp = sp_timestamp::InherentDataProvider::new(d.into());

    futures::executor::block_on(timestamp.provide_inherent_data(&mut inherent_data))
        .map_err(|e| format!("creating inherent data: {:?}", e))?;
    Ok(inherent_data)
}
