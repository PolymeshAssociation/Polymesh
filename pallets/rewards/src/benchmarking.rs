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

use crate::*;

use codec::{Decode, Encode};
use frame_benchmarking::benchmarks;
use polymesh_common_utilities::{
    benchs::{user, AccountIdOf},
    traits::TestUtilsFn,
};
use sp_runtime::MultiSignature;

type Rewards<T> = crate::Module<T>;

pub(crate) const SEED: u32 = 0;

benchmarks! {
    where_clause { where T: Config, T: TestUtilsFn<AccountIdOf<T>> }

    claim_itn_reward {
        // Construct a mainnet user and an ITN one.
        let mainnet = user::<T>("alice", SEED);
        let mainnet_acc = mainnet.account();
        let itn = user::<T>("bob", SEED);
        let itn_acc = itn.account();

        // Endow rewards pot with sufficient balance to withdraw from.
        let _ = T::Currency::deposit_into_existing(&<Rewards<T>>::account_id(), (2 * ONE_POLY).try_into().ok().unwrap());

        // Register a reward for the ITN account.
        <ItnRewards<T>>::insert(&itn_acc, ItnRewardStatus::Unclaimed(ONE_POLY));

        // Create the signature needed to claim the reward.
        let mut msg = [0u8; 48];
        msg[..32].copy_from_slice(&mainnet_acc.encode());
        msg[32..].copy_from_slice(b"claim_itn_reward");
        let sig = MultiSignature::Sr25519(itn.sign(&msg).unwrap()).encode();
        let sig = Decode::decode(&mut sig.as_slice()).unwrap();

        let itn_acc2 = itn_acc.clone();
    }: _(RawOrigin::None, mainnet_acc, itn_acc, sig)
    verify {
        assert_eq!(<ItnRewards<T>>::get(&itn_acc2), Some(ItnRewardStatus::Claimed));
    }

    set_itn_reward_status {
        let user = user::<T>("alice", SEED);
        let user_acc = user.account();
    }: _(RawOrigin::Root, user_acc, ItnRewardStatus::Claimed)
    verify {
        assert!(<ItnRewards<T>>::get(&user.account()).is_some());
    }
}
