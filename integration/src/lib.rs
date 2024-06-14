// re-export from polymesh-api-tester.
pub use polymesh_api_tester::extras::*;
pub use polymesh_api_tester::*;

use polymesh_api::types::polymesh_primitives::{
    identity_id::PortfolioId, secondary_key::PalletPermissions, subset::SubsetRestriction,
    DispatchableName, PalletName,
};
use polymesh_api::*;

use anyhow::{anyhow, Result};

/// Helper trait to add methods to `Permissions`
pub trait IntegrationPermissions {
    fn whole() -> Self;
    fn empty() -> Self;
    fn set_asset(&mut self, assets: &[Ticker], these: bool);
    fn set_portfolio(&mut self, portfolios: &[PortfolioId], these: bool);
    fn set_extrinsic(&mut self, pallets: &[PalletPermissions], these: bool);
    fn allow_extrinsic(&mut self, pallet: &str, extrinsic: &str);
    fn clear_asset(&mut self);
    fn clear_portfolio(&mut self);
    fn clear_extrinsic(&mut self);
    fn clear_all(&mut self) {
        self.clear_asset();
        self.clear_portfolio();
        self.clear_extrinsic();
    }
}

impl IntegrationPermissions for Permissions {
    fn whole() -> Self {
        Permissions {
            asset: SubsetRestriction::Whole,
            extrinsic: SubsetRestriction::Whole,
            portfolio: SubsetRestriction::Whole,
        }
    }

    fn empty() -> Permissions {
        Permissions {
            asset: SubsetRestriction::These(Default::default()),
            extrinsic: SubsetRestriction::These(Default::default()),
            portfolio: SubsetRestriction::These(Default::default()),
        }
    }

    fn set_asset(&mut self, assets: &[Ticker], these: bool) {
        if these {
            self.asset = SubsetRestriction::These(assets.iter().cloned().collect())
        } else {
            self.asset = SubsetRestriction::Except(assets.iter().cloned().collect())
        }
    }

    fn set_portfolio(&mut self, portfolios: &[PortfolioId], these: bool) {
        if these {
            self.portfolio = SubsetRestriction::These(portfolios.iter().cloned().collect())
        } else {
            self.portfolio = SubsetRestriction::Except(portfolios.iter().cloned().collect())
        }
    }

    fn set_extrinsic(&mut self, extrinsics: &[PalletPermissions], these: bool) {
        if these {
            self.extrinsic = SubsetRestriction::These(extrinsics.iter().cloned().collect())
        } else {
            self.extrinsic = SubsetRestriction::Except(extrinsics.iter().cloned().collect())
        }
    }

    fn allow_extrinsic(&mut self, pallet: &str, extrinsic: &str) {
        let pallet_name = PalletName(pallet.as_bytes().into());
        let dispatchable = DispatchableName(extrinsic.as_bytes().into());
        if let SubsetRestriction::These(pallets) = &mut self.extrinsic {
            // Check if the pallet is already in the set.
            for pallet in pallets.clone() {
                if pallet.pallet_name == pallet_name {
                    // Found it.
                    let mut new_pallet = pallet.clone();
                    if let SubsetRestriction::These(dispatchables) =
                        &mut new_pallet.dispatchable_names
                    {
                        dispatchables.insert(dispatchable);
                    } else {
                        new_pallet.dispatchable_names =
                            SubsetRestriction::These([dispatchable].into());
                    }
                    pallets.remove(&pallet);
                    pallets.insert(new_pallet);
                    return;
                }
            }
            // Need to add the pallet.
            pallets.insert(PalletPermissions {
                pallet_name,
                dispatchable_names: SubsetRestriction::These([dispatchable].into()),
            });
        } else {
            // Convert from `Whole` or `Except`.
            self.extrinsic = SubsetRestriction::These(
                [PalletPermissions {
                    pallet_name,
                    dispatchable_names: SubsetRestriction::These([dispatchable].into()),
                }]
                .into(),
            );
        }
    }

    fn clear_asset(&mut self) {
        self.asset = SubsetRestriction::These(Default::default())
    }

    fn clear_portfolio(&mut self) {
        self.portfolio = SubsetRestriction::These(Default::default())
    }

    fn clear_extrinsic(&mut self) {
        self.extrinsic = SubsetRestriction::These(Default::default())
    }
}

/// Helper trait to add methods to `User`
#[async_trait::async_trait]
pub trait IntegrationUser: Signer {
    fn get_sk(&self, sk: usize) -> Result<&AccountSigner>;

    fn get_sk_mut(&mut self, sk: usize) -> Result<&mut AccountSigner>;

    async fn set_key_permissions(
        &mut self,
        sk: usize,
        permissions: &Permissions,
    ) -> Result<TransactionResults>;

    async fn create_child_identity(&mut self, sk: usize) -> Result<User>;
}

#[async_trait::async_trait]
impl IntegrationUser for User {
    fn get_sk(&self, sk: usize) -> Result<&AccountSigner> {
        self.secondary_keys
            .get(sk)
            .ok_or_else(|| anyhow!("Missing secondary key: {sk}"))
    }

    fn get_sk_mut(&mut self, sk: usize) -> Result<&mut AccountSigner> {
        self.secondary_keys
            .get_mut(sk)
            .ok_or_else(|| anyhow!("Missing secondary key: {sk}"))
    }

    async fn set_key_permissions(
        &mut self,
        sk: usize,
        permissions: &Permissions,
    ) -> Result<TransactionResults> {
        let sk = self.get_sk(sk)?.account();
        let res = self
            .api
            .call()
            .identity()
            .set_secondary_key_permissions(sk, permissions.clone())?
            .submit_and_watch(self)
            .await?;
        Ok(res)
    }

    async fn create_child_identity(&mut self, sk: usize) -> Result<User> {
        if sk >= self.secondary_keys.len() {
            return Err(anyhow!("Missing secondary key: {sk}"));
        }
        let sk = self.secondary_keys.remove(sk);
        let mut res = self
            .api
            .call()
            .identity()
            .create_child_identity(sk.account())?
            .submit_and_watch(self)
            .await?;
        let did = get_identity_id(&mut res)
            .await?
            .ok_or_else(|| anyhow!("Failed to create child identity"))?;
        let mut child = User::new(&self.api, sk);
        child.did = Some(did);
        Ok(child)
    }
}
