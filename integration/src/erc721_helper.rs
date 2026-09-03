//! Helpers for the Polymesh native-NFT ERC-721 precompile.
//!
//! Every Polymesh NFT collection is reachable from the EVM at a deterministic address derived
//! from its 16-byte asset id, where `pallet_precompiles` exposes an ERC-721 interface backed by
//! `pallet_nft`. The ERC-721 `tokenId` is the on-chain `NFTId`.

use anyhow::Result;
use codec::Encode;

use alloy::primitives::{Address, U256};
use alloy::sol_types::SolCall;

use polymesh_api::types::polymesh_primitives::asset::{AssetName, AssetType, NonFungibleType};
use polymesh_api::types::polymesh_primitives::nft::NFTCollectionKeys;
use polymesh_api::types::primitive_types::H160;
use polymesh_precompiles::INonFungibleAsset as ierc721;

use crate::*;

/// The `AddressMatcher` id used by `NonFungibleAssetInterface`.
pub const POLYMESH_NFT_PRECOMPILE_PREFIX: u16 = 9;

/// The precompile address for the NFT collection `asset_id`.
///
/// Same layout as the fungible precompile, with a different matcher id:
///
/// ```text
/// xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxpppp0000
/// ^ asset id (16 bytes)           ^ matcher (BE)
/// ```
pub fn nft_precompile_address(asset_id: AssetId) -> H160 {
    let mut address = [0u8; 20];
    let encoded = asset_id.encode();
    let bytes: [u8; 16] = encoded
        .try_into()
        .expect("AssetId scale encoding is 16 bytes; qed");
    address[0..16].copy_from_slice(&bytes);
    address[16..18].copy_from_slice(&POLYMESH_NFT_PRECOMPILE_PREFIX.to_be_bytes());
    H160(address)
}

/// Creates an NFT collection with no mandatory metadata keys and returns its ERC-721 precompile.
pub async fn create_erc721_collection(
    api: &Api,
    node: &EthNode,
    owner: &mut User,
    name: &str,
) -> Result<Erc721Collection> {
    let mut res = api
        .call()
        .asset()
        .create_asset(
            AssetName(name.as_bytes().to_vec()),
            false,
            AssetType::NonFungible(NonFungibleType::Derivative),
            vec![],
            None,
        )?
        .submit_and_watch(owner)
        .await?;
    res.ok().await?;
    let asset_id = get_asset_id(&mut res)
        .await?
        .ok_or_else(|| anyhow::anyhow!("AssetCreated event missing"))?;

    api.call()
        .nft()
        .create_nft_collection(Some(asset_id), None, NFTCollectionKeys(vec![]))?
        .execute(owner)
        .await?
        .ok()
        .await?;

    Erc721Collection::new(api, node, asset_id).await
}

/// The ERC-721 precompile for a Polymesh NFT collection.
#[derive(Clone)]
pub struct Erc721Collection {
    pub api: Api,
    pub asset_id: AssetId,
    pub node: EthNode,
    pub address: Address,
}

impl Erc721Collection {
    /// Resolves the precompile for an existing collection.
    pub async fn new(api: &Api, node: &EthNode, asset_id: AssetId) -> Result<Self> {
        Ok(Self {
            api: api.clone(),
            asset_id,
            node: node.clone(),
            address: to_eth_address(&nft_precompile_address(asset_id)),
        })
    }

    /// The collection address as a `polymesh-api` `H160`.
    pub fn h160(&self) -> H160 {
        to_h160(&self.address)
    }

    // --- reads -------------------------------------------------------------

    pub async fn name(&self) -> Result<String> {
        self.node.call(self.address, &ierc721::nameCall {}).await
    }

    pub async fn symbol(&self) -> Result<String> {
        self.node.call(self.address, &ierc721::symbolCall {}).await
    }

    pub async fn token_uri(&self, token_id: u64) -> Result<String> {
        self.node
            .call(
                self.address,
                &ierc721::tokenURICall {
                    tokenId: U256::from(token_id),
                },
            )
            .await
    }

    pub async fn total_supply(&self) -> Result<u128> {
        let value = self
            .node
            .call(self.address, &ierc721::totalSupplyCall {})
            .await?;
        Ok(value.try_into()?)
    }

    pub async fn balance_of(&self, owner: Address) -> Result<u128> {
        let value = self
            .node
            .call(self.address, &ierc721::balanceOfCall { owner })
            .await?;
        Ok(value.try_into()?)
    }

    pub async fn owner_of(&self, token_id: u64) -> Result<Address> {
        self.node
            .call(
                self.address,
                &ierc721::ownerOfCall {
                    tokenId: U256::from(token_id),
                },
            )
            .await
    }

    pub async fn get_approved(&self, token_id: u64) -> Result<Address> {
        self.node
            .call(
                self.address,
                &ierc721::getApprovedCall {
                    tokenId: U256::from(token_id),
                },
            )
            .await
    }

    pub async fn is_approved_for_all(&self, owner: Address, operator: Address) -> Result<bool> {
        self.node
            .call(
                self.address,
                &ierc721::isApprovedForAllCall { owner, operator },
            )
            .await
    }

    pub async fn supports_interface(&self, interface_id: [u8; 4]) -> Result<bool> {
        self.node
            .call(
                self.address,
                &ierc721::supportsInterfaceCall {
                    interfaceId: interface_id.into(),
                },
            )
            .await
    }

    pub async fn can_transfer(&self, from: Address, to: Address, token_id: u64) -> Result<bool> {
        self.node
            .call(
                self.address,
                &ierc721::canTransferCall {
                    from,
                    to,
                    tokenId: U256::from(token_id),
                },
            )
            .await
    }

    pub async fn can_send(&self, account: Address) -> Result<bool> {
        self.node
            .call(self.address, &ierc721::canSendCall { account })
            .await
    }

    pub async fn can_receive(&self, account: Address) -> Result<bool> {
        self.node
            .call(self.address, &ierc721::canReceiveCall { account })
            .await
    }

    /// Runs a typed call without submitting it, so tests can assert on revert reasons.
    pub async fn try_call<C: SolCall>(&self, from: Address, call: &C) -> Result<C::Return> {
        self.node.call_from(from, self.address, call).await
    }

    // --- writes ------------------------------------------------------------

    /// Submits `call` to this collection and returns the logs it emitted.
    pub async fn send<C: SolCall>(
        &self,
        caller: &mut dyn ContractCaller,
        call: C,
    ) -> Result<Vec<ContractLog>> {
        caller.send_call(self.h160(), call.abi_encode()).await
    }

    /// Issues a new NFT to the caller's account key.
    pub async fn mint(
        &self,
        caller: &mut dyn ContractCaller,
        metadata_values: Vec<Vec<u8>>,
    ) -> Result<Vec<ContractLog>> {
        self.send(
            caller,
            ierc721::mintCall {
                metadataValues: metadata_values.into_iter().map(Into::into).collect(),
            },
        )
        .await
    }

    /// Redeems `token_id` from the caller's account key.
    pub async fn burn(
        &self,
        caller: &mut dyn ContractCaller,
        token_id: u64,
    ) -> Result<Vec<ContractLog>> {
        self.send(
            caller,
            ierc721::burnCall {
                tokenId: U256::from(token_id),
            },
        )
        .await
    }

    pub async fn transfer_from(
        &self,
        caller: &mut dyn ContractCaller,
        from: Address,
        to: Address,
        token_id: u64,
    ) -> Result<Vec<ContractLog>> {
        self.send(
            caller,
            ierc721::transferFromCall {
                from,
                to,
                tokenId: U256::from(token_id),
            },
        )
        .await
    }

    pub async fn safe_transfer_from(
        &self,
        caller: &mut dyn ContractCaller,
        from: Address,
        to: Address,
        token_id: u64,
    ) -> Result<Vec<ContractLog>> {
        self.send(
            caller,
            ierc721::safeTransferFrom_0Call {
                from,
                to,
                tokenId: U256::from(token_id),
            },
        )
        .await
    }

    pub async fn approve(
        &self,
        caller: &mut dyn ContractCaller,
        to: Address,
        token_id: u64,
    ) -> Result<Vec<ContractLog>> {
        self.send(
            caller,
            ierc721::approveCall {
                to,
                tokenId: U256::from(token_id),
            },
        )
        .await
    }

    pub async fn set_approval_for_all(
        &self,
        caller: &mut dyn ContractCaller,
        operator: Address,
        approved: bool,
    ) -> Result<Vec<ContractLog>> {
        self.send(
            caller,
            ierc721::setApprovalForAllCall { operator, approved },
        )
        .await
    }

    pub async fn forced_transfer(
        &self,
        caller: &mut dyn ContractCaller,
        from: Address,
        token_id: u64,
    ) -> Result<Vec<ContractLog>> {
        self.send(
            caller,
            ierc721::forcedTransferCall {
                from,
                tokenId: U256::from(token_id),
            },
        )
        .await
    }
}
