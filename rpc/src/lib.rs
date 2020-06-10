// This file is part of the Polymesh distribution (https://github.com/PolymathNetwork/Polymesh).
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

/// Helper macro to forward call to Api.
/// It also maps any error into an `RpcError`.
macro_rules! rpc_forward_call {
    ($self:ident, $at:ident, $f:expr, $err_msg: literal) => {{
        let api = $self.client.runtime_api();
        let at = BlockId::hash($at.unwrap_or_else(|| $self.client.info().best_hash));

        let result = $f(api, &at).map_err(|e| RpcError {
            code: ErrorCode::ServerError($crate::Error::RuntimeError as i64),
            message: $err_msg.into(),
            data: Some(format!("{:?}", e).into()),
        })?;

        Ok(result)
    }};
}

pub mod asset;
pub mod pips;
