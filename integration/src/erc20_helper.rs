//! Helpers for the Polymesh native-asset ERC-20 precompile.
//!
//! Every Polymesh asset is reachable from the EVM at a deterministic address
//! derived from its asset index, where `pallet_precompiles` exposes an ERC-20
//! interface backed by `pallet_asset`.

use anyhow::{anyhow, Result};

use alloy::primitives::{Address, U256};
use alloy::sol_types::SolCall;

use polymesh_api::types::polymesh_primitives::asset::AssetHolderKind;
use polymesh_api::types::polymesh_primitives::ticker::Ticker;
use polymesh_api::types::primitive_types::H160;
use polymesh_precompiles::IFungibleAsset as ierc20;

use crate::*;

/// The `AddressMatcher::Prefix` value used by `PolymeshInterface`.
pub const POLYMESH_PRECOMPILE_PREFIX: u16 = 8;

/// All Polymesh assets report 6 decimals through the precompile.
pub const ERC20_DECIMALS: u8 = 6;

/// The precompile address for the asset with index `asset_index`.
///
/// `pallet_revive` matches a `Prefix` precompile on bytes `[16, 17]` of the
/// address (big endian), reserves bytes `[18, 19]` for builtin precompiles, and
/// leaves the leading 4 bytes free for the precompile to use:
///
/// ```text
/// xxxxxxxx000000000000000000000000pppp0000
/// ^ asset index (BE)              ^ matcher (BE)
/// ```
pub fn precompile_address(asset_index: u32) -> H160 {
    let mut address = [0u8; 20];
    address[0..4].copy_from_slice(&asset_index.to_be_bytes());
    address[16..18].copy_from_slice(&POLYMESH_PRECOMPILE_PREFIX.to_be_bytes());
    H160(address)
}

/// Looks up the index assigned to `asset_id` when it was created.
pub async fn asset_index(api: &Api, asset_id: AssetId) -> Result<Option<u32>> {
    Ok(api.query().asset().asset_id_to_index(asset_id).await?)
}

/// A random ticker, so that repeated test runs against the same chain don't
/// collide on the global ticker registry.
pub fn unique_ticker(prefix: &str) -> Ticker {
    use rand::Rng;
    let mut ticker = [b'0'; 12];
    let prefix = prefix.as_bytes();
    let len = prefix.len().min(ticker.len());
    ticker[..len].copy_from_slice(&prefix[..len]);
    let mut rng = rand::thread_rng();
    for byte in ticker[len..].iter_mut() {
        *byte = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"[rng.gen_range(0..36)];
    }
    Ticker(ticker)
}

/// Registers `ticker` and links it to `asset_id`, so that the ERC-20 `symbol()`
/// method returns it.
pub async fn link_ticker(
    api: &Api,
    owner: &mut User,
    asset_id: AssetId,
    ticker: Ticker,
) -> Result<()> {
    api.call()
        .asset()
        .register_unique_ticker(ticker.clone())?
        .execute(owner)
        .await?
        .ok()
        .await?;
    api.call()
        .asset()
        .link_ticker_to_asset_id(ticker, asset_id)?
        .execute(owner)
        .await?
        .ok()
        .await?;
    Ok(())
}

/// Creates a native asset and returns it together with its ERC-20 precompile.
///
/// The ERC-20 interface reads and writes `pallet_asset`'s *account* balances, so
/// the initial supply is issued with [`AssetHolderKind::Account`] instead of
/// into the issuer's default portfolio.
pub async fn create_erc20_asset(
    api: &Api,
    node: &EthNode,
    issuer: &mut User,
    name: &str,
    mint: u128,
) -> Result<(AssetHelper, Erc20Asset)> {
    let asset = AssetHelper::new_full(
        api,
        issuer,
        name,
        mint,
        Default::default(),
        false,
        Some(AssetHolderKind::Account),
    )
    .await?;
    let erc20 = Erc20Asset::new(api, node, asset.asset_id).await?;
    Ok((asset, erc20))
}

/// An ERC-20 token, either a Polymesh native asset (through the precompile) or
/// a plain Solidity token.
///
/// Both are just an address behind the same ABI, so tests can drive either one
/// through this type. Read-only methods go through `eth_call`, state-changing
/// methods take a [`ContractCaller`] so the same scenario can be run from a
/// Substrate signer or an Ethereum wallet.
#[derive(Clone)]
pub struct Token {
    pub node: EthNode,
    pub address: Address,
}

impl Token {
    /// Wraps the token deployed at `address`.
    pub fn new(node: &EthNode, address: Address) -> Self {
        Self {
            node: node.clone(),
            address,
        }
    }

    /// The token address as a `polymesh-api` `H160`.
    pub fn h160(&self) -> H160 {
        to_h160(&self.address)
    }

    // --- reads -------------------------------------------------------------

    pub async fn name(&self) -> Result<String> {
        self.node.call(self.address, &ierc20::nameCall {}).await
    }

    pub async fn symbol(&self) -> Result<String> {
        self.node.call(self.address, &ierc20::symbolCall {}).await
    }

    pub async fn decimals(&self) -> Result<u8> {
        self.node.call(self.address, &ierc20::decimalsCall {}).await
    }

    pub async fn total_supply(&self) -> Result<u128> {
        let value = self
            .node
            .call(self.address, &ierc20::totalSupplyCall {})
            .await?;
        Ok(value.try_into()?)
    }

    pub async fn balance_of(&self, account: Address) -> Result<u128> {
        let value = self
            .node
            .call(self.address, &ierc20::balanceOfCall { account })
            .await?;
        Ok(value.try_into()?)
    }

    pub async fn allowance(&self, owner: Address, spender: Address) -> Result<u128> {
        let value = self
            .node
            .call(self.address, &ierc20::allowanceCall { owner, spender })
            .await?;
        Ok(value.try_into()?)
    }

    /// Runs a typed call against the token without submitting it, so that tests
    /// can assert on revert reasons.
    pub async fn try_call<C: SolCall>(&self, from: Address, call: &C) -> Result<C::Return> {
        self.node.call_from(from, self.address, call).await
    }

    // --- writes ------------------------------------------------------------

    pub async fn transfer(
        &self,
        caller: &mut dyn ContractCaller,
        to: Address,
        value: u128,
    ) -> Result<Vec<ContractLog>> {
        self.send(
            caller,
            ierc20::transferCall {
                to,
                value: U256::from(value),
            },
        )
        .await
    }

    pub async fn approve(
        &self,
        caller: &mut dyn ContractCaller,
        spender: Address,
        value: u128,
    ) -> Result<Vec<ContractLog>> {
        self.send(
            caller,
            ierc20::approveCall {
                spender,
                value: U256::from(value),
            },
        )
        .await
    }

    pub async fn transfer_from(
        &self,
        caller: &mut dyn ContractCaller,
        from: Address,
        to: Address,
        value: u128,
    ) -> Result<Vec<ContractLog>> {
        self.send(
            caller,
            ierc20::transferFromCall {
                from,
                to,
                value: U256::from(value),
            },
        )
        .await
    }

    /// Submits `call` to this token and returns the logs it emitted.
    pub async fn send<C: SolCall>(
        &self,
        caller: &mut dyn ContractCaller,
        call: C,
    ) -> Result<Vec<ContractLog>> {
        caller.send_call(self.h160(), call.abi_encode()).await
    }
}

/// The ERC-20 precompile for a Polymesh native asset.
///
/// Derefs to the shared [`Token`] interface and adds the Polymesh-specific
/// `mint` and `burn` methods.
#[derive(Clone)]
pub struct Erc20Asset {
    pub api: Api,
    pub asset_id: AssetId,
    pub index: u32,
    pub token: Token,
}

impl std::ops::Deref for Erc20Asset {
    type Target = Token;

    fn deref(&self) -> &Token {
        &self.token
    }
}

impl Erc20Asset {
    /// Resolves the precompile for an existing asset.
    pub async fn new(api: &Api, node: &EthNode, asset_id: AssetId) -> Result<Self> {
        let index = asset_index(api, asset_id)
            .await?
            .ok_or_else(|| anyhow!("asset {asset_id:?} has no asset index"))?;
        Ok(Self {
            api: api.clone(),
            asset_id,
            index,
            token: Token::new(node, to_eth_address(&precompile_address(index))),
        })
    }

    /// Issues `value` tokens to the caller. The caller must be the asset owner.
    pub async fn mint(
        &self,
        caller: &mut dyn ContractCaller,
        value: u128,
    ) -> Result<Vec<ContractLog>> {
        self.send(
            caller,
            ierc20::mintCall {
                value: U256::from(value),
            },
        )
        .await
    }

    /// Redeems `value` tokens from the caller's balance.
    pub async fn burn(
        &self,
        caller: &mut dyn ContractCaller,
        value: u128,
    ) -> Result<Vec<ContractLog>> {
        self.send(
            caller,
            ierc20::burnCall {
                value: U256::from(value),
            },
        )
        .await
    }
}
