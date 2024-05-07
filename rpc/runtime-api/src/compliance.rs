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

//! Runtime API definition for Compliance module.

use frame_support::dispatch::DispatchError;

use polymesh_primitives::compliance_manager::ComplianceReport;
use polymesh_primitives::{IdentityId, Ticker};

sp_api::decl_runtime_apis! {

    pub trait ComplianceApi {
        /// Checks all compliance requirements for the given ticker
        ///
        /// ```ignore
        /// curl http://localhost:9933 -H "Content-Type: application/json" -d '{
        ///     "id":1,
        ///     "jsonrpc":"2.0",
        ///     "method": "compliance_complianceReport",
        ///     "params":[
        ///       { "did": "0x0100000000000000000000000000000000000000000000000000000000000000", "kind": "Default"},
        ///       { "did": "0x0200000000000000000000000000000000000000000000000000000000000000", "kind": "Default"},
        ///       { "ticker": "0x5449434B4552303030303031", "ids": [1]}
        ///     ]
        ///   }'
        /// ```
        fn compliance_report(
            ticker: &Ticker,
            sender_identity: &IdentityId,
            receiver_identity: &IdentityId
        ) -> Result<ComplianceReport, DispatchError>;
    }
}
