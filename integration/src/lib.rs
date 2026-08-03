// re-export from polymesh-api-tester.
pub use polymesh_api_tester::extras::*;
pub use polymesh_api_tester::*;

use std::collections::{BTreeMap, BTreeSet};

use polymesh_api::types::polymesh_primitives::{
    identity_id::PortfolioId,
    secondary_key::{ExtrinsicPermissions, PalletPermissions},
    subset::SubsetRestriction,
    ExtrinsicName, PalletName,
};
use polymesh_api::*;

use anyhow::{anyhow, Result};

mod asset_helper;
pub use asset_helper::*;

#[cfg(feature = "current_release")]
pub mod confidential_assets_helper;

#[cfg(feature = "current_release")]
pub mod contracts;

#[cfg(feature = "current_release")]
mod identity_helper;
#[cfg(feature = "current_release")]
pub use identity_helper::*;

#[cfg(feature = "current_release")]
mod revive_helper;
#[cfg(feature = "current_release")]
pub use revive_helper::*;

#[cfg(feature = "current_release")]
mod eth_helper;
#[cfg(feature = "current_release")]
pub use eth_helper::*;

#[cfg(feature = "current_release")]
mod erc20_helper;
#[cfg(feature = "current_release")]
pub use erc20_helper::*;

#[cfg(feature = "current_release")]
mod swap_helper;
#[cfg(feature = "current_release")]
pub use swap_helper::*;

#[cfg(any(feature = "previous_release", feature = "current_release"))]
mod sto;
#[cfg(any(feature = "previous_release", feature = "current_release"))]
pub use sto::*;

pub async fn wait_for_results(tx_res: &mut TransactionResults) -> Result<()> {
    let finalize = option_env!("WAIT_FOR_FINALIZE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    // Wait for transaction to execute.
    if finalize {
        tx_res.wait_finalized().await?;
    } else {
        tx_res.wait_in_block().await?;
    }

    Ok(())
}

pub async fn get_batch_results(res: &mut TransactionResults) -> Result<Vec<bool>> {
    let events = res
        .events()
        .await?
        .ok_or_else(|| anyhow!("Failed to get batch events"))?;
    Ok(events
        .0
        .iter()
        .filter_map(|rec| match rec.event {
            RuntimeEvent::Utility(UtilityEvent::ItemCompleted) => Some(true),
            RuntimeEvent::Utility(UtilityEvent::ItemFailed { .. }) => Some(false),
            _ => None,
        })
        .collect())
}

#[derive(Clone, Default, PartialEq, Eq)]
pub enum RestrictionMode {
    #[default]
    Whole,
    These,
    Except,
}

#[derive(Clone)]
pub struct SubsetBuilder<T> {
    mode: RestrictionMode,
    entries: Option<BTreeSet<T>>,
}

impl<T> Default for SubsetBuilder<T> {
    fn default() -> Self {
        Self {
            mode: Default::default(),
            entries: None,
        }
    }
}

impl<T: Clone + Ord> SubsetBuilder<T> {
    fn whole() -> Self {
        Self {
            mode: RestrictionMode::Whole,
            entries: None,
        }
    }

    fn empty() -> Self {
        Self {
            mode: RestrictionMode::These,
            entries: Some(Default::default()),
        }
    }

    pub fn set(&mut self, entries: &[T], these: bool) {
        if these {
            self.mode = RestrictionMode::These;
        } else {
            self.mode = RestrictionMode::Except;
        }
        self.entries = Some(entries.iter().cloned().collect())
    }

    pub fn add(&mut self, entry: &T) {
        let entries = self.entries.get_or_insert(Default::default());
        entries.insert(entry.clone());
    }

    pub fn build(&self) -> SubsetRestriction<T> {
        match &self.mode {
            RestrictionMode::Whole => SubsetRestriction::Whole,
            RestrictionMode::These => {
                SubsetRestriction::These(self.entries.clone().unwrap_or_default())
            }
            RestrictionMode::Except => {
                SubsetRestriction::Except(self.entries.clone().unwrap_or_default())
            }
        }
    }
}

#[derive(Clone, Default)]
pub struct PalletPermissionsBuilder {
    mode: RestrictionMode,
    entries: Option<BTreeMap<String, SubsetBuilder<String>>>,
}

impl PalletPermissionsBuilder {
    fn whole() -> Self {
        Self {
            mode: RestrictionMode::Whole,
            entries: None,
        }
    }

    fn empty() -> Self {
        Self {
            mode: RestrictionMode::These,
            entries: Some(Default::default()),
        }
    }

    pub fn set(&mut self, extrinsics: &[(String, String)], these: bool) {
        if these {
            self.mode = RestrictionMode::These;
        } else {
            self.mode = RestrictionMode::Except;
        }
        let entries = self.entries.get_or_insert(Default::default());
        for (pallet, extrinsic) in extrinsics {
            let pallet = entries.entry(pallet.clone()).or_default();
            pallet.add(extrinsic);
        }
    }

    fn allow_pallet(&mut self, pallet: &str) {
        if self.mode != RestrictionMode::These {
            self.mode = RestrictionMode::These;
            self.entries = None;
        }
        let entries = self.entries.get_or_insert(Default::default());
        let pallet = entries.entry(pallet.to_string()).or_default();
        *pallet = SubsetBuilder::whole();
    }

    fn allow_extrinsic(&mut self, pallet: &str, extrinsic: &str) {
        if self.mode != RestrictionMode::These {
            self.mode = RestrictionMode::These;
            self.entries = None;
        }
        let entries = self.entries.get_or_insert(Default::default());
        let pallet = entries.entry(pallet.to_string()).or_default();
        pallet.add(&extrinsic.to_string());
    }

    fn build_entries(&self) -> BTreeMap<PalletName, PalletPermissions> {
        if let Some(entries) = &self.entries {
            entries
                .iter()
                .map(|(pallet, extrinsics)| {
                    let extrinsics = match extrinsics.build() {
                        SubsetRestriction::Whole => SubsetRestriction::Whole,
                        SubsetRestriction::These(names) => SubsetRestriction::These(
                            names.into_iter().map(|n| ExtrinsicName(n.into())).collect(),
                        ),
                        SubsetRestriction::Except(names) => SubsetRestriction::Except(
                            names.into_iter().map(|n| ExtrinsicName(n.into())).collect(),
                        ),
                    };
                    (PalletName(pallet.into()), PalletPermissions { extrinsics })
                })
                .collect()
        } else {
            Default::default()
        }
    }

    pub fn build(&self) -> ExtrinsicPermissions {
        match &self.mode {
            RestrictionMode::Whole => ExtrinsicPermissions::Whole,
            RestrictionMode::These => ExtrinsicPermissions::These(self.build_entries()),
            RestrictionMode::Except => ExtrinsicPermissions::Except(self.build_entries()),
        }
    }
}

#[derive(Clone, Default)]
pub struct PermissionsBuilder {
    asset: SubsetBuilder<AssetId>,
    portfolio: SubsetBuilder<PortfolioId>,
    extrinsic: PalletPermissionsBuilder,
}

impl PermissionsBuilder {
    pub fn whole() -> Self {
        Self {
            asset: SubsetBuilder::whole(),
            portfolio: SubsetBuilder::whole(),
            extrinsic: PalletPermissionsBuilder::whole(),
        }
    }

    pub fn empty() -> Self {
        Self {
            asset: SubsetBuilder::empty(),
            portfolio: SubsetBuilder::empty(),
            extrinsic: PalletPermissionsBuilder::empty(),
        }
    }

    pub fn set_asset(&mut self, assets: &[AssetId], these: bool) {
        self.asset.set(assets, these);
    }

    pub fn set_portfolio(&mut self, portfolios: &[PortfolioId], these: bool) {
        self.portfolio.set(portfolios, these);
    }

    pub fn set_extrinsic(&mut self, extrinsics: &[(String, String)], these: bool) {
        self.extrinsic.set(extrinsics, these);
    }

    pub fn allow_pallet(&mut self, pallet: &str) {
        self.extrinsic.allow_pallet(pallet);
    }

    pub fn allow_extrinsic(&mut self, pallet: &str, extrinsic: &str) {
        self.extrinsic.allow_extrinsic(pallet, extrinsic);
    }

    pub fn clear_asset(&mut self) {
        self.asset = SubsetBuilder::empty()
    }

    pub fn clear_portfolio(&mut self) {
        self.portfolio = SubsetBuilder::empty()
    }

    pub fn clear_extrinsic(&mut self) {
        self.extrinsic = PalletPermissionsBuilder::empty()
    }

    pub fn build(&self) -> Permissions {
        Permissions {
            asset: self.asset.build(),
            portfolio: self.portfolio.build(),
            extrinsic: self.extrinsic.build(),
        }
    }
}

impl From<&PermissionsBuilder> for Permissions {
    fn from(builder: &PermissionsBuilder) -> Self {
        builder.build()
    }
}

impl From<PermissionsBuilder> for Permissions {
    fn from(builder: PermissionsBuilder) -> Self {
        builder.build()
    }
}

pub async fn get_auth_id(res: &mut TransactionResults) -> Result<Option<u64>> {
    if let Some(events) = res.events().await? {
        for rec in &events.0 {
            match &rec.event {
                RuntimeEvent::Identity(IdentityEvent::AuthorizationAdded(_, _, _, auth, _, _)) => {
                    return Ok(Some(*auth));
                }
                _ => (),
            }
        }
    }
    Ok(None)
}

#[cfg(feature = "current_release")]
pub async fn get_ag_id(
    res: &mut TransactionResults,
) -> Result<Option<polymesh_api::types::polymesh_primitives::agent::AGId>> {
    if let Some(events) = res.events().await? {
        for rec in &events.0 {
            match &rec.event {
                RuntimeEvent::ExternalAgents(ExternalAgentsEvent::GroupCreated(_, _, ag_id, _)) => {
                    return Ok(Some(ag_id.clone()));
                }
                _ => (),
            }
        }
    }
    Ok(None)
}

#[cfg(feature = "current_release")]
pub async fn get_ca_id(
    res: &mut TransactionResults,
) -> Result<Option<polymesh_api::types::pallet_corporate_actions::CAId>> {
    if let Some(events) = res.events().await? {
        for rec in &events.0 {
            match &rec.event {
                RuntimeEvent::CorporateAction(CorporateActionEvent::CAInitiated(
                    _,
                    ca_id,
                    _,
                    _,
                )) => {
                    return Ok(Some(ca_id.clone()));
                }
                _ => (),
            }
        }
    }
    Ok(None)
}

/// Helper trait to add methods to `User`
#[async_trait::async_trait]
pub trait IntegrationUser: Signer {
    fn get_sk(&self, sk: usize) -> Result<&AccountSigner>;

    fn get_sk_count(&self) -> usize;

    fn get_sk_mut(&mut self, sk: usize) -> Result<&mut AccountSigner>;

    async fn set_key_permissions(
        &mut self,
        sk: usize,
        permissions: impl Into<Permissions> + Send,
    ) -> Result<TransactionResults>;

    async fn set_all_keys_permissions(
        &mut self,
        permissions: impl Into<Permissions> + Send,
    ) -> Result<()> {
        let permissions = permissions.into();
        let sk_count = self.get_sk_count();
        let mut results = Vec::new();
        for sk in 0..sk_count {
            results.push(self.set_key_permissions(sk, permissions.clone()).await?);
        }
        for mut res in results {
            res.ok().await?;
        }
        Ok(())
    }

    async fn ensure_key_permissions(
        &self,
        sk: usize,
        permissions: impl Into<Permissions> + Send,
    ) -> Result<()>;
}

#[async_trait::async_trait]
impl IntegrationUser for User {
    fn get_sk(&self, sk: usize) -> Result<&AccountSigner> {
        self.secondary_keys
            .get(sk)
            .ok_or_else(|| anyhow!("Missing secondary key: {sk}"))
    }

    fn get_sk_count(&self) -> usize {
        self.secondary_keys.len()
    }

    fn get_sk_mut(&mut self, sk: usize) -> Result<&mut AccountSigner> {
        self.secondary_keys
            .get_mut(sk)
            .ok_or_else(|| anyhow!("Missing secondary key: {sk}"))
    }

    async fn set_key_permissions(
        &mut self,
        sk: usize,
        permissions: impl Into<Permissions> + Send,
    ) -> Result<TransactionResults> {
        let permissions = permissions.into();
        let sk = self.get_sk(sk)?.account();
        let res = self
            .api
            .call()
            .identity()
            .set_secondary_key_permissions(sk, permissions)?
            .submit_and_watch(self)
            .await?;
        Ok(res)
    }

    async fn ensure_key_permissions(
        &self,
        sk: usize,
        permissions: impl Into<Permissions> + Send,
    ) -> Result<()> {
        let permissions = permissions.into();
        let sk = self.get_sk(sk)?.account();
        let asset = self
            .api
            .query()
            .identity()
            .key_asset_permissions(sk)
            .await?
            .ok_or_else(|| anyhow!("Missing asset permissions"))?;
        assert_eq!(permissions.asset, asset);
        let portfolio = self
            .api
            .query()
            .identity()
            .key_portfolio_permissions(sk)
            .await?
            .ok_or_else(|| anyhow!("Missing portfolio permissions"))?;
        assert_eq!(permissions.portfolio, portfolio);
        let extrinsic = self
            .api
            .query()
            .identity()
            .key_extrinsic_permissions(sk)
            .await?
            .ok_or_else(|| anyhow!("Missing extrinsic permissions"))?;
        assert_eq!(permissions.extrinsic, extrinsic);
        Ok(())
    }
}

/// A helper function to sign a SCALE encoded message with a signing key.
pub async fn sign_with_key<S: Signer, T: sp_core::Encode>(
    signer: &S,
    message: &T,
) -> Result<sp_runtime::MultiSignature> {
    let encoded = {
        use crate::client::{BytesPayload, Encoded};
        use sp_core::Encode;

        BytesPayload(Encoded::from(message)).encode()
    };
    Ok(signer.sign(&encoded[..]).await?)
}
