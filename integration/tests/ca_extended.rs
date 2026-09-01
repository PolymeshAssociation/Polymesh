//! Extra corporate-action configuration: record dates, docs, withholding tax, targets.
#[cfg(feature = "current_release")]
mod ca_extended_tests {
    use std::collections::BTreeSet;

    use anyhow::Result;

    use integration::*;
    use polymesh_api::types::pallet_corporate_actions::{
        CADetails, CAKind, RecordDateSpec, TargetIdentities, TargetTreatment,
    };
    use polymesh_api::types::polymesh_primitives::{
        asset::AssetHolderKind,
        document::{Document, DocumentId, DocumentName, DocumentUri},
    };

    async fn create_asset(
        tester: &mut PolymeshTester,
        owner: &mut User,
        ticker: &str,
        amount: u128,
    ) -> Result<AssetId> {
        let helper = AssetHelper::new_full(
            &tester.api,
            owner,
            ticker,
            amount,
            BTreeSet::new(),
            false,
            Some(AssetHolderKind::DefaultPortfolio),
        )
        .await?;
        Ok(helper.asset_id)
    }

    /// change_record_date can bind a CA to an existing checkpoint.
    #[tokio::test]
    #[test_log::test]
    async fn change_record_date_to_checkpoint() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["CaeOwner"]).await?.into_iter();
        let mut owner = users.next().unwrap();

        let asset_id = create_asset(&mut tester, &mut owner, "CAERD", 1_000_000).await?;
        let now = tester.api.query().timestamp().now().await?;
        let mut ca_res = tester
            .api
            .call()
            .corporate_action()
            .initiate_corporate_action(
                asset_id.clone(),
                CAKind::IssuerNotice,
                now,
                None,
                CADetails(b"rd".to_vec()),
                None,
                None,
                None,
            )?
            .submit_and_watch(&mut owner)
            .await?;
        ca_res.ok().await?;
        let ca_id = get_ca_id(&mut ca_res).await?.expect("ca id");

        let mut cp_res = tester
            .api
            .call()
            .checkpoint()
            .create_checkpoint(asset_id)?
            .submit_and_watch(&mut owner)
            .await?;
        cp_res.ok().await?;
        let cp_id = get_checkpoint_id(&mut cp_res).await?.expect("cp id");

        tester
            .api
            .call()
            .corporate_action()
            .change_record_date(ca_id, Some(RecordDateSpec::Existing(cp_id)))?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        Ok(())
    }

    /// Documents can be attached to a CA.
    #[tokio::test]
    #[test_log::test]
    async fn link_ca_document() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["CaeDocOwner"]).await?.into_iter();
        let mut owner = users.next().unwrap();

        let asset_id = create_asset(&mut tester, &mut owner, "CAEDOC", 1_000_000).await?;
        let doc = Document {
            uri: DocumentUri(b"ipfs://ca-doc".to_vec()),
            content_hash: polymesh_api::types::polymesh_primitives::document_hash::DocumentHash::None,
            name: DocumentName(b"Notice".to_vec()),
            doc_type: None,
            filing_date: None,
        };
        tester
            .api
            .call()
            .asset()
            .add_documents(vec![doc], asset_id.clone())?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        let now = tester.api.query().timestamp().now().await?;
        let mut ca_res = tester
            .api
            .call()
            .corporate_action()
            .initiate_corporate_action(
                asset_id,
                CAKind::IssuerNotice,
                now,
                None,
                CADetails(b"docs".to_vec()),
                None,
                None,
                None,
            )?
            .submit_and_watch(&mut owner)
            .await?;
        ca_res.ok().await?;
        let ca_id = get_ca_id(&mut ca_res).await?.expect("ca id");

        tester
            .api
            .call()
            .corporate_action()
            .link_ca_doc(ca_id, vec![DocumentId(0)])?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        Ok(())
    }

    /// Default and per-DID withholding tax plus default targets.
    #[tokio::test]
    #[test_log::test]
    async fn withholding_tax_and_targets() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester
            .users(&["CaeTaxOwner", "CaeHolder"])
            .await?
            .into_iter();
        let mut owner = users.next().unwrap();
        let holder = users.next().unwrap();

        let asset_id = create_asset(&mut tester, &mut owner, "CAETAX", 1_000_000).await?;
        let permill = sp_arithmetic::per_things::Permill::from_percent(10);
        tester
            .api
            .call()
            .corporate_action()
            .set_default_withholding_tax(asset_id.clone(), permill.into())?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        let higher = sp_arithmetic::per_things::Permill::from_percent(25);
        tester
            .api
            .call()
            .corporate_action()
            .set_did_withholding_tax(
                asset_id.clone(),
                holder.did.expect("holder did"),
                Some(higher.into()),
            )?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        tester
            .api
            .call()
            .corporate_action()
            .set_default_targets(
                asset_id,
                TargetIdentities {
                    identities: vec![holder.did.expect("holder did")],
                    treatment: TargetTreatment::Include,
                },
            )?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        Ok(())
    }
}