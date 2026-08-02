//! Ethereum-side helpers for the Revive tests.
//!
//! These drive `pallet_revive` through its Ethereum compatibility layer: an
//! `alloy` provider talking JSON-RPC to an `eth-rpc` node, with transactions
//! signed locally by secp256k1 keys.
//!
//! See [`crate::revive_helper`] for the same operations driven by Substrate
//! extrinsics.

use anyhow::{anyhow, Result};

use alloy::network::{EthereumWallet, TransactionBuilder};
use alloy::primitives::{Address, Bytes, U256};
use alloy::providers::{DynProvider, Provider, ProviderBuilder};
use alloy::rpc::types::{TransactionReceipt, TransactionRequest};
use alloy::signers::local::PrivateKeySigner;
use alloy::sol_types::{SolCall, SolEvent};

use polymesh_api::types::primitive_types::{H160, H256};

use crate::contracts::{CodeKind, ContractCode};
use crate::*;

/// Default `eth-rpc` endpoint, overridden with the `ETH_RPC_URL` env var.
pub const DEFAULT_ETH_RPC_URL: &str = "http://127.0.0.1:8545";

/// `eth_estimateGas` under-reports for `pallet_revive`, which charges Substrate
/// weight and storage deposits on top of the EVM gas. The same work-around is
/// used by the Foundry CI job (`--gas-estimate-multiplier 500`).
pub const DEFAULT_GAS_MULTIPLIER: u64 = 5;

/// How often [`EthNode::wait_for_sync`] re-checks the `eth-rpc` block number.
const ETH_RPC_SYNC_INTERVAL: std::time::Duration = std::time::Duration::from_millis(200);

/// How many times [`EthNode::wait_for_sync`] re-checks before giving up.
const ETH_RPC_SYNC_TRIES: u32 = 150;

/// Well-known development keys funded by the `--dev` chain spec.
pub mod dev_keys {
    pub const ALITH: &str = "0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133";
    pub const BALTATHAR: &str =
        "0x8075991ce870b93a8870eca0c0f91913d12f47948ca0fd25b49c6fa7cdbeee8b";
    pub const CHARLETH: &str = "0x0b6e18cafb6ed99687ec547bd28139cafdd2bffe70e6b688025de6b445aa5c5b";
    pub const DOROTHY: &str = "0x39539ab1876910bbf3a223d84a29e28f1cb4e2e456503e7e91ed39b2e7223d68";
    pub const ETHAN: &str = "0x7dce9bc8babb68fec1409be38c8e1a52650206a7ed90ff956ae8a6d15eeaaef4";
}

/// The endpoint used by the tests.
pub fn eth_rpc_url() -> String {
    std::env::var("ETH_RPC_URL").unwrap_or_else(|_| DEFAULT_ETH_RPC_URL.to_string())
}

/// Converts a `polymesh-api` `H160` to an `alloy` address.
pub fn to_eth_address(address: &H160) -> Address {
    Address::from(address.0)
}

/// Converts an `alloy` address to a `polymesh-api` `H160`.
pub fn to_h160(address: &Address) -> H160 {
    H160(address.0 .0)
}

/// Maps `signer`'s account into `pallet_revive` and returns its EVM address.
///
/// Precompiles resolve `msg.sender` and address arguments back to Substrate
/// accounts through that mapping, so an account has to be mapped before it can
/// be used as an ERC-20 holder.
pub async fn eth_address_of<S: Signer>(api: &Api, signer: &mut S) -> Result<Address> {
    Ok(to_eth_address(&map_account(api, signer).await?))
}

/// A connection to an `eth-rpc` node.
///
/// The provider has no wallet attached, so it can only be used for reads
/// (`eth_call`, `eth_getBalance`, ...). Use [`EthWallet`] to send transactions.
#[derive(Clone)]
pub struct EthNode {
    pub url: String,
    pub provider: DynProvider,
    pub chain_id: u64,
    api: Api,
}

impl EthNode {
    /// Connects to the endpoint given by `ETH_RPC_URL`.
    pub async fn new(api: &Api) -> Result<Self> {
        let url = eth_rpc_url();
        // No fillers: they would add a nonce and gas price to `eth_call`
        // requests, which `eth-rpc` then validates as if it were a real
        // transaction.
        let provider = ProviderBuilder::new()
            .disable_recommended_fillers()
            .connect_http(url.parse()?)
            .erased();
        let chain_id = provider.get_chain_id().await?;
        log::info!("connected to eth-rpc at {url}, chain id {chain_id}");
        Ok(Self {
            url,
            provider,
            chain_id,
            api: api.clone(),
        })
    }

    /// Waits until the `eth-rpc` node has indexed the chain's current best block.
    ///
    /// `eth-rpc` follows the node's best blocks asynchronously, so a read issued
    /// right after a Substrate extrinsic was included would otherwise be served
    /// from a block that predates it. The block numbers are shared between both
    /// sides, so it is enough to compare them.
    pub async fn wait_for_sync(&self) -> Result<()> {
        let target = self
            .api
            .client()
            .get_block_header(None)
            .await?
            .ok_or_else(|| anyhow!("node has no best block"))?
            .number;
        for _ in 0..ETH_RPC_SYNC_TRIES {
            if self.provider.get_block_number().await? >= target as u64 {
                return Ok(());
            }
            tokio::time::sleep(ETH_RPC_SYNC_INTERVAL).await;
        }
        Err(anyhow!(
            "eth-rpc at {} did not reach block {target}",
            self.url
        ))
    }

    /// Performs an `eth_call` and returns the raw return data.
    pub async fn call_raw(&self, to: Address, data: Vec<u8>) -> Result<Bytes> {
        self.wait_for_sync().await?;
        let tx = TransactionRequest::default().to(to).input(data.into());
        Ok(self.provider.call(tx).await?)
    }

    /// Performs an `eth_call` with a typed Solidity call and decodes the result.
    pub async fn call<C: SolCall>(&self, to: Address, call: &C) -> Result<C::Return> {
        let out = self.call_raw(to, call.abi_encode()).await?;
        Ok(C::abi_decode_returns(&out)?)
    }

    /// Same as [`Self::call`], but the call appears to come from `from`.
    pub async fn call_from<C: SolCall>(
        &self,
        from: Address,
        to: Address,
        call: &C,
    ) -> Result<C::Return> {
        self.wait_for_sync().await?;
        let tx = TransactionRequest::default()
            .from(from)
            .to(to)
            .input(call.abi_encode().into());
        let out = self.provider.call(tx).await?;
        Ok(C::abi_decode_returns(&out)?)
    }

    /// The deployed code at `address`, empty if there is no contract there.
    pub async fn code_at(&self, address: Address) -> Result<Bytes> {
        self.wait_for_sync().await?;
        Ok(self.provider.get_code_at(address).await?)
    }

    /// The native balance of `address`, in the EVM's 18 decimal representation.
    pub async fn balance(&self, address: Address) -> Result<U256> {
        self.wait_for_sync().await?;
        Ok(self.provider.get_balance(address).await?)
    }

    /// A wallet backed by a randomly generated key.
    pub fn new_wallet(&self) -> EthWallet {
        EthWallet::new(self, PrivateKeySigner::random())
    }

    /// A wallet backed by a specific secret key, e.g. one of [`dev_keys`].
    pub fn wallet_from_secret(&self, secret: &str) -> Result<EthWallet> {
        Ok(EthWallet::new(self, secret.parse()?))
    }
}

/// An Ethereum wallet that signs transactions locally and submits them through
/// the `eth-rpc` node.
#[derive(Clone)]
pub struct EthWallet {
    pub signer: PrivateKeySigner,
    pub address: Address,
    pub chain_id: u64,
    /// Multiplier applied to `eth_estimateGas`, see [`DEFAULT_GAS_MULTIPLIER`].
    pub gas_multiplier: u64,
    /// The identity linked to this wallet's Substrate account, once onboarded.
    pub did: Option<IdentityId>,
    node: EthNode,
    provider: DynProvider,
}

impl EthWallet {
    /// Wraps `signer` with a provider that signs and fills transactions for it.
    pub fn new(node: &EthNode, signer: PrivateKeySigner) -> Self {
        let address = signer.address();
        let provider = ProviderBuilder::new()
            .wallet(EthereumWallet::from(signer.clone()))
            .connect_http(node.url.parse().expect("node url was already parsed"))
            .erased();
        Self {
            signer,
            address,
            chain_id: node.chain_id,
            gas_multiplier: DEFAULT_GAS_MULTIPLIER,
            did: None,
            node: node.clone(),
            provider,
        }
    }

    /// This wallet's address as a `polymesh-api` `H160`.
    pub fn h160(&self) -> H160 {
        to_h160(&self.address)
    }

    /// The Substrate account `pallet_revive` maps this address to.
    ///
    /// Ethereum wallets can't sign Substrate extrinsics, so they can never call
    /// `revive.map_account()` and always use the `0xEE`-padded fallback account.
    pub fn account(&self) -> AccountId {
        fallback_account(&self.h160())
    }

    /// Gives this wallet POLYX to pay for gas and storage deposits.
    pub async fn fund(&self, tester: &mut PolymeshTester, polyx: u128) -> Result<()> {
        fund_polyx(tester, self.account(), polyx).await
    }

    /// Registers an identity for this wallet's Substrate account and funds it.
    ///
    /// A DID is required before the account can hold Polymesh assets, and the
    /// POLYX pays for gas and storage deposits.
    pub async fn onboard(
        &mut self,
        tester: &mut PolymeshTester,
        polyx: u128,
    ) -> Result<IdentityId> {
        let did = onboard_account(tester, self.account(), polyx).await?;
        self.did = Some(did);
        Ok(did)
    }

    /// The identity of this wallet, which must have been onboarded first.
    pub fn did(&self) -> Result<IdentityId> {
        self.did
            .ok_or_else(|| anyhow!("eth wallet {} has no identity", self.address))
    }

    /// Signs and submits `tx`, then waits for its receipt.
    ///
    /// Returns an error if the transaction reverted.
    pub async fn send(&self, tx: TransactionRequest) -> Result<TransactionReceipt> {
        self.node.wait_for_sync().await?;
        let tx = tx.from(self.address);
        // Fill in a gas limit ourselves: the estimate returned by `eth-rpc` does
        // not cover the Substrate-side weight and storage deposit.
        let gas = self.provider.estimate_gas(tx.clone()).await?;
        let tx = tx.gas_limit(gas.saturating_mul(self.gas_multiplier));

        let receipt = self
            .provider
            .send_transaction(tx)
            .await?
            .get_receipt()
            .await?;
        if !receipt.status() {
            return Err(anyhow!(
                "eth transaction {:?} reverted",
                receipt.transaction_hash
            ));
        }
        Ok(receipt)
    }

    /// Deploys a contract and returns its address.
    pub async fn deploy(
        &self,
        contract: &ContractCode,
        kind: CodeKind,
        ctor_args: Vec<u8>,
    ) -> Result<Address> {
        let receipt = self
            .send(
                TransactionRequest::default().with_deploy_code(contract.init_code(kind, ctor_args)),
            )
            .await?;
        receipt
            .contract_address
            .ok_or_else(|| anyhow!("no contract address in receipt for {}", contract.name))
    }

    /// Sends a typed Solidity call as a transaction.
    pub async fn call<C: SolCall>(&self, to: Address, call: &C) -> Result<TransactionReceipt> {
        self.send(
            TransactionRequest::default()
                .to(to)
                .input(call.abi_encode().into()),
        )
        .await
    }

    /// Runs a typed Solidity call as an `eth_call` from this wallet's address.
    pub async fn read<C: SolCall>(&self, to: Address, call: &C) -> Result<C::Return> {
        self.node.call_from(self.address, to, call).await
    }
}

/// Decodes all Solidity events of type `E` emitted by `address` in `receipt`.
pub fn decode_receipt_logs<E: SolEvent>(
    receipt: &TransactionReceipt,
    address: Address,
) -> Result<Vec<E>> {
    let mut decoded = Vec::new();
    for log in receipt.logs() {
        if log.address() != address || log.topics().first() != Some(&E::SIGNATURE_HASH) {
            continue;
        }
        decoded.push(E::decode_log_data(log.data())?);
    }
    Ok(decoded)
}

#[async_trait::async_trait]
impl ContractCaller for EthWallet {
    fn caller_address(&self) -> H160 {
        self.h160()
    }

    fn caller_account(&self) -> AccountId {
        self.account()
    }

    async fn send_call(&mut self, to: H160, data: Vec<u8>) -> Result<Vec<ContractLog>> {
        let receipt = self
            .send(
                TransactionRequest::default()
                    .to(to_eth_address(&to))
                    .input(data.into()),
            )
            .await?;
        Ok(receipt
            .logs()
            .iter()
            .map(|log| ContractLog {
                address: to_h160(&log.address()),
                topics: log.topics().iter().map(|t| H256(t.0)).collect(),
                data: log.data().data.to_vec(),
            })
            .collect())
    }
}
