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

use alloc::vec::Vec;

use pallet_revive::precompiles::Error;
use pallet_revive::precompiles::Ext;

use polymesh_precompiles::IPolymeshRuntime;

use crate::common::Common;
use crate::interface::PolymeshRuntimeInterface;
use crate::Config;

impl<T: Config> PolymeshRuntimeInterface<T> {
    /// Accepts a pending `BecomeAgent` authorization.
    pub(crate) fn accept_become_agent(
        call: &IPolymeshRuntime::externalAgentsAcceptBecomeAgentCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let caller = Common::<T>::caller(env)?;

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            pallet_external_agents::Call::<T>::accept_become_agent {
                auth_id: call.authId,
            },
        )?;

        Ok(Vec::new())
    }
}
