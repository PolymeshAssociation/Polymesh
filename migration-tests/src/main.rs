use polymesh_runtime::{DryRunRuntimeUpgrade, Runtime};
use remote_externalities::{Builder, CacheMode, CacheName};

async fn test_migration<F, G>(pre_tests: F, post_tests: G)
where
    F: FnOnce() -> (),
    G: FnOnce() -> (),
{
    let mut state = Builder::new()
        .uri("http://159.69.94.51:9933".into())
        .module("System")
        .cache_mode(CacheMode::None)
        .build()
        .await;
    state.execute_with(pre_tests);
    state.execute_with(<Runtime as DryRunRuntimeUpgrade>::dry_run_runtime_upgrade);
    state.execute_with(post_tests);
}

#[tokio::main]
async fn main() {
    test_migration(
        || {
            println!("pre");
        },
        || {
            println!("post");
        },
    )
    .await;
}
