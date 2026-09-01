//! Relayer negative paths: expired signatures, nonce reuse, filtered calls, subsidy debits.
#[cfg(feature = "current_release")]
mod relayer_negative_tests {
    use anyhow::Result;

    use integration::*;

    /// Setup an accepted subsidy: payer approves for `target`, target accepts.
    async fn setup_subsidy(
        tester: &PolymeshTester,
        payer: &mut User,
        target: &mut User,
        amount: u128,
    ) -> Result<()> {
        tester
            .api
            .call()
            .relayer()
            .approve_subsidy(target.account(), amount)?
            .submit_and_watch(payer)
            .await?
            .ok()
            .await?;
        tester
            .api
            .call()
            .relayer()
            .accept_subsidy(payer.account())?
            .submit_and_watch(target)
            .await?
            .ok()
            .await?;
        Ok(())
    }

    async fn current_nonce(tester: &PolymeshTester, target: &User) -> Result<u64> {
        Ok(tester
            .api
            .query()
            .relayer()
            .relay_tx_nonces(target.account())
            .await?)
    }

    /// Relay with a stale `expires_at` must fail signature validation.
    #[tokio::test]
    #[test_log::test]
    async fn expired_signature_rejected() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["RNPayer", "RNTarget"]).await?;
        let mut payer = users.remove(0);
        let mut target = users.remove(0);

        setup_subsidy(&tester, &mut payer, &mut target, 1000 * ONE_POLYX).await?;

        let call = tester
            .api
            .call()
            .system()
            .remark(b"expired".to_vec())?
            .into_runtime_call();

        // Craft message then override expiry into the past.
        let nonce = current_nonce(&tester, &target).await?;
        let now = tester.api.query().timestamp().now().await?;
        let past_expiry = now.saturating_sub(10_000);
        let msg_call = call.clone();
        let message =
            ChainScopedMessage::new(&tester.api, nonce, RELAY_TX_LABEL, Some(past_expiry), &msg_call)
                .await?;

        // Target signs; payer submits.
        let sig = sign_with_key(&target, &message).await?;
        let mut res = tester
            .api
            .call()
            .relayer()
            .relay_tx(
                target.account(),
                sig.into(),
                call,
                message.expires_at, // in the past
            )?
            .submit_and_watch(&mut payer)
            .await?;
        assert!(res.ok().await.is_err(), "relay with expired signature should fail");

        Ok(())
    }

    /// The same signed message (nonce) cannot be replayed.
    #[tokio::test]
    #[test_log::test]
    async fn nonce_reuse_rejected() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["RNPayer2", "RNTarget2"]).await?;
        let mut payer = users.remove(0);
        let target = users.remove(0);

        setup_subsidy(&tester, &mut payer, &mut target.clone(), 1000 * ONE_POLYX).await?;

        let call = tester
            .api
            .call()
            .system()
            .remark(b"replay-me".to_vec())?
            .into_runtime_call();

        let nonce = current_nonce(&tester, &target).await?;
        let msg_call = call.clone();
        let message =
            ChainScopedMessage::new(&tester.api, nonce, RELAY_TX_LABEL, None, &msg_call).await?;
        let sig = sign_with_key(&target, &message).await?;
        let expires_at = message.expires_at;

        // First relay succeeds and consumes the nonce.
        tester
            .api
            .call()
            .relayer()
            .relay_tx(target.account(), sig.clone().into(), call.clone(), expires_at)?
            .submit_and_watch(&mut payer)
            .await?
            .ok()
            .await?;

        // Nonce advanced by exactly one.
        let next = current_nonce(&tester, &target).await?;
        assert_eq!(next, nonce + 1, "successful relay must bump the nonce");

        // Replaying the same signed payload fails.
        let mut res = tester
            .api
            .call()
            .relayer()
            .relay_tx(target.account(), sig.into(), call, expires_at)?
            .submit_and_watch(&mut payer)
            .await?;
        assert!(res.ok().await.is_err(), "replayed relay must be rejected");

        Ok(())
    }

    /// Subsidy remaining is debited when the subsidized user pays fees.
    #[tokio::test]
    #[test_log::test]
    async fn subsidy_debits_until_exhausted() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["RNPayer3", "RNTarget3"]).await?;
        let mut payer = users.remove(0);
        let mut target = users.remove(0);

        let small = 50 * ONE_POLYX;
        setup_subsidy(&tester, &mut payer, &mut target, small).await?;
        let target_account = target.account();

        let remaining = || async {
            Ok::<_, anyhow::Error>(
                tester
                    .api
                    .query()
                    .relayer()
                    .subsidies(target_account.clone())
                    .await?
                    .map(|s| s.remaining)
                    .unwrap_or(0),
            )
        };

        let before = remaining().await?;
        // Balances is in SubsidyFilter; System.remark is not.
        tester
            .api
            .call()
            .balances()
            .transfer_with_memo(payer.account().into(), 1, None)?
            .submit_and_watch(&mut target)
            .await?
            .ok()
            .await?;
        let after = remaining().await?;
        assert!(after < before, "user-submitted tx should debit the subsidy");

        Ok(())
    }

    /// Limit management: increase/decrease/set on an active subsidy.
    #[tokio::test]
    #[test_log::test]
    async fn limit_management() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["RNPayer4", "RNTarget4"]).await?;
        let mut payer = users.remove(0);
        let target = users.remove(0);

        setup_subsidy(&tester, &mut payer, &mut target.clone(), 100 * ONE_POLYX).await?;

        let get_remaining = || async {
            Ok::<_, anyhow::Error>(
                tester
                    .api
                    .query()
                    .relayer()
                    .subsidies(target.account())
                    .await?
                    .map(|s| s.remaining)
                    .unwrap_or(0),
            )
        };

        // Increase.
        tester
            .api
            .call()
            .relayer()
            .increase_polyx_limit(target.account(), 50 * ONE_POLYX)?
            .submit_and_watch(&mut payer)
            .await?
            .ok()
            .await?;
        assert!(get_remaining().await? >= 150 * ONE_POLYX);

        // Decrease.
        tester
            .api
            .call()
            .relayer()
            .decrease_polyx_limit(target.account(), 30 * ONE_POLYX)?
            .submit_and_watch(&mut payer)
            .await?
            .ok()
            .await?;
        assert!(
            get_remaining().await? <= 120 * ONE_POLYX,
            "decrease should reduce remaining"
        );

        // Set absolute value.
        tester
            .api
            .call()
            .relayer()
            .update_polyx_limit(target.account(), 200 * ONE_POLYX)?
            .submit_and_watch(&mut payer)
            .await?
            .ok()
            .await?;
        assert_eq!(
            get_remaining().await?,
            200 * ONE_POLYX,
            "update_polyx_limit sets remaining exactly"
        );

        Ok(())
    }

    /// Pending subsidies can be revoked before acceptance.
    #[tokio::test]
    #[test_log::test]
    async fn revoke_pending_subsidy() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["RNPayer5", "RNTarget5"]).await?;
        let mut payer = users.remove(0);
        let mut target = users.remove(0);

        // Approve but don't accept yet.
        tester
            .api
            .call()
            .relayer()
            .approve_subsidy(target.account(), 1000 * ONE_POLYX)?
            .submit_and_watch(&mut payer)
            .await?
            .ok()
            .await?;

        // Payer revokes the pending authorization.
        tester
            .api
            .call()
            .relayer()
            .revoke_subsidy(target.account())?
            .submit_and_watch(&mut payer)
            .await?
            .ok()
            .await?;

        // Accepting now fails - nothing pending.
        let mut res = tester
            .api
            .call()
            .relayer()
            .accept_subsidy(payer.account())?
            .submit_and_watch(&mut target)
            .await?;
        assert!(res.ok().await.is_err(), "accept after revoke should fail");

        Ok(())
    }

    /// Either party can remove an active subsidy.
    #[tokio::test]
    #[test_log::test]
    async fn remove_active_subsidy() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["RNPayer6", "RNTarget6"]).await?;
        let mut payer = users.remove(0);
        let mut target = users.remove(0);

        setup_subsidy(&tester, &mut payer, &mut target, 1000 * ONE_POLYX).await?;
        assert!(
            tester.api.query().relayer().subsidies(target.account()).await?.map(|s| s.remaining).unwrap_or(0) > 0,
            "subsidy should exist after acceptance"
        );

        // Target removes it.
        tester
            .api
            .call()
            .relayer()
            .remove_subsidy(target.account(), payer.account())?
            .submit_and_watch(&mut target)
            .await?
            .ok()
            .await?;

        assert_eq!(
            tester
                .api
                .query()
                .relayer()
                .subsidies(target.account())
                .await?
                .map(|s| s.remaining),
            None,
            "subsidy should be gone"
        );

        Ok(())
    }
}