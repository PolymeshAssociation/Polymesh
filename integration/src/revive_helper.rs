//! Helpers for deploying and calling `pallet_revive` contracts through normal
//! Substrate extrinsics.
//!
//! See [`crate::eth_helper`] for the same operations driven by Ethereum
//! transactions through the `eth-rpc` node.

use anyhow::{anyhow, Result};

use alloy::primitives::keccak256;
use alloy::sol_types::{SolCall, SolEvent};

use polymesh_api::types::pallet_revive::pallet::ReviveEvent;
use polymesh_api::types::primitive_types::{H160, H256};
use sp_weights::Weight;

use crate::contracts::{CodeKind, ContractCode};
use crate::*;

/// Weight limit used for contract extrinsics.
///
/// Contract calls are metered by `pallet_revive` itself, so the tests just pass
/// a generous limit instead of running a dry-run for every call.
pub const CONTRACT_WEIGHT_LIMIT: Weight = Weight::from_parts(50_000_000_000, 2_000_000);

/// Storage deposit limit used for contract extrinsics.
///
/// `DepositPerByte` is 0.06 POLYX, so deploying a few KB of bytecode needs a
/// few hundred POLYX of head room.
pub const STORAGE_DEPOSIT_LIMIT: u128 = 10_000 * ONE_POLYX;

/// POLYX given to each account in the Revive tests.
///
/// Contract deployment pays a storage deposit per code byte, which is far more
/// than the other integration tests need.
pub const REVIVE_INIT_POLYX: u128 = 100_000;

/// Standard setup for the Revive tests: a tester funded for contract
/// deployments plus a connection to the `eth-rpc` node.
pub async fn revive_tester() -> Result<(PolymeshTester, EthNode)> {
    let mut tester = PolymeshTester::new().await?;
    tester.set_init_polyx(REVIVE_INIT_POLYX);
    let node = EthNode::new(&tester.api).await?;
    Ok((tester, node))
}

/// Maps a Substrate account to the address `pallet_revive` sees for it.
///
/// This is `AccountId32Mapper`'s "to address" direction: the last 20 bytes of
/// the keccak hash of the account id.
pub fn address_of(account: &AccountId) -> H160 {
    let hash = keccak256(account.0);
    let mut addr = [0u8; 20];
    addr.copy_from_slice(&hash[12..]);
    H160(addr)
}

/// The account id `pallet_revive` uses for an address that has never called
/// `revive.map_account()`.
///
/// This is the "fallback account": the address padded with `0xEE` bytes. It is
/// also the account id of a deployed contract.
pub fn fallback_account(address: &H160) -> AccountId {
    let mut account = [0xEEu8; 32];
    account[..20].copy_from_slice(&address.0);
    AccountId::from(account)
}

/// Resolves the account id `pallet_revive` will use for `address`, taking a
/// previous `map_account()` into account.
pub async fn account_of(api: &Api, address: &H160) -> Result<AccountId> {
    Ok(api
        .query()
        .revive()
        .original_account(address.clone())
        .await?
        .unwrap_or_else(|| fallback_account(address)))
}

/// Links `signer`'s Substrate account to its Ethereum address.
///
/// Without this, `pallet_revive` cannot map the address back to the original
/// account, so anything that acts on the caller's Substrate account (such as
/// the Polymesh ERC-20 precompile) would see the `0xEE` fallback account
/// instead. Calling this twice is a no-op.
pub async fn map_account<S: Signer>(api: &Api, signer: &mut S) -> Result<H160> {
    let address = address_of(&signer.account());
    if api
        .query()
        .revive()
        .original_account(address.clone())
        .await?
        .is_none()
    {
        api.call()
            .revive()
            .map_account()?
            .execute(signer)
            .await?
            .ok()
            .await?;
    }
    Ok(address)
}

/// Drops a signer's cached nonce, so its next transaction reads the nonce back
/// from the chain.
///
/// The cache lives behind the signer's lock, so it has to be reset through the
/// same lock the transaction code uses.
pub async fn reset_nonce<S: Signer>(signer: &mut S) {
    match signer.lock().await {
        Some(mut locked) => locked.set_nonce(0).await,
        None => signer.set_nonce(0).await,
    }
}

/// Returns the address of the contract instantiated by `res`.
pub async fn get_contract_address(res: &mut TransactionResults) -> Result<Option<H160>> {
    if let Some(events) = res.events().await? {
        for rec in &events.0 {
            if let RuntimeEvent::Revive(ReviveEvent::Instantiated { contract, .. }) = &rec.event {
                return Ok(Some(contract.clone()));
            }
        }
    }
    Ok(None)
}

/// A log emitted by a contract or precompile.
///
/// Both transaction styles produce these: `pallet_revive` reports them as
/// `ContractEmitted` runtime events, `eth-rpc` as receipt logs.
#[derive(Clone, Debug)]
pub struct ContractLog {
    pub address: H160,
    pub topics: Vec<H256>,
    pub data: Vec<u8>,
}

/// Decodes the Solidity events of type `E` that `address` emitted in `logs`.
pub fn decode_contract_logs<E: SolEvent>(logs: &[ContractLog], address: &H160) -> Result<Vec<E>> {
    let mut decoded = Vec::new();
    for log in logs {
        if &log.address != address {
            continue;
        }
        let topics: Vec<alloy::primitives::B256> = log.topics.iter().map(|t| t.0.into()).collect();
        if topics.first() != Some(&E::SIGNATURE_HASH) {
            continue;
        }
        decoded.push(E::decode_raw_log(topics, &log.data)?);
    }
    Ok(decoded)
}

/// Returns the `ContractEmitted` events of `res`.
pub async fn contract_logs(res: &mut TransactionResults) -> Result<Vec<ContractLog>> {
    let mut logs = Vec::new();
    if let Some(events) = res.events().await? {
        for rec in &events.0 {
            if let RuntimeEvent::Revive(ReviveEvent::ContractEmitted {
                contract,
                data,
                topics,
            }) = &rec.event
            {
                logs.push(ContractLog {
                    address: contract.clone(),
                    topics: topics.clone(),
                    data: data.clone(),
                });
            }
        }
    }
    Ok(logs)
}

/// Decodes all Solidity events of type `E` emitted by `address` in `res`.
pub async fn decode_logs<E: SolEvent>(
    res: &mut TransactionResults,
    address: &H160,
) -> Result<Vec<E>> {
    decode_contract_logs(&contract_logs(res).await?, address)
}

/// A deployed contract, addressed by its `pallet_revive` address.
#[derive(Clone)]
pub struct Contract {
    pub api: Api,
    pub address: H160,
}

impl Contract {
    /// Wraps an already deployed contract.
    pub fn new(api: &Api, address: H160) -> Self {
        Self {
            api: api.clone(),
            address,
        }
    }

    /// Deploys `code` with `revive.instantiate_with_code`.
    ///
    /// `ctor_args` are the ABI-encoded constructor arguments; see
    /// [`crate::contracts::ctor`].
    pub async fn deploy<S: Signer>(
        api: &Api,
        signer: &mut S,
        contract: &ContractCode,
        kind: CodeKind,
        ctor_args: Vec<u8>,
    ) -> Result<Self> {
        map_account(api, signer).await?;
        let (code, data) = contract.deploy_payload(kind, ctor_args);
        let mut res = api
            .call()
            .revive()
            .instantiate_with_code(
                0,
                CONTRACT_WEIGHT_LIMIT,
                STORAGE_DEPOSIT_LIMIT,
                code,
                data,
                None,
            )?
            .execute(signer)
            .await?;
        res.ok().await?;
        // `pallet_revive` bumps the deployer's account nonce itself, to give
        // every contract a distinct address like EVM's `CREATE` does. That extra
        // increment is invisible to the signer's nonce cache, so drop it and let
        // the next transaction read the nonce back from the chain.
        reset_nonce(signer).await;

        let address = get_contract_address(&mut res)
            .await?
            .ok_or_else(|| anyhow!("no `Instantiated` event for {}", contract.name))?;
        log::info!("deployed {} at {:?}", contract.name, address);
        Ok(Self::new(api, address))
    }

    /// The account id used by this contract when it calls into the runtime.
    pub fn account(&self) -> AccountId {
        fallback_account(&self.address)
    }

    /// Calls the contract with raw ABI-encoded input.
    pub async fn call_raw<S: Signer>(
        &self,
        signer: &mut S,
        data: Vec<u8>,
        value: u128,
    ) -> Result<TransactionResults> {
        map_account(&self.api, signer).await?;
        Ok(self
            .api
            .call()
            .revive()
            .call(
                self.address.clone(),
                value,
                CONTRACT_WEIGHT_LIMIT,
                STORAGE_DEPOSIT_LIMIT,
                data,
            )?
            .execute(signer)
            .await?)
    }

    /// Calls the contract with a typed Solidity call and waits for it to succeed.
    ///
    /// Note that `revive.call` does not return the contract's return data to the
    /// caller, so state has to be checked with a follow-up `eth_call` (see
    /// [`crate::eth_helper::EthNode::call`]) or by inspecting emitted events.
    pub async fn call<S: Signer, C: SolCall>(
        &self,
        signer: &mut S,
        call: &C,
    ) -> Result<TransactionResults> {
        let mut res = self.call_raw(signer, call.abi_encode(), 0).await?;
        res.ok().await?;
        Ok(res)
    }

    /// Calls the contract through any [`ContractCaller`].
    ///
    /// This lets a test run the same scenario with either transaction style by
    /// swapping the caller.
    pub async fn call_as<C: SolCall>(
        &self,
        caller: &mut dyn ContractCaller,
        call: &C,
    ) -> Result<Vec<ContractLog>> {
        caller
            .send_call(self.address.clone(), call.abi_encode())
            .await
    }
}

/// Something that can submit a state-changing call to a contract or precompile.
///
/// Implemented for both transaction styles: [`SubstrateCaller`] wraps a
/// Substrate signer using `revive.call`, and [`crate::eth_helper::EthWallet`]
/// signs an Ethereum transaction submitted through the `eth-rpc` node.
#[async_trait::async_trait]
pub trait ContractCaller: Send {
    /// The address the callee will see as `msg.sender`.
    fn caller_address(&self) -> H160;

    /// The Substrate account the callee's runtime calls will be attributed to.
    fn caller_account(&self) -> AccountId;

    /// Submits `data` to `to`, waits for the call to succeed and returns the
    /// logs it emitted.
    async fn send_call(&mut self, to: H160, data: Vec<u8>) -> Result<Vec<ContractLog>>;
}

/// A [`ContractCaller`] backed by a Substrate signer using `revive.call`.
pub struct SubstrateCaller<'a, S: Signer> {
    pub api: Api,
    pub address: H160,
    pub signer: &'a mut S,
}

impl<'a, S: Signer> SubstrateCaller<'a, S> {
    /// Wraps `signer` as a contract caller.
    ///
    /// This maps the signer's account first, so that precompiles resolve
    /// `msg.sender` back to the original Substrate account instead of the
    /// `0xEE` fallback account.
    pub async fn new(api: &Api, signer: &'a mut S) -> Result<Self> {
        let address = map_account(api, signer).await?;
        Ok(Self {
            api: api.clone(),
            address,
            signer,
        })
    }
}

#[async_trait::async_trait]
impl<S: Signer> ContractCaller for SubstrateCaller<'_, S> {
    fn caller_address(&self) -> H160 {
        self.address.clone()
    }

    fn caller_account(&self) -> AccountId {
        self.signer.account()
    }

    async fn send_call(&mut self, to: H160, data: Vec<u8>) -> Result<Vec<ContractLog>> {
        let mut res = self
            .api
            .call()
            .revive()
            .call(to, 0, CONTRACT_WEIGHT_LIMIT, STORAGE_DEPOSIT_LIMIT, data)?
            .execute(self.signer)
            .await?;
        res.ok().await?;
        contract_logs(&mut res).await
    }
}
