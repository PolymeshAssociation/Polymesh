// This file is part of the Polymesh distribution (https://github.com/PolymathNetwork/Polymesh).
// Copyright (c) 2020 Polymath
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.
//
// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use polymesh_primitives::{DispatchableName, IdentityId, PalletName};

/// Permissions module configuration trait.
pub trait PermissionChecker: frame_system::Trait {
    /// The type that implements the permission check function.
    type Checker: CheckAccountCallPermissions<Self::AccountId>;
}

/// A permission checker for calls from accounts to extrinsics.
pub trait CheckAccountCallPermissions<AccountId> {
    /// Checks whether `who` can call the current extrinsic represented by `pallet_name` and
    /// `function_name`.
    ///
    /// Returns:
    ///
    /// - `Some(did)` where `did` is the idenitity ID on behalf of which the caller is allowed to
    /// make tis call.
    ///
    /// - `None` if there is no such identity.
    fn check_account_call_permissions(
        who: &AccountId,
        pallet_name: &PalletName,
        function_name: &DispatchableName,
    ) -> Option<IdentityId>;
}
