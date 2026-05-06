// >=v7.3
// Comprehensive test coverage for error conditions and edge cases in DART confidential settlements
#[cfg(feature = "current_release")]
mod confidential_assets_negative_tests {
    use anyhow::Result;
    use integration::confidential_assets_helper::*;

    // ========================================================================
    // Test Case: Settlement with unregistered asset
    // ========================================================================
    /// Try to create settlement with asset that doesn't exist on-chain.
    /// Expected: Settlement creation fails.
    #[tokio::test]
    #[test_log::test]
    async fn settlement_with_unregistered_asset() -> Result<()> {
        let tester = DartAssetTester::init(&["Auditor", "Sender", "Receiver", "Venue"]).await?;
        let auditor = tester.user("Auditor").await;
        let sender = tester.user("Sender").await;
        let receiver = tester.user("Receiver").await;
        let venue = tester.user("Venue").await;

        // Setup: create one asset for reference
        let asset = tester
            .create_asset(&sender, "Test Asset", &[], &[&auditor], Some(1000))
            .await?;

        // Register sender and receiver for the valid asset
        sender.register_account().await?;
        sender.register_account_asset(asset.id).await?;
        receiver.register_account().await?;
        receiver.register_account_asset(asset.id).await?;

        // Try to create settlement with a non-existent asset ID (use a very high ID that's unlikely to exist)
        let fake_asset_id = 99999u32 as DartAssetId;
        let settlement_result =
            create_test_settlement(&tester, &venue, &sender, &receiver, fake_asset_id, 100).await;

        // Verify the settlement creation failed
        assert_operation_fails(settlement_result, "Settlement with unregistered asset");
        log::info!("Test Case passed: Settlement creation correctly rejected unregistered asset");

        Ok(())
    }

    // ========================================================================
    // Test Case: Sender with insufficient balance
    // ========================================================================
    /// Register sender for asset with limited minted amount (100 tokens).
    /// Try to affirm settlement with larger amount (150 tokens).
    /// Expected: Proof generation fails with balance validation error.
    #[tokio::test]
    #[test_log::test]
    async fn sender_insufficient_balance() -> Result<()> {
        let tester = DartAssetTester::init(&["Auditor", "Sender", "Receiver", "Venue"]).await?;
        let auditor = tester.user("Auditor").await;
        let sender = tester.user("Sender").await;
        let receiver = tester.user("Receiver").await;
        let venue = tester.user("Venue").await;

        // Create asset and mint only 100 tokens to sender
        let asset = tester
            .create_asset(&sender, "Limited Asset", &[], &[&auditor], Some(100))
            .await?;

        sender.register_account().await?;
        sender.register_account_asset(asset.id).await?;
        receiver.register_account().await?;
        receiver.register_account_asset(asset.id).await?;

        // Create settlement with amount larger than the sender's balance (150 tokens)
        let settlement = create_test_settlement(
            &tester, &venue, &sender, &receiver, asset.id, 150, // More than available
        )
        .await?;

        // Try to have sender affirm the leg (should fail due to insufficient balance)
        let affirm_results = settlement.senders_affirm_legs(&tester).await;

        assert_operation_fails(
            affirm_results,
            "Sender affirmation with insufficient balance",
        );
        log::info!(
            "Test Case passed: Settlement creation correctly rejected due to insufficient balance"
        );

        Ok(())
    }

    // ========================================================================
    // Test Case: Double affirmation by sender
    // ========================================================================
    /// Create settlement, sender affirms once successfully.
    /// Try to call sender_affirmation() again on same leg.
    /// Expected: Second affirmation fails.
    #[tokio::test]
    #[test_log::test]
    async fn double_sender_affirmation() -> Result<()> {
        let tester = DartAssetTester::init(&["Auditor", "Sender", "Receiver", "Venue"]).await?;
        let auditor = tester.user("Auditor").await;
        let sender = tester.user("Sender").await;
        let receiver = tester.user("Receiver").await;
        let venue = tester.user("Venue").await;

        // Setup
        let asset = tester
            .create_asset(&sender, "Double Affirm Test", &[], &[&auditor], Some(1000))
            .await?;

        sender.register_account().await?;
        sender.register_account_asset(asset.id).await?;
        receiver.register_account().await?;
        receiver.register_account_asset(asset.id).await?;

        // Create settlement
        let settlement =
            create_test_settlement(&tester, &venue, &sender, &receiver, asset.id, 100).await?;

        // First affirmation succeeds
        settlement.senders_affirm_legs(&tester).await?;

        // Second affirmation on same leg should fail
        let second_affirmation = settlement.senders_affirm_legs(&tester).await;

        assert_operation_fails(second_affirmation, "Second sender affirmation");
        log::info!("Test Case passed: Second sender affirmation correctly rejected");

        Ok(())
    }

    // ========================================================================
    // Test Case: Sender claiming without prior affirmation
    // ========================================================================
    /// Create settlement, skip sender affirmation, try `sender_revert_affirmation()` immediately
    /// Expected: Revert affirmation fails with on-chain state validation error
    #[tokio::test]
    #[test_log::test]
    async fn sender_claiming_without_affirmation() -> Result<()> {
        let tester = DartAssetTester::init(&["Auditor", "Sender", "Receiver", "Venue"]).await?;
        let auditor = tester.user("Auditor").await;
        let sender = tester.user("Sender").await;
        let receiver = tester.user("Receiver").await;
        let venue = tester.user("Venue").await;

        // Setup
        let asset = tester
            .create_asset(&sender, "No Affirmation Test", &[], &[&auditor], Some(1000))
            .await?;

        sender.register_account().await?;
        sender.register_account_asset(asset.id).await?;
        receiver.register_account().await?;
        receiver.register_account_asset(asset.id).await?;

        // Create settlement
        let settlement =
            create_test_settlement(&tester, &venue, &sender, &receiver, asset.id, 100).await?;

        // Try to have sender revert affirmation without affirming first (should fail)
        let revert_result = settlement.senders_revert_affirmation_legs(&tester).await;

        assert_operation_fails(revert_result, "Sender revert without prior affirmation");
        log::info!("Test Case passed: Sender revert affirmation correctly rejected without prior affirmation");

        Ok(())
    }

    // ========================================================================
    // Test Case: Sender claiming multiple times
    // ========================================================================
    /// Create settlement, sender affirms, sender reverts affirmation successfully
    /// Try to call `sender_revert_affirmation()` again on same leg
    /// Expected: Second revert affirmation fails with "Leg not found" or state validation error
    #[tokio::test]
    #[test_log::test]
    async fn sender_claim_multiple_times() -> Result<()> {
        let tester = DartAssetTester::init(&["Auditor", "Sender", "Receiver", "Venue"]).await?;
        let auditor = tester.user("Auditor").await;
        let sender = tester.user("Sender").await;
        let receiver = tester.user("Receiver").await;
        let venue = tester.user("Venue").await;

        // Setup
        let asset = tester
            .create_asset(
                &sender,
                "Multiple Revert Test",
                &[],
                &[&auditor],
                Some(1000),
            )
            .await?;

        sender.register_account().await?;
        sender.register_account_asset(asset.id).await?;
        receiver.register_account().await?;
        receiver.register_account_asset(asset.id).await?;

        // Create settlement
        let settlement =
            create_test_settlement(&tester, &venue, &sender, &receiver, asset.id, 100).await?;

        // Sender affirms
        settlement.senders_affirm_legs(&tester).await?;

        // First revert succeeds
        settlement.senders_revert_affirmation_legs(&tester).await?;

        // Second revert on same leg should fail
        let second_revert = settlement.senders_revert_affirmation_legs(&tester).await;

        assert_operation_fails(
            second_revert,
            "Second sender revert affirmation should fail",
        );
        log::info!("Test Case passed: Second sender revert affirmation correctly rejected");

        Ok(())
    }

    // ========================================================================
    // Test Case: Sender claiming from executed settlement
    // ========================================================================
    /// Create settlement with mediator, all parties affirm, settlement executes
    /// Try `sender_revert_affirmation()` on executed settlement
    /// Expected: Fails with "Can only reverse pending/rejected settlement" on-chain check
    #[tokio::test]
    #[test_log::test]
    async fn sender_revert_from_executed_settlement() -> Result<()> {
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
                "Executed Settlement Revert Test",
                &[&mediator],
                &[&auditor],
                Some(1000),
            )
            .await?;

        sender.register_account().await?;
        sender.register_account_asset(asset.id).await?;
        receiver.register_account().await?;
        receiver.register_account_asset(asset.id).await?;

        // Create settlement
        let settlement =
            create_test_settlement(&tester, &venue, &sender, &receiver, asset.id, 100).await?;

        // All parties affirm including mediator (settlement becomes executed)
        settlement.affirm_legs(&tester).await?;

        // Try to have sender revert affirmation from executed settlement (should fail)
        let revert_result = settlement.senders_revert_affirmation_legs(&tester).await;

        assert_operation_fails(
            revert_result,
            "Sender revert affirmation from executed settlement should fail",
        );
        log::info!("Test Case passed: Sender revert affirmation correctly rejected from executed settlement");

        Ok(())
    }

    // ========================================================================
    // Test Case: Affirming after mediator rejection
    // ========================================================================
    /// Create settlement with mediator, mediator rejects leg.
    /// Try to call sender_affirmation() or receiver_affirmation() on rejected leg.
    /// Expected: Affirmation fails due to settlement status (Rejected).
    #[tokio::test]
    #[test_log::test]
    async fn affirmation_after_mediator_rejection() -> Result<()> {
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
                "Mediator Rejection Test",
                &[&mediator],
                &[&auditor],
                Some(1000),
            )
            .await?;

        sender.register_account().await?;
        sender.register_account_asset(asset.id).await?;
        receiver.register_account().await?;
        receiver.register_account_asset(asset.id).await?;

        // Create settlement
        let settlement =
            create_test_settlement(&tester, &venue, &sender, &receiver, asset.id, 100).await?;

        // Mediator rejects first
        settlement.mediators_affirm_legs(&tester, false).await?;

        // Try to have sender affirm after rejection (should fail)
        let late_sender_affirmation = settlement.senders_affirm_legs(&tester).await;

        assert_operation_fails(
            late_sender_affirmation,
            "Sender affirmation after mediator rejection",
        );

        // Try to have receiver affirm after rejection (should fail)
        let late_receiver_affirmation = settlement.receivers_affirm_legs(&tester).await;

        assert_operation_fails(
            late_receiver_affirmation,
            "Receiver affirmation after mediator rejection",
        );
        log::info!("Test Case passed: Affirmations correctly rejected after mediator rejection");

        Ok(())
    }

    // ========================================================================
    // Test Case: Receiver claiming from rejected settlement
    // ========================================================================
    /// Create settlement with mediator, mediator rejects, settlement marked rejected.
    /// Try receiver_claim() on rejected leg.
    /// Expected: Claim fails with on-chain settlement status check.
    #[tokio::test]
    #[test_log::test]
    async fn receiver_claim_from_rejected_settlement() -> Result<()> {
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
                "Rejected Settlement Test",
                &[&mediator],
                &[&auditor],
                Some(1000),
            )
            .await?;

        sender.register_account().await?;
        sender.register_account_asset(asset.id).await?;
        receiver.register_account().await?;
        receiver.register_account_asset(asset.id).await?;

        // Create settlement
        let settlement =
            create_test_settlement(&tester, &venue, &sender, &receiver, asset.id, 100).await?;

        // Sender and receiver affirm
        settlement.senders_affirm_legs(&tester).await?;
        settlement.receivers_affirm_legs(&tester).await?;

        // Mediator REJECTS (not accepts)
        settlement.mediators_affirm_legs(&tester, false).await?;

        // Try to have receiver claim from rejected settlement (should fail)
        let claim_result = settlement.receivers_claim_assets(&tester).await;

        assert_operation_fails(claim_result, "Receiver claim from rejected settlement");
        log::info!("Test Case passed: Claim correctly rejected from rejected settlement");

        Ok(())
    }

    // ========================================================================
    // Test Case: Receiver claiming multiple times
    // ========================================================================
    /// Create settlement, all parties affirm, receiver claims successfully.
    /// Try to call receiver_claim() again on same leg.
    /// Expected: Second claim fails.
    #[tokio::test]
    #[test_log::test]
    async fn receiver_claim_multiple_times() -> Result<()> {
        let tester = DartAssetTester::init(&["Auditor", "Sender", "Receiver", "Venue"]).await?;
        let auditor = tester.user("Auditor").await;
        let sender = tester.user("Sender").await;
        let receiver = tester.user("Receiver").await;
        let venue = tester.user("Venue").await;

        // Setup
        let asset = tester
            .create_asset(&sender, "Double Claim Test", &[], &[&auditor], Some(1000))
            .await?;

        sender.register_account().await?;
        sender.register_account_asset(asset.id).await?;
        receiver.register_account().await?;
        receiver.register_account_asset(asset.id).await?;

        // Create settlement
        let settlement =
            create_test_settlement(&tester, &venue, &sender, &receiver, asset.id, 100).await?;

        // All parties affirm
        settlement.affirm_legs(&tester).await?;

        // First claim succeeds
        settlement.receivers_claim_assets(&tester).await?;

        // Second claim on same leg should fail
        let second_claim = settlement.receivers_claim_assets(&tester).await;

        assert_operation_fails(second_claim, "Second claim should fail");
        log::info!("Test Case passed: Second claim correctly rejected");

        Ok(())
    }

    // ========================================================================
    // Test Case: Double receiver affirmation
    // ========================================================================
    /// Create settlement, receiver affirms once successfully.
    /// Try to call receiver_affirmation() again on same leg.
    /// Expected: Second affirmation fails.
    #[tokio::test]
    #[test_log::test]
    async fn double_receiver_affirmation() -> Result<()> {
        let tester = DartAssetTester::init(&["Auditor", "Sender", "Receiver", "Venue"]).await?;
        let auditor = tester.user("Auditor").await;
        let sender = tester.user("Sender").await;
        let receiver = tester.user("Receiver").await;
        let venue = tester.user("Venue").await;

        // Setup
        let asset = tester
            .create_asset(
                &sender,
                "Double Receiver Affirm Test",
                &[],
                &[&auditor],
                Some(1000),
            )
            .await?;

        sender.register_account().await?;
        sender.register_account_asset(asset.id).await?;
        receiver.register_account().await?;
        receiver.register_account_asset(asset.id).await?;

        // Create settlement
        let settlement =
            create_test_settlement(&tester, &venue, &sender, &receiver, asset.id, 100).await?;

        // First affirmation succeeds
        settlement.receivers_affirm_legs(&tester).await?;

        // Second affirmation on same leg should fail
        let second_affirmation = settlement.receivers_affirm_legs(&tester).await;

        assert_operation_fails(
            second_affirmation,
            "Second receiver affirmation should fail",
        );
        log::info!("Test Case 8 passed: Second receiver affirmation correctly rejected");

        Ok(())
    }

    // ========================================================================
    // Test Case: Mediator affirming/rejecting multiple times
    // ========================================================================
    /// Create settlement with mediator, mediator affirms/rejects once.
    /// Try to call mediator_affirmation() again on same leg.
    /// Expected: Second affirmation fails.
    #[tokio::test]
    #[test_log::test]
    async fn mediator_affirmation_multiple_times() -> Result<()> {
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
                "Mediator Double Affirm Test",
                &[&mediator],
                &[&auditor],
                Some(1000),
            )
            .await?;

        sender.register_account().await?;
        sender.register_account_asset(asset.id).await?;
        receiver.register_account().await?;
        receiver.register_account_asset(asset.id).await?;

        // Create settlement
        let settlement =
            create_test_settlement(&tester, &venue, &sender, &receiver, asset.id, 100).await?;

        // First mediator affirmation (accept) succeeds
        settlement.mediators_affirm_legs(&tester, true).await?;

        // Second mediator affirmation on same leg should fail
        let second_affirmation = settlement.mediators_affirm_legs(&tester, false).await;

        assert_operation_fails(
            second_affirmation,
            "Second mediator affirmation should fail",
        );
        log::info!("Test Case passed: Second mediator affirmation correctly rejected");

        Ok(())
    }

    // ========================================================================
    // Test Case: Mediator rejecting executed settlement
    // ========================================================================
    /// Create settlement with all parties affirmed and executed.
    /// Try to call mediator_affirmation(accept=false) on executed settlement.
    /// Expected: Rejection fails with on-chain settlement status check.
    #[tokio::test]
    #[test_log::test]
    async fn mediator_rejection_of_executed_settlement() -> Result<()> {
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
                "Executed Rejection Test",
                &[&mediator],
                &[&auditor],
                Some(1000),
            )
            .await?;

        sender.register_account().await?;
        sender.register_account_asset(asset.id).await?;
        receiver.register_account().await?;
        receiver.register_account_asset(asset.id).await?;

        // Create settlement
        let settlement =
            create_test_settlement(&tester, &venue, &sender, &receiver, asset.id, 100).await?;

        // All parties affirm including mediator (settlement becomes executed)
        settlement.affirm_legs(&tester).await?;

        // Try to have mediator reject from executed settlement (should fail)
        let late_rejection = settlement.mediators_affirm_legs(&tester, false).await;

        assert_operation_fails(
            late_rejection,
            "Mediator rejection of executed settlement should fail",
        );
        log::info!(
            "Test Case passed: Mediator rejection correctly rejected from executed settlement"
        );

        Ok(())
    }

    // ========================================================================
    // Test Case: Late asset registration by receiver
    // ========================================================================
    /// Create settlement WITHOUT registering receiver for asset first.
    /// Receiver registers for asset only after settlement creation.
    /// Expected: Receiver affirmation/claim succeeds after registration.
    #[tokio::test]
    #[test_log::test]
    async fn receiver_late_asset_registration() -> Result<()> {
        let tester = DartAssetTester::init(&["Auditor", "Sender", "Receiver", "Venue"]).await?;
        let auditor = tester.user("Auditor").await;
        let sender = tester.user("Sender").await;
        let receiver = tester.user("Receiver").await;
        let venue = tester.user("Venue").await;

        // Setup asset
        let asset = tester
            .create_asset(
                &sender,
                "Late Registration Test",
                &[],
                &[&auditor],
                Some(1000),
            )
            .await?;

        sender.register_account().await?;
        sender.register_account_asset(asset.id).await?;
        // NOTE: receiver is NOT registered for the asset yet

        receiver.register_account().await?;
        // receiver.register_account_asset(asset.id) <- NOT called

        // Create settlement without receiver being registered for asset
        // Note: This may succeed at settlement creation (on-chain can't enforce off-chain registration)
        let settlement =
            create_test_settlement(&tester, &venue, &sender, &receiver, asset.id, 100).await?;

        // Now sender affirms
        settlement.senders_affirm_legs(&tester).await?;

        // If settlement creation succeeded, receiver should register for asset
        receiver.register_account_asset(asset.id).await?;

        // After registration, receiver affirmation should succeed
        settlement.receivers_affirm_legs(&tester).await?;

        // Settlement should now be executed.

        // Receiver claim should also succeed
        settlement.receivers_claim_assets(&tester).await?;

        log::info!("Test Case passed: Receiver operations succeeded after late asset registration");

        Ok(())
    }

    // ========================================================================
    // Test Case: Affirming with wrong amount (asset mismatch test)
    // ========================================================================
    /// Create settlement with one amount, try to affirm with a different amount.
    /// Expected: Affirmation fails due to amount mismatch.
    #[tokio::test]
    #[test_log::test]
    async fn affirmation_with_wrong_amount() -> Result<()> {
        let tester = DartAssetTester::init(&["Auditor", "Sender", "Receiver", "Venue"]).await?;
        let auditor = tester.user("Auditor").await;
        let sender = tester.user("Sender").await;
        let receiver = tester.user("Receiver").await;
        let venue = tester.user("Venue").await;

        // Setup
        let asset = tester
            .create_asset(&sender, "Wrong Amount Test", &[], &[&auditor], Some(1000))
            .await?;

        sender.register_account().await?;
        sender.register_account_asset(asset.id).await?;
        receiver.register_account().await?;
        receiver.register_account_asset(asset.id).await?;

        // Create settlement with amount 100
        let settlement =
            create_test_settlement(&tester, &venue, &sender, &receiver, asset.id, 100).await?;

        // Try to affirm as sender with DIFFERENT amount (50 instead of 100)
        let wrong_amount_affirmation = sender
            .sender_affirmation(&tester, settlement.legs[0].leg_ref, asset.id, 50) // Wrong amount
            .await;

        assert_operation_fails(
            wrong_amount_affirmation,
            "Sender affirmation with wrong amount should fail",
        );
        log::info!("Test Case passed: Affirmation with wrong amount correctly rejected");

        Ok(())
    }

    // ========================================================================
    // Test Case: Affirmation by unregistered party
    // ========================================================================
    /// Create settlement but DON'T register receiver for asset before affirming.
    /// Try to have receiver affirm without prior asset registration.
    /// Expected: Affirmation fails due to missing asset registration.
    #[tokio::test]
    #[test_log::test]
    async fn affirmation_by_unregistered_party() -> Result<()> {
        let tester = DartAssetTester::init(&["Auditor", "Sender", "Receiver", "Venue"]).await?;
        let auditor = tester.user("Auditor").await;
        let sender = tester.user("Sender").await;
        let receiver = tester.user("Receiver").await;
        let venue = tester.user("Venue").await;

        // Setup asset
        let asset = tester
            .create_asset(
                &sender,
                "Unregistered Affirm Test",
                &[],
                &[&auditor],
                Some(1000),
            )
            .await?;

        sender.register_account().await?;
        sender.register_account_asset(asset.id).await?;
        receiver.register_account().await?;
        // NOTE: receiver NOT registered for asset

        // Create settlement without receiver being registered for asset
        let settlement =
            create_test_settlement(&tester, &venue, &sender, &receiver, asset.id, 100).await?;

        // Sender can affirm (is registered)
        settlement.senders_affirm_legs(&tester).await?;

        // Try to have receiver affirm without being registered (should fail)
        let unregistered_affirm = receiver
            .receiver_affirmation(&tester, settlement.legs[0].leg_ref, asset.id, 100)
            .await;

        assert_operation_fails(
            unregistered_affirm,
            "Receiver affirmation without asset registration should fail",
        );
        log::info!("Test Case passed: Affirmation by unregistered party correctly rejected");

        Ok(())
    }
}
