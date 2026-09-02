// >=v7.3
#[cfg(feature = "current_release")]
mod worker_modules_tests {
    use anyhow::Result;

    use integration::{worker_modules_helper::*, PolymeshTester};

    async fn upload_v1(helper: &mut WorkerModulesHelper) -> Result<()> {
        let polkavm = include_bytes!(
            "../../worker/protocol/testing/v1/polymesh-worker-protocol-testing.polkavm.zst"
        )
        .to_vec();
        let wasm = include_bytes!(
            "../../worker/protocol/testing/v1/polymesh-worker-protocol-testing.wasm.zst"
        )
        .to_vec();

        // Upload the module code and config.
        helper.update_version(ProtocolVersion {
            major: 1,
            minor: 0,
            patch: 0,
        });
        helper
            .upload_modules_and_config(
                ProtocolInitializationMethod::SaveContextFromFirstInstance,
                vec![
                    (BackendModuleKind::PolkaVM, 1, polkavm.clone()),
                    (BackendModuleKind::Wasm, 1, wasm.clone()),
                ],
            )
            .await?;
        Ok(())
    }

    async fn upload_v2(helper: &mut WorkerModulesHelper) -> Result<()> {
        let polkavm = include_bytes!(
            "../../worker/protocol/testing/v2/polymesh-worker-protocol-testing.polkavm.zst"
        )
        .to_vec();
        let wasm = include_bytes!(
            "../../worker/protocol/testing/v2/polymesh-worker-protocol-testing.wasm.zst"
        )
        .to_vec();

        // Upload the module code and config.
        helper.update_version(ProtocolVersion {
            major: 2,
            minor: 0,
            patch: 0,
        });
        helper
            .upload_modules_and_config(
                ProtocolInitializationMethod::SaveContextFromFirstInstance,
                vec![
                    (BackendModuleKind::PolkaVM, 1, polkavm.clone()),
                    (BackendModuleKind::Wasm, 1, wasm.clone()),
                ],
            )
            .await?;
        Ok(())
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_protocol_upgrades() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["ProtocolTester"]).await?.into_iter();
        let mut user = users.next().expect("User not found");

        // Get the current protocol version from the worker-testing pallet.
        let protocol = tester
            .api
            .query()
            .worker_testing()
            .current_protocol_version()
            .await?
            .expect("Current protocol version not found");
        let mut worker_helper = WorkerModulesHelper::new(&tester, protocol.clone());

        // Enable work session in the worker-testing pallet.
        worker_helper
            .sudo_call(
                tester
                    .api
                    .call()
                    .worker_testing()
                    .set_enable_work_session(true)?,
            )
            .await?;

        // Run the `VerifyVersion` work request.
        tester
            .api
            .call()
            .worker_testing()
            .test_version(protocol.clone())?
            .submit_and_watch(&mut user)
            .await?
            .wait_finalized()
            .await?;

        // Register the testing protocol with the WorkerModules pallet.
        worker_helper
            .register_protocol(
                "Testing",
                "Polymesh Worker Protocol Testing",
                protocol.version.clone(),
            )
            .await?;

        // Upload the v1 protocol modules and config.  This only uploads the new modules and config for the new version, and does not change the active protocol version.
        upload_v1(&mut worker_helper).await?;

        // Verify that the active protocol version is still the same.
        tester
            .api
            .call()
            .worker_testing()
            .test_version(protocol.clone())?
            .submit_and_watch(&mut user)
            .await?
            .wait_finalized()
            .await?;

        // Change the active protocol version to v1.0.0.
        let protocol_v1 = Protocol {
            id: protocol.id.clone(),
            version: ProtocolVersion {
                major: 1,
                minor: 0,
                patch: 0,
            },
        };
        worker_helper
            .sudo_call(
                tester
                    .api
                    .call()
                    .worker_testing()
                    .set_protocol_version(protocol_v1.clone())?,
            )
            .await?;

        // Verify that the active protocol version is now v1.0.0.
        let mut res = tester
            .api
            .call()
            .worker_testing()
            .test_version(protocol_v1.clone())?
            .submit_and_watch(&mut user)
            .await?;
        res.ok().await?;
        res.wait_finalized().await?;

        // Upload the v2 protocol modules and config.  This only uploads the new modules and config for the new version, and does not change the active protocol version.
        upload_v2(&mut worker_helper).await?;

        // Verify that the active protocol version is still v1.0.0.
        let mut res = tester
            .api
            .call()
            .worker_testing()
            .test_version(protocol_v1.clone())?
            .submit_and_watch(&mut user)
            .await?;
        res.ok().await?;
        res.wait_finalized().await?;

        // Change the active protocol version to v2.0.0.
        let protocol_v2 = Protocol {
            id: protocol.id.clone(),
            version: ProtocolVersion {
                major: 2,
                minor: 0,
                patch: 0,
            },
        };
        worker_helper
            .sudo_call(
                tester
                    .api
                    .call()
                    .worker_testing()
                    .set_protocol_version(protocol_v2.clone())?,
            )
            .await?;

        // Verify that the active protocol version is now v2.0.0.
        let mut res = tester
            .api
            .call()
            .worker_testing()
            .test_version(protocol_v2.clone())?
            .submit_and_watch(&mut user)
            .await?;
        res.ok().await?;
        res.wait_finalized().await?;

        Ok(())
    }
}
