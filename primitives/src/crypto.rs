// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymesh Association

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use codec::{Encode, Output};
use frame_support::pallet_prelude::Zero;
use frame_system::pallet_prelude::BlockNumberFor;
use sp_core::{
    crypto::ByteArray,
    sr25519::{Public, Signature},
    H512,
};
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_runtime::AnySignature;
use sp_runtime_interface::{
    pass_by::{AllocateAndReturnByCodec, PassFatPointerAndRead},
    runtime_interface,
};

/// Native interface for runtime module to use some Schnorrkel functionality.
#[runtime_interface]
pub trait NativeSchnorrkel {
    /// Sign the message `message`, using the given secret key.
    /// It returns `None` if the secret key cannot be created from the input raw bytes.
    fn sign(
        raw_sk: PassFatPointerAndRead<&[u8]>,
        message: PassFatPointerAndRead<&[u8]>,
    ) -> AllocateAndReturnByCodec<Option<Signature>> {
        use schnorrkel::{keys::SecretKey, signing_context, Keypair};
        const SIGNING_CTX: &[u8] = b"substrate";

        SecretKey::from_bytes(raw_sk)
            .map(|sk| {
                let pair = Keypair::from(sk);
                let context = signing_context(SIGNING_CTX);
                let raw_signature = pair.sign(context.bytes(message)).to_bytes();
                Signature::from_raw(raw_signature)
            })
            .ok()
    }
}

/// BytesWrapped is a wrapper for signing the raw SCALE bytes of `T`.
///
/// The wrapped `T` type will be SCALE encoded and wrapped with a prefix & suffix `<Bytes>...T SCALE Encoded...</Bytes>` before signing.
pub struct BytesWrapped<'a, T>(pub &'a T);

const BYTES_PREFIX: &[u8] = b"<Bytes>";
const BYTES_SUFFIX: &[u8] = b"</Bytes>";

impl<'a, T: Encode> From<&'a T> for BytesWrapped<'a, T> {
    fn from(other: &'a T) -> Self {
        Self(other)
    }
}

impl<'a, T: Encode> Encode for BytesWrapped<'a, T> {
    fn size_hint(&self) -> usize {
        BYTES_PREFIX.len() + self.0.size_hint() + BYTES_SUFFIX.len()
    }
    fn encode_to<D: Output + ?Sized>(&self, dest: &mut D) {
        dest.write(BYTES_PREFIX);
        self.0.encode_to(dest);
        dest.write(BYTES_SUFFIX);
    }
}

/// Label for signing identity secondary key addition authorization.
pub const IDENTITY_ADD_SECONDARY_KEY_LABEL: &[u8] = b"Polymesh Identity Add Secondary Key";

/// Label for signing relay transactions.
pub const RELAY_TX_LABEL: &[u8] = b"Polymesh Relay Transaction";

/// Label for signing STO fundraiser receipts.
pub const STO_FUNDRAISER_RECEIPT_LABEL: &[u8] = b"Polymesh STO Fundraiser Receipt";

/// Label for signing settlement receipts.
pub const SETTLEMENT_RECEIPT_LABEL: &[u8] = b"Polymesh Settlement Receipt";

/// Chain scoped message for signing and verifying signatures. This is used to prevent signature replay attacks across different chains.
#[derive(Encode)]
pub struct ChainScopedMessage<T: frame_system::Config + pallet_timestamp::Config, M: Encode> {
    genesis_hash: T::Hash,
    nonce_or_id: u64,
    label: &'static [u8],
    expires_at: T::Moment,
    message: M,
}

impl<T: frame_system::Config + pallet_timestamp::Config, M: Encode> ChainScopedMessage<T, M> {
    /// Creates a new [`ChainScopedMessage`].
    pub fn new(
        nonce_or_id: u64,
        label: &'static [u8],
        expires_at: T::Moment,
        message: M,
    ) -> Option<ChainScopedMessage<T, M>> {
        // Check expiration
        let now = <pallet_timestamp::Pallet<T>>::get();
        if expires_at <= now {
            return None;
        }

        let genesis_hash = frame_system::Pallet::<T>::block_hash(BlockNumberFor::<T>::zero());
        Some(ChainScopedMessage {
            genesis_hash,
            nonce_or_id,
            label,
            expires_at,
            message,
        })
    }

    /// Create a new [`ChainScopedMessage`] unchecked.
    pub fn new_unchecked(
        nonce_or_id: u64,
        label: &'static [u8],
        expires_at: T::Moment,
        message: M,
    ) -> ChainScopedMessage<T, M> {
        let genesis_hash = frame_system::Pallet::<T>::block_hash(BlockNumberFor::<T>::zero());
        ChainScopedMessage {
            genesis_hash,
            nonce_or_id,
            label,
            expires_at,
            message,
        }
    }

    /// Verify a signature using the given public key and this chain scoped message.
    pub fn verify_signature<V>(
        &self,
        key: &<<V as Verify>::Signer as IdentifyAccount>::AccountId,
        signature: &V,
    ) -> bool
    where
        V: Verify,
    {
        verify_signature_common(key, signature, self)
    }

    /// Verify any signature (sr25519 or ed25519) using the given public key and this chain scoped message.
    pub fn verify_any_signature(&self, key: &T::AccountId, signature: H512) -> bool {
        verify_any_signature::<T, _>(key, signature, self)
    }
}

impl<T: frame_system::Config + pallet_timestamp::Config, M: Encode> ChainScopedMessage<T, M> {
    /// Used by benchmarks to sign a chain scoped message with the native schnorrkel implementation. This is not meant to be used in production code.
    pub fn native_sign(&self, raw_sk: &[u8]) -> Option<Signature> {
        let wrapped_message = BytesWrapped(self).encode();
        native_schnorrkel::sign(raw_sk, wrapped_message.as_slice())
    }
}

#[cfg(feature = "std")]
impl<T: frame_system::Config + pallet_timestamp::Config, M: Encode> ChainScopedMessage<T, M> {
    /// Used by unit tests to sign a message with a test key from the sr25519 keyring. This is not meant to be used in production code.
    pub fn sign(&self, key: &sp_keyring::Sr25519Keyring) -> Option<Signature> {
        let wrapped_message = BytesWrapped(self).encode();
        Some(key.sign(&wrapped_message))
    }
}

/// Verify any signature using the given public key and message.
/// This will try to verify the signature using both the sr25519 and ed25519 algorithms.
pub fn verify_any_signature<T: frame_system::Config, M: Encode>(
    key: &T::AccountId,
    signature: H512,
    message: &M,
) -> bool {
    let signature = AnySignature::from(Signature::from_h512(signature));

    if let Some(key) = Public::from_slice(&key.encode()).ok() {
        verify_signature_common(&key, &signature, message)
    } else {
        // It shouldn't be possible to fail to convert an `AccountId` to a `Public` key.
        false
    }
}

fn verify_signature_common<V, M>(
    key: &<<V as Verify>::Signer as IdentifyAccount>::AccountId,
    signature: &V,
    message: &M,
) -> bool
where
    V: Verify,
    M: Encode,
{
    // Try to verify the signature with a wrapped message.
    let wrapped_message = BytesWrapped(message).encode();
    signature.verify(wrapped_message.as_slice(), key)
}
