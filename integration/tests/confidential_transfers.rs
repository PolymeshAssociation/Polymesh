// >=v7.3
#[cfg(feature = "current_release")]
mod confidential_assets_tests {
    use anyhow::Result;

    use integration::confidential_assets_helper::*;
    use polymesh_dart::LegConfig;

    #[tokio::test]
    #[test_log::test]
    async fn simple_dart_funding() -> Result<()> {
        const NUM_ASSETS: usize = 5;
        let mut names = vec!["Auditor1".to_string()];
        for asset_idx in 0..NUM_ASSETS {
            names.push(format!("Issuer{}", asset_idx));
            names.push(format!("Investor{}", asset_idx));
        }
        let name_refs = names.iter().map(String::as_str).collect::<Vec<_>>();
        let tester = DartAssetTester::init(&name_refs).await?;
        let auditor = tester.user("Auditor1").await;

        let mut tasks = Vec::new();
        // Create multiple assets concurrently and fund one investor each.
        for asset_idx in 0..NUM_ASSETS {
            let issuer = tester.user(&format!("Issuer{}", asset_idx)).await;
            let investor = tester.user(&format!("Investor{}", asset_idx)).await;
            let tester = tester.clone();
            let auditor = auditor.clone();
            tasks.push(tokio::spawn(async move {
                let name = format!("Test Asset {}", asset_idx);
                tester
                    .create_asset_and_fund_investors(
                        &issuer,
                        &name,
                        &[],
                        &[&auditor],
                        Some(500),
                        &[&investor],
                        100,
                        false,
                    )
                    .await
            }));
        }
        // Wait for all tasks to complete.
        for task in tasks {
            task.await??;
        }

        Ok(())
    }

    #[tokio::test]
    #[test_log::test]
    async fn simple_dart_funding_instant_settlement() -> Result<()> {
        const NUM_ASSETS: usize = 5;
        let mut names = vec!["Auditor1".to_string()];
        for asset_idx in 0..NUM_ASSETS {
            names.push(format!("InstantIssuer{}", asset_idx));
            names.push(format!("InstantInvestor{}", asset_idx));
        }
        let name_refs = names.iter().map(String::as_str).collect::<Vec<_>>();
        let tester = DartAssetTester::init(&name_refs).await?;
        let auditor = tester.user("Auditor1").await;

        let mut tasks = Vec::new();
        // Create multiple assets concurrently and fund one investor each.
        for asset_idx in 0..NUM_ASSETS {
            let issuer = tester.user(&format!("InstantIssuer{}", asset_idx)).await;
            let investor = tester.user(&format!("InstantInvestor{}", asset_idx)).await;
            let tester = tester.clone();
            let auditor = auditor.clone();
            tasks.push(tokio::spawn(async move {
                let name = format!("Test Asset {}", asset_idx);
                tester
                    .create_asset_and_fund_investors(
                        &issuer,
                        &name,
                        &[],
                        &[&auditor],
                        Some(500),
                        &[&investor],
                        100,
                        true,
                    )
                    .await
            }));
        }
        // Wait for all tasks to complete.
        for task in tasks {
            task.await??;
        }

        Ok(())
    }

    /// Test fee account topup.
    #[tokio::test]
    #[test_log::test]
    async fn fee_account_topup() -> Result<()> {
        let tester =
            DartAssetTester::init(&["FeeAccountUser", "Issuer", "Investor", "Auditor", "Relayer"])
                .await?;
        let user = tester.user("FeeAccountUser").await;
        let issuer = tester.user("Issuer").await;
        let investor = tester.user("Investor").await;
        let auditor = tester.user("Auditor").await;
        let relayer = tester.user("Relayer").await;

        let asset_task = {
            let tester = tester.clone();
            let issuer = issuer.clone();
            let auditor = auditor.clone();
            let investor = investor.clone();

            tokio::spawn(async move {
                // Create and mint asset.
                let asset = tester
                    .create_asset(&issuer, "Test Fee payment", &[], &[&auditor], Some(1_000))
                    .await?;

                // Register investor for asset.
                investor.register_account().await?;
                investor.register_account_asset(asset.id).await?;
                Ok::<_, anyhow::Error>(asset)
            })
        };

        // Register fee account.
        user.register_fee_account(42_000_000).await?;

        // Top up fee account.
        user.fee_account_topup(100_000_000).await?;

        // Set the relayer for the fee account.
        user.set_relayer(relayer).await;

        let asset = asset_task.await??;
        // Create a settlement proof to be submitted using a relayer.
        let _settlement = DartSettlementState::new(
            &tester,
            &user,
            &[DartLeg {
                sender: issuer.clone(),
                receiver: investor.clone(),
                asset_id: asset.id,
                amount: 100,
                config: Default::default(),
            }],
            Some(b"Test fee payment"),
        )
        .await?;

        // Do another top up to ensure fee account works after settlement.
        user.fee_account_topup(100_000_000).await?;

        Ok(())
    }

    /// Atomic swap of two assets between two investors.
    ///
    #[tokio::test]
    #[test_log::test]
    async fn atomic_swap_test() -> Result<()> {
        let tester = DartAssetTester::init_with_relayer(
            &[
                "Asset1_Issuer",
                "Asset2_Issuer",
                "Venue",
                "SwapInvestor1",
                "SwapInvestor2",
                "Asset1_Auditor",
                "Asset2_Mediator",
            ],
            "Relayer",
        )
        .await?;
        let asset1_issuer = tester.user("Asset1_Issuer").await;
        let asset2_issuer = tester.user("Asset2_Issuer").await;
        let venue = tester.user("Venue").await;
        let investor1 = tester.user("SwapInvestor1").await;
        let investor2 = tester.user("SwapInvestor2").await;
        let asset1_auditor = tester.user("Asset1_Auditor").await;
        let asset2_mediator = tester.user("Asset2_Mediator").await;

        // Create two assets (one with a mediator and one with an auditor) with different issuers and fund one investor each.
        let asset1 = {
            let tester = tester.clone();
            let investor1 = investor1.clone();
            let investor2 = investor2.clone();
            tokio::spawn(async move {
                let asset = tester
                    .create_asset_and_fund_investors(
                        &asset1_issuer,
                        "Asset1",
                        &[],
                        &[&asset1_auditor],
                        Some(50_000),
                        &[&investor1],
                        500,
                        false,
                    )
                    .await?;

                // Register the other investor for the asset.
                investor2.register_account_asset(asset.id).await?;

                Ok::<_, anyhow::Error>(asset)
            })
        };
        let asset2 = {
            let tester = tester.clone();
            let investor1 = investor1.clone();
            let investor2 = investor2.clone();
            tokio::spawn(async move {
                let asset = tester
                    .create_asset_and_fund_investors(
                        &asset2_issuer,
                        "Asset2",
                        &[&asset2_mediator],
                        &[],
                        Some(100_000),
                        &[&investor2],
                        20_000,
                        false,
                    )
                    .await?;

                // Register the other investor for the asset.
                investor1.register_account_asset(asset.id).await?;

                Ok::<_, anyhow::Error>(asset)
            })
        };
        let asset1 = asset1.await??;
        let asset2 = asset2.await??;
        let asset1_id = asset1.id;
        let asset2_id = asset2.id;

        // Create atomic swap settlement.
        let swap_task = {
            let tester = tester.clone();
            let venue = venue.clone();
            let investor1 = investor1.clone();
            let investor2 = investor2.clone();
            tokio::spawn(async move {
                DartSettlementState::new(
                    &tester,
                    &venue,
                    &[
                        DartLeg {
                            sender: investor1.clone(),
                            receiver: investor2.clone(),
                            asset_id: asset1_id,
                            amount: 250,
                            config: Default::default(),
                        },
                        DartLeg {
                            sender: investor2.clone(),
                            receiver: investor1.clone(),
                            asset_id: asset2_id,
                            amount: 3_000,
                            config: Default::default(),
                        },
                    ],
                    Some(b"Test atomic swap"),
                )
                .await
            })
        };
        // Wait for all tasks to complete.
        let swap = swap_task.await??;

        // All parties affirm all legs.
        swap.affirm_legs(&tester).await?;

        // The investors can now claim their assets.
        swap.receivers_claim_assets(&tester).await?;

        Ok(())
    }

    /// Atomic swap of two assets between two investors with revealed asset ids.
    ///
    #[tokio::test]
    #[test_log::test]
    async fn atomic_swap_revealed_assets() -> Result<()> {
        let tester = DartAssetTester::init_with_relayer(
            &[
                "Asset1_Issuer",
                "Asset2_Issuer",
                "Venue",
                "SwapInvestor1",
                "SwapInvestor2",
                "Asset1_Auditor",
                "Asset2_Mediator",
            ],
            "Relayer",
        )
        .await?;
        let asset1_issuer = tester.user("Asset1_Issuer").await;
        let asset2_issuer = tester.user("Asset2_Issuer").await;
        let venue = tester.user("Venue").await;
        let investor1 = tester.user("SwapInvestor1").await;
        let investor2 = tester.user("SwapInvestor2").await;
        let asset1_auditor = tester.user("Asset1_Auditor").await;
        let asset2_mediator = tester.user("Asset2_Mediator").await;

        // Create two assets (one with a mediator and one with an auditor) with different issuers and fund one investor each.
        let asset1 = {
            let tester = tester.clone();
            let investor1 = investor1.clone();
            let investor2 = investor2.clone();
            tokio::spawn(async move {
                let asset = tester
                    .create_asset_and_fund_investors(
                        &asset1_issuer,
                        "Asset1",
                        &[],
                        &[&asset1_auditor],
                        Some(50_000),
                        &[&investor1],
                        500,
                        false,
                    )
                    .await?;

                // Register the other investor for the asset.
                investor2.register_account_asset(asset.id).await?;

                Ok::<_, anyhow::Error>(asset)
            })
        };
        let asset2 = {
            let tester = tester.clone();
            let investor1 = investor1.clone();
            let investor2 = investor2.clone();
            tokio::spawn(async move {
                let asset = tester
                    .create_asset_and_fund_investors(
                        &asset2_issuer,
                        "Asset2",
                        &[&asset2_mediator],
                        &[],
                        Some(100_000),
                        &[&investor2],
                        20_000,
                        false,
                    )
                    .await?;

                // Register the other investor for the asset.
                investor1.register_account_asset(asset.id).await?;

                Ok::<_, anyhow::Error>(asset)
            })
        };
        let asset1 = asset1.await??;
        let asset2 = asset2.await??;
        let asset1_id = asset1.id;
        let asset2_id = asset2.id;

        // Create atomic swap settlement.
        let swap_task = {
            let tester = tester.clone();
            let venue = venue.clone();
            let investor1 = investor1.clone();
            let investor2 = investor2.clone();
            tokio::spawn(async move {
                DartSettlementState::new(
                    &tester,
                    &venue,
                    &[
                        DartLeg {
                            sender: investor1.clone(),
                            receiver: investor2.clone(),
                            asset_id: asset1_id,
                            amount: 250,
                            config: LegConfig {
                                reveal_asset_id: true,
                                ..Default::default()
                            },
                        },
                        DartLeg {
                            sender: investor2.clone(),
                            receiver: investor1.clone(),
                            asset_id: asset2_id,
                            amount: 3_000,
                            config: LegConfig {
                                reveal_asset_id: true,
                                ..Default::default()
                            },
                        },
                    ],
                    Some(b"Test atomic swap"),
                )
                .await
            })
        };
        // Wait for all tasks to complete.
        let swap = swap_task.await??;

        // All parties affirm all legs.
        swap.affirm_legs(&tester).await?;

        // The investors can now claim their assets.
        swap.receivers_claim_assets(&tester).await?;

        Ok(())
    }

    // ========================================================================
    // Test Case: Sender reverting pending/rejected settlement
    // ========================================================================
    /// Test A: Settlement pending → sender reverses → verify reversal succeeds
    /// Test B: Settlement rejected by mediator → sender reverses → verify reversal succeeds
    /// Expected: Both reversals succeed with correct state transitions
    #[tokio::test]
    #[test_log::test]
    async fn sender_revert_pending_and_rejected_settlement() -> Result<()> {
        let tester =
            DartAssetTester::init(&["Auditor", "Mediator", "Sender", "Receiver", "Venue"]).await?;
        let auditor = tester.user("Auditor").await;
        let mediator = tester.user("Mediator").await;
        let sender = tester.user("Sender").await;
        let receiver = tester.user("Receiver").await;
        let venue = tester.user("Venue").await;

        // Create asset with mediator
        let asset = tester
            .create_asset(
                &sender,
                "Pending and Rejected Revert Test",
                &[&mediator],
                &[&auditor],
                Some(1000),
            )
            .await?;

        sender.register_account().await?;
        sender.register_account_asset(asset.id).await?;
        receiver.register_account().await?;
        receiver.register_account_asset(asset.id).await?;

        // Test A: Pending settlement revert
        let pending_settlement =
            create_test_settlement(&tester, &venue, &sender, &receiver, asset.id, 100).await?;

        // Sender affirms
        pending_settlement.senders_affirm_legs(&tester).await?;

        // Sender reverts while still pending
        pending_settlement
            .senders_revert_affirmation_legs(&tester)
            .await?;
        log::info!("Test A passed: Sender successfully reverted pending settlement");

        // Test B: Rejected settlement revert
        let rejected_settlement =
            create_test_settlement(&tester, &venue, &sender, &receiver, asset.id, 100).await?;

        // Sender affirm.
        rejected_settlement.senders_affirm_legs(&tester).await?;

        // Mediator rejects.
        rejected_settlement
            .mediators_affirm_legs(&tester, false)
            .await?;

        // Sender reverts affirmation after rejection
        rejected_settlement
            .senders_revert_affirmation_legs(&tester)
            .await?;
        log::info!("Test B passed: Sender successfully reverted rejected settlement");

        Ok(())
    }

    // ========================================================================
    // Test Case: Receiver reverting pending/rejected settlement
    // ========================================================================
    /// Test A: Settlement pending → receiver reverses → verify reversal succeeds
    /// Test B: Settlement rejected by mediator → receiver reverses → verify reversal succeeds
    /// Expected: Both reversals succeed with correct state transitions
    #[tokio::test]
    #[test_log::test]
    async fn receiver_revert_pending_and_rejected_settlement() -> Result<()> {
        let tester =
            DartAssetTester::init(&["Auditor", "Mediator", "Sender", "Receiver", "Venue"]).await?;
        let auditor = tester.user("Auditor").await;
        let mediator = tester.user("Mediator").await;
        let sender = tester.user("Sender").await;
        let receiver = tester.user("Receiver").await;
        let venue = tester.user("Venue").await;

        // Create asset with mediator
        let asset = tester
            .create_asset(
                &sender,
                "Pending and Rejected Revert Test",
                &[&mediator],
                &[&auditor],
                Some(1000),
            )
            .await?;

        sender.register_account().await?;
        sender.register_account_asset(asset.id).await?;
        receiver.register_account().await?;
        receiver.register_account_asset(asset.id).await?;

        // Test A: Pending settlement revert
        let pending_settlement =
            create_test_settlement(&tester, &venue, &sender, &receiver, asset.id, 100).await?;

        // Receiver affirms
        pending_settlement.receivers_affirm_legs(&tester).await?;

        // Receiver reverts while still pending
        pending_settlement
            .receivers_revert_affirmation_legs(&tester)
            .await?;
        log::info!("Test A passed: Receiver successfully reverted pending settlement");

        // Test B: Rejected settlement revert
        let rejected_settlement =
            create_test_settlement(&tester, &venue, &sender, &receiver, asset.id, 100).await?;

        // Receiver affirm.
        rejected_settlement.receivers_affirm_legs(&tester).await?;

        // Mediator rejects.
        rejected_settlement
            .mediators_affirm_legs(&tester, false)
            .await?;

        // Receiver reverts affirmation after rejection
        rejected_settlement
            .receivers_revert_affirmation_legs(&tester)
            .await?;
        log::info!("Test B passed: Receiver successfully reverted rejected settlement");

        Ok(())
    }
}
