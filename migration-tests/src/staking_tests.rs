use crate::test_migration;
use polymesh_runtime::Runtime;
use polymesh_runtime_old::Runtime as RuntimeOld;

type StakingOld = pallet_staking_old::Module<RuntimeOld>;
type Staking = pallet_staking::Module<Runtime>;

#[test]
fn first_test() {
    test_migration(
        || {
            println!(
                "Validator count before migration: {:?}",
                StakingOld::validator_count()
            );
        },
        || {
            println!(
                "Validator count after migration: {:?}",
                Staking::validator_count()
            );
        },
    );
}
