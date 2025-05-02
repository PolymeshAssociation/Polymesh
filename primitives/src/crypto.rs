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
use sp_core::{
    crypto::ByteArray,
    sr25519::{Public, Signature},
    H512,
};
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_runtime::AnySignature;
use sp_runtime_interface::runtime_interface;

/// Native interface for runtime module to use some Schnorrkel functionality.
#[runtime_interface]
pub trait NativeSchnorrkel {
    /// Sign the message `message`, using the given secret key.
    /// It returns `None` if the secret key cannot be created from the input raw bytes.
    fn sign(raw_sk: [u8; 64], message: &[u8]) -> Option<Signature> {
        use schnorrkel::{keys::SecretKey, signing_context, Keypair};
        const SIGNING_CTX: &[u8] = b"substrate";

        SecretKey::from_bytes(&raw_sk[..])
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

/// Verify any signature using the given public key and message.
/// This will try to verify the signature using both the sr25519 and ed25519 algorithms.
pub fn verify_any_signature<T: frame_system::Config, M: Encode>(
    key: &T::AccountId,
    signature: H512,
    message: &M,
    only_wrapped: bool,
) -> bool {
    let signature = AnySignature::from(Signature::from_h512(signature));

    if let Some(key) = Public::from_slice(&key.encode()).ok() {
        verify_signature_common(&key, &signature, message, only_wrapped)
    } else {
        // It shouldn't be possible to fail to convert an `AccountId` to a `Public` key.
        false
    }
}

/// Verify a signature using the given public key and message.
pub fn verify_signature<T, V, M>(
    key: &T::AccountId,
    signature: &V,
    message: &M,
    only_wrapped: bool,
) -> bool
where
    T: frame_system::Config,
    V: Verify<Signer: IdentifyAccount<AccountId = T::AccountId>>,
    M: Encode,
{
    verify_signature_common(key, signature, message, only_wrapped)
}

fn verify_signature_common<V, M>(
    key: &<<V as Verify>::Signer as IdentifyAccount>::AccountId,
    signature: &V,
    message: &M,
    only_wrapped: bool,
) -> bool
where
    V: Verify,
    M: Encode,
{
    // Try to verify the signature with a wrapped message.
    let wrapped_message = BytesWrapped(message).encode();
    if signature.verify(wrapped_message.as_slice(), key) {
        true
    } else if only_wrapped {
        // We only accept wrapped messages.
        false
    } else {
        // Try old legacy verification with the raw message.
        let encoded = message.encode();
        signature.verify(encoded.as_slice(), key)
    }
}
