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

#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};

use codec::{Decode, Encode};

/// Identity roles.
/// # TODO
/// 2. Review documents:
///     - [MESH-235](https://polymath.atlassian.net/browse/MESH-235)
///     - [Polymesh: Roles/Permissions](https://docs.google.com/document/d/12u-rMavow4fvidsFlLcLe7DAXuqWk8XUHOBV9kw05Z8/)
#[allow(missing_docs)]
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum IdentityRole {
    Issuer,
    SimpleTokenIssuer,
    Validator,
    ClaimIssuer,
    // From MESH-235
    Investor,
    NodeRunner,
    PM,
    CddAMLClaimIssuer,
    AccreditedInvestorClaimIssuer,
    VerifiedIdentityClaimIssuer,
    Custom(u8),
}
