use polymesh_runtime::{DryRunRuntimeUpgrade, Runtime};
use polymesh_runtime_old::Runtime as RuntimeOld;
use remote_externalities::{Builder, CacheConfig, Mode, OfflineConfig, OnlineConfig};
use std::time::Instant;
use futures::executor::block_on;

type StakingOld = pallet_staking_old::Module<RuntimeOld>;
type Staking = pallet_staking::Module<Runtime>;

pub fn test_migration<F, G>(pre_tests: F, post_tests: G)
where
    F: FnOnce() -> (),
    G: FnOnce() -> (),
{
    let mode = if std::path::Path::new(".").join("CACHE").exists() {
        Mode::Offline(OfflineConfig::default())
    } else {
        Mode::Online(OnlineConfig {
            uri: "http://159.69.94.51:9933".into(),
            cache: Some(CacheConfig::default()),
            ..Default::default()
        })
    };

    let mut state = block_on(Builder::new().mode(mode).build());

    state.execute_with(pre_tests);

    let now = Instant::now();
    state.execute_with(<Runtime as DryRunRuntimeUpgrade>::dry_run_runtime_upgrade);
    let elapsed = now.elapsed();
    println!("Storage Migration took: {:#?}", elapsed);

    state.execute_with(post_tests);
}

fn main() {
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
