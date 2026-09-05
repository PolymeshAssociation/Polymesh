//! Utility pallet: batch vs batch_all, as_derivative.
#[cfg(feature = "current_release")]
mod utility_calls_tests {
    use anyhow::Result;

    use integration::*;

    /// `batch` continues after an item failure; `batch_all` is atomic.
    #[tokio::test]
    #[test_log::test]
    async fn batch_vs_batch_all() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["UtilUser"]).await?.into_iter();
        let mut user = users.next().unwrap();

        let ok_call = tester
            .api
            .call()
            .system()
            .remark(b"ok".to_vec())?
            .into_runtime_call();

        // Freeze a non-existent asset so this item fails at dispatch.
        let fail_call = tester
            .api
            .call()
            .asset()
            .freeze(AssetId([0u8; 16]))?
            .into_runtime_call();

        // force_batch records per-item success/failure without aborting.
        let mut batch_res = tester
            .api
            .call()
            .utility()
            .force_batch(vec![ok_call.clone(), fail_call.clone()])?
            .submit_and_watch(&mut user)
            .await?;
        let results = get_batch_results(&mut batch_res).await?;
        assert_eq!(
            results,
            vec![true, false],
            "force_batch continues after a failure"
        );

        // batch_all: atomic — the whole call fails.
        let mut all_res = tester
            .api
            .call()
            .utility()
            .batch_all(vec![ok_call, fail_call])?
            .submit_and_watch(&mut user)
            .await?;
        assert!(
            all_res.ok().await.is_err(),
            "batch_all must fail if any item fails"
        );

        Ok(())
    }

    /// `as_derivative` dispatches as a derived sub-account.
    #[tokio::test]
    #[test_log::test]
    async fn as_derivative_remark() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["UtilDeriv"]).await?.into_iter();
        let mut user = users.next().unwrap();

        let call = tester
            .api
            .call()
            .system()
            .remark(b"derivative".to_vec())?
            .into_runtime_call();

        tester
            .api
            .call()
            .utility()
            .as_derivative(0, call)?
            .submit_and_watch(&mut user)
            .await?
            .ok()
            .await?;

        Ok(())
    }
}
