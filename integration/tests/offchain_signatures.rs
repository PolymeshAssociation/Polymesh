// <v8.0
#[cfg(feature = "previous_release")]
mod offchain_tests {
    use anyhow::{bail, Result};

    use sp_runtime::MultiSignature;

    use polymesh_api::client::Signer;
    use polymesh_api::polymesh::types::{
        pallet_utility::UniqueCall,
        polymesh_primitives::identity::SecondaryKeyWithAuth,
        polymesh_primitives::secondary_key::{Permissions, SecondaryKey},
        primitive_types::H512,
    };
    use polymesh_api_client_extras::*;

    use integration::*;

    /// Test add secondary key using offchain signatures.
    #[tokio::test]
    async fn add_secondary_key_with_authorization() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut user = tester.user("User1").await?;
        let target_id = user.did.expect("User1 did");

        // Create a new signer for each secondary key.
        let mut secondary_keys = vec![
            tester.new_signer_idx("User1", 1)?,
            tester.new_signer_idx("User1", 2)?,
        ];

        // Get the current off-chain nonce for `did`.
        let nonce = tester
            .api
            .query()
            .identity()
            .off_chain_authorization_nonce(target_id)
            .await?;

        // Get current timestamp from chain.
        let now = tester.api.query().timestamp().now().await?;
        let expires_at = now + 60_000; // Expire after 1 minute (ms).

        // Prepare authorazation data.
        let auth = TargetIdAuthorization {
            target_id,
            nonce,
            expires_at,
        };

        // Secondary keys with authorization and permissions.
        let permissions: Permissions = serde_json::from_value(serde_json::json!({
          "asset": "Whole",
          "extrinsic": "Whole",
          "portfolio": "Whole",
        }))?;
        let mut keys = Vec::new();
        for key in &mut secondary_keys {
            match sign_with_key(key, &auth).await? {
                MultiSignature::Sr25519(sig) => {
                    keys.push(SecondaryKeyWithAuth {
                        secondary_key: SecondaryKey {
                            key: key.account(),
                            permissions: permissions.clone(),
                        },
                        auth_signature: H512(sig.0),
                    });
                }
                _ => {
                    bail!("Only Sr25519 keys supported.");
                }
            }
        }

        // Add secondary keys with authorization.
        let mut res = tester
            .api
            .call()
            .identity()
            .add_secondary_keys_with_authorization(keys, expires_at)?
            .submit_and_watch(&mut user)
            .await?;
        let events = res.events().await?;
        println!("Add secondary keys with auth: events = {:#?}", events);
        Ok(())
    }

    /// Test `Utility::relay_tx` using offchain signatures.
    #[tokio::test]
    async fn relay_tx() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["RelayerUser1", "RelayedUser2"]).await?;
        let mut relayer = users.remove(0);
        let relayed = users.remove(0);

        // Get the current off-chain nonce for `RelayedUser2`.
        let nonce = tester
            .api
            .query()
            .utility()
            .nonces(relayer.account())
            .await?;

        for idx in 0..3 {
            // Relay a System.remark call.
            let remark_call = tester
                .api
                .call()
                .system()
                .remark(format!("Hello, Polymesh! {idx}").into())?
                .into_runtime_call();

            let unique_call = UniqueCall {
                call: Box::new(remark_call.clone()),
                nonce: nonce + idx,
            };
            let sig = sign_with_key(&relayed, &unique_call).await?;

            // Use `relayer` to relay the call.
            tester
                .api
                .call()
                .utility()
                .relay_tx(relayed.account(), sig.into(), unique_call)?
                .execute(&mut relayer)
                .await?;
        }

        Ok(())
    }
}

// >=v8.0
#[cfg(feature = "current_release")]
mod offchain_tests {
    use anyhow::{bail, Result};

    use sp_runtime::MultiSignature;

    use polymesh_api::client::Signer;
    use polymesh_api::polymesh::types::{
        polymesh_primitives::identity::SecondaryKeyWithAuth,
        polymesh_primitives::secondary_key::{Permissions, SecondaryKey},
        primitive_types::H512,
    };
    use polymesh_api_client_extras::*;

    use integration::*;

    /// Test add secondary key using offchain signatures.
    #[tokio::test]
    async fn add_secondary_key_with_authorization() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut user = tester.user("User1").await?;
        let target_id = user.did.expect("User1 did");

        // Create a new signer for each secondary key.
        let mut secondary_keys = vec![
            tester.new_signer_idx("User1", 1)?,
            tester.new_signer_idx("User1", 2)?,
        ];

        // Get the current off-chain nonce for `did`.
        let nonce = tester
            .api
            .query()
            .identity()
            .off_chain_authorization_nonce(target_id)
            .await?;

        // Get current timestamp from chain.
        let now = tester.api.query().timestamp().now().await?;
        let expires_at = now + 60_000; // Expire after 1 minute (ms).

        // Prepare authorazation data.
        let auth = TargetIdAuthorization {
            target_id,
            nonce,
            expires_at,
        };

        // Secondary keys with authorization and permissions.
        let permissions: Permissions = serde_json::from_value(serde_json::json!({
          "asset": "Whole",
          "extrinsic": "Whole",
          "portfolio": "Whole",
        }))?;
        let mut keys = Vec::new();
        for key in &mut secondary_keys {
            match sign_with_key(key, &auth).await? {
                MultiSignature::Sr25519(sig) => {
                    keys.push(SecondaryKeyWithAuth {
                        secondary_key: SecondaryKey {
                            key: key.account(),
                            permissions: permissions.clone(),
                        },
                        auth_signature: H512(sig.0),
                    });
                }
                _ => {
                    bail!("Only Sr25519 keys supported.");
                }
            }
        }

        // Add secondary keys with authorization.
        let mut res = tester
            .api
            .call()
            .identity()
            .add_secondary_keys_with_authorization(keys, expires_at)?
            .submit_and_watch(&mut user)
            .await?;
        let events = res.events().await?;
        println!("Add secondary keys with auth: events = {:#?}", events);
        Ok(())
    }

    /// Test `Relayer::relay_tx` using offchain signatures.
    #[tokio::test]
    async fn relay_tx() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester.users(&["RelayerUser1", "RelayedUser2"]).await?;
        let mut relayer = users.remove(0);
        let relayed = users.remove(0);

        // Get the current off-chain nonce for `RelayedUser2`.
        let nonce = tester
            .api
            .query()
            .relayer()
            .relay_tx_nonces(relayer.account())
            .await?;

        for idx in 0..3 {
            // Relay a System.remark call.
            let remark_call = tester
                .api
                .call()
                .system()
                .remark(format!("Hello, Polymesh! {idx}").into())?
                .into_runtime_call();

            let call = remark_call.clone();
            let message =
                ChainScopedMessage::new(&tester.api, nonce + idx, RELAY_TX_LABEL, None, &call)
                    .await?;
            let expires_at = message.expires_at;
            let sig = sign_with_key(&relayed, &message).await?;

            // Use `relayer` to relay the call.
            tester
                .api
                .call()
                .relayer()
                .relay_tx(relayed.account(), sig.into(), call, expires_at)?
                .execute(&mut relayer)
                .await?;
        }

        Ok(())
    }
}
