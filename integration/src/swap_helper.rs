//! Helpers for the `SimpleSwap` test contract.

use anyhow::Result;

use alloy::primitives::{Address, U256};

use crate::contracts::{ctor, CodeKind, ISimpleSwap, SIMPLE_SWAP};
use crate::*;

/// A deployed `SimpleSwap` contract together with the two tokens it swaps.
///
/// Either token may be a Polymesh native asset (through the ERC-20 precompile)
/// or a plain Solidity ERC-20, see [`Token`].
pub struct SwapHelper {
    pub contract: Contract,
    pub address: Address,
    pub token_a: Token,
    pub token_b: Token,
    node: EthNode,
}

impl SwapHelper {
    /// Deploys `SimpleSwap` and registers an identity for its account.
    ///
    /// The contract holds its own inventory and Polymesh requires an identity
    /// for every asset holder. Contracts can't sign extrinsics, so a registrar
    /// has to register the identity for them.
    pub async fn deploy<S: Signer>(
        tester: &mut PolymeshTester,
        node: &EthNode,
        deployer: &mut S,
        kind: CodeKind,
        token_a: &Token,
        token_b: &Token,
        rate_num: u128,
        rate_den: u128,
    ) -> Result<Self> {
        let api = tester.api.clone();
        let contract = Contract::deploy(
            &api,
            deployer,
            &SIMPLE_SWAP,
            kind,
            ctor::simple_swap(token_a.address, token_b.address, rate_num, rate_den),
        )
        .await?;
        register_did(tester, contract.account()).await?;

        Ok(Self {
            address: to_eth_address(&contract.address),
            contract,
            token_a: token_a.clone(),
            token_b: token_b.clone(),
            node: node.clone(),
        })
    }

    /// The Substrate account the contract holds native assets in.
    pub fn account(&self) -> AccountId {
        self.contract.account()
    }

    /// How much token B `amount_in` of token A is worth.
    pub async fn quote_a_to_b(&self, amount_in: u128) -> Result<u128> {
        let value = self
            .node
            .call(
                self.address,
                &ISimpleSwap::quoteAtoBCall {
                    amountIn: U256::from(amount_in),
                },
            )
            .await?;
        Ok(value.try_into()?)
    }

    /// How much token A `amount_in` of token B is worth.
    pub async fn quote_b_to_a(&self, amount_in: u128) -> Result<u128> {
        let value = self
            .node
            .call(
                self.address,
                &ISimpleSwap::quoteBtoACall {
                    amountIn: U256::from(amount_in),
                },
            )
            .await?;
        Ok(value.try_into()?)
    }

    /// Swaps `amount_in` of token A for token B.
    ///
    /// The caller must have approved the contract on token A first.
    pub async fn swap_a_to_b(
        &self,
        caller: &mut dyn ContractCaller,
        amount_in: u128,
    ) -> Result<Vec<ContractLog>> {
        self.contract
            .call_as(
                caller,
                &ISimpleSwap::swapAtoBCall {
                    amountIn: U256::from(amount_in),
                },
            )
            .await
    }

    /// Swaps `amount_in` of token B for token A.
    ///
    /// The caller must have approved the contract on token B first.
    pub async fn swap_b_to_a(
        &self,
        caller: &mut dyn ContractCaller,
        amount_in: u128,
    ) -> Result<Vec<ContractLog>> {
        self.contract
            .call_as(
                caller,
                &ISimpleSwap::swapBtoACall {
                    amountIn: U256::from(amount_in),
                },
            )
            .await
    }

    /// The `Swap` events emitted by this contract in `logs`.
    pub fn swap_events(&self, logs: &[ContractLog]) -> Result<Vec<ISimpleSwap::Swap>> {
        decode_contract_logs(logs, &self.contract.address)
    }

    /// The contract's balance of both tokens.
    pub async fn liquidity(&self) -> Result<(u128, u128)> {
        Ok((
            self.token_a.balance_of(self.address).await?,
            self.token_b.balance_of(self.address).await?,
        ))
    }
}
