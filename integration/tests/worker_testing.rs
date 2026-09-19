// >=v7.3
#[cfg(feature = "current_release")]
mod worker_modules_tests {
    use anyhow::{anyhow, Result};

    use integration::{
        worker_modules_helper::*, PolymeshTester, RuntimeEvent, User, WorkerTestingEvent,
    };

    /// Submits a `test_version` work request and checks the `TestingProtocolTask` event.
    ///
    /// The testing protocol module compares the supplied protocol against its own compile-time
    /// version, so the task result shows which module version the worker actually loaded. The
    /// extrinsic succeeds either way, so the result has to be read from the event.
    async fn check_active_version(
        tester: &PolymeshTester,
        user: &mut User,
        protocol: &Protocol,
        expect_active: bool,
    ) -> Result<()> {
        let mut res = tester
            .api
            .call()
            .worker_testing()
            .test_version(protocol.clone())?
            .submit_and_watch(user)
            .await?;
        res.ok().await?;

        let events = res
            .events()
            .await?
            .ok_or_else(|| anyhow!("no events for `test_version({protocol:?})`"))?;
        // Copy the task result out so the borrow on `res` ends before finalizing below.
        let task_result = events
            .0
            .iter()
            .find_map(|rec| match &rec.event {
                RuntimeEvent::WorkerTesting(WorkerTestingEvent::TestingProtocolTask {
                    result,
                    ..
                }) => Some(result.as_ref().map(|_| ()).map_err(|err| format!("{err:?}"))),
                _ => None,
            })
            .ok_or_else(|| {
                anyhow!("missing `TestingProtocolTask` event for `test_version({protocol:?})`")
            })?;
        res.wait_finalized().await?;

        match (expect_active, task_result) {
            (true, Err(err)) => Err(anyhow!(
                "expected {protocol:?} to be the active protocol version, got error: {err}"
            )),
            (false, Ok(())) => Err(anyhow!(
                "expected {protocol:?} to not be the active protocol version, but the version check passed"
            )),
            _ => Ok(()),
        }
    }

    async fn upload_v1(helper: &mut WorkerModulesHelper) -> Result<()> {
        let polkavm_zst = include_bytes!(
            "../../worker/modules/testing/v1/polymesh-worker-protocol-testing.polkavm.zst"
        )
        .to_vec();
        let polkavm = include_bytes!(
            "../../worker/modules/testing/v1/polymesh-worker-protocol-testing.polkavm"
        )
        .to_vec();
        let wasm_zst = include_bytes!(
            "../../worker/modules/testing/v1/polymesh-worker-protocol-testing.wasm.zst"
        )
        .to_vec();
        let wasm =
            include_bytes!("../../worker/modules/testing/v1/polymesh-worker-protocol-testing.wasm")
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
                    (BackendModuleKind::PolkaVM, 2, polkavm_zst.clone()),
                    (BackendModuleKind::PolkaVM, 1, polkavm.clone()),
                    (BackendModuleKind::Wasm, 2, wasm_zst.clone()),
                    (BackendModuleKind::Wasm, 1, wasm.clone()),
                ],
            )
            .await?;
        Ok(())
    }

    async fn upload_v2(helper: &mut WorkerModulesHelper) -> Result<()> {
        let polkavm_zst = include_bytes!(
            "../../worker/modules/testing/v2/polymesh-worker-protocol-testing.polkavm.zst"
        )
        .to_vec();
        let polkavm = include_bytes!(
            "../../worker/modules/testing/v2/polymesh-worker-protocol-testing.polkavm"
        )
        .to_vec();
        let wasm_zst = include_bytes!(
            "../../worker/modules/testing/v2/polymesh-worker-protocol-testing.wasm.zst"
        )
        .to_vec();
        let wasm =
            include_bytes!("../../worker/modules/testing/v2/polymesh-worker-protocol-testing.wasm")
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
                    (BackendModuleKind::PolkaVM, 2, polkavm_zst.clone()),
                    (BackendModuleKind::PolkaVM, 1, polkavm.clone()),
                    (BackendModuleKind::Wasm, 2, wasm_zst.clone()),
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

        // Run the `VerifyVersion` work request against the genesis protocol version.
        check_active_version(&tester, &mut user, &protocol, true).await?;

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
        check_active_version(&tester, &mut user, &protocol, true).await?;

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

        // Verify that the active protocol version is now v1.0.0 and no longer the old version.
        check_active_version(&tester, &mut user, &protocol_v1, true).await?;
        check_active_version(&tester, &mut user, &protocol, false).await?;

        // Upload the v2 protocol modules and config.  This only uploads the new modules and config for the new version, and does not change the active protocol version.
        upload_v2(&mut worker_helper).await?;

        // Verify that the active protocol version is still v1.0.0.
        check_active_version(&tester, &mut user, &protocol_v1, true).await?;

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

        // Verify that the active protocol version is now v2.0.0 and no longer v1.0.0.
        check_active_version(&tester, &mut user, &protocol_v2, true).await?;
        check_active_version(&tester, &mut user, &protocol_v1, false).await?;

        Ok(())
    }
}
