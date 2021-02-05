use futures::executor::block_on;
use polymesh_runtime::{DryRunRuntimeUpgrade, Runtime};
use remote_externalities::{Builder, CacheConfig, Mode, OfflineConfig, OnlineConfig};
use std::{sync::Mutex, time::Instant};

#[macro_use]
extern crate lazy_static;

#[cfg(test)]
mod staking_tests;

struct TestState;

impl TestState {
    fn init() {
        if !std::path::Path::new(".").join("CACHE").exists() {
            block_on(
                Builder::new()
                    .mode(Mode::Online(OnlineConfig {
                        uri: "http://159.69.94.51:9933".into(),
                        cache: Some(CacheConfig::default()),
                        ..Default::default()
                    }))
                    .build(),
            );
        }
    }
}

lazy_static! {
    static ref TEST_STATE: Mutex<()> = Mutex::new(TestState::init());
}

/// This is the main helper function for writing migration tests,
/// This function takes two closures as input. It first executes the first closure on current blockchain state.
/// It then executes the storage migrations and finally it executes the second closure.
pub fn test_migration<F, G>(pre_tests: F, post_tests: G)
where
    F: FnOnce() -> (),
    G: FnOnce() -> (),
{
    lazy_static::initialize(&TEST_STATE);

    let mut state = block_on(
        Builder::new()
            .mode(Mode::Offline(OfflineConfig::default()))
            .build(),
    );

    state.execute_with(pre_tests);

    let now = Instant::now();
    state.execute_with(<Runtime as DryRunRuntimeUpgrade>::dry_run_runtime_upgrade);
    let elapsed = now.elapsed();
    println!("Storage Migrations took: {:#?}", elapsed);

    state.execute_with(post_tests);
}
