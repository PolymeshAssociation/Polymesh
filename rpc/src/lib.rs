// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymath

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

/// Error type of this RPC api.
pub enum Error {
    /// The transaction was not decodable.
    DecodeError,
    /// The call to runtime failed.
    RuntimeError,
}

impl From<Error> for i32 {
    fn from(e: Error) -> i32 {
        match e {
            Error::RuntimeError => 1,
            Error::DecodeError => 2,
        }
    }
}

/// Helper macro to forward call to Api.
/// It also maps any error into an `RpcError`.
macro_rules! rpc_forward_call {
    ($self:ident, $at:ident, $f:expr, $err_msg: literal) => {{
        let api = $self.client.runtime_api();
        let at_hash = $at.unwrap_or_else(|| $self.client.info().best_hash);

        let result = $f(api, at_hash).map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                crate::Error::RuntimeError.into(),
                $err_msg,
                Some(e.to_string()),
            ))
        })?;

        Ok(result)
    }};
}

pub mod asset;
pub mod compliance_manager;
pub mod identity;
pub mod nft;
pub mod pips;
pub mod transaction_payment;
