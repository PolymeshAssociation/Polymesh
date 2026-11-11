use std::{env, fs, path::Path, process::Command};

fn build_module(root: &Path, out: &Path, target: &str, package: &str, features: &str) {
    let target_dir = out.join("native-target");
    let mut command = Command::new(env::var_os("CARGO").expect("CARGO is set by Cargo"));
    command
        .current_dir(root)
        .env("CARGO_TARGET_DIR", &target_dir)
        .env(
            "CARGO_ENCODED_RUSTFLAGS",
            "-C\x1fpanic=abort\x1f-C\x1fstrip=symbols\x1f-C\x1fcodegen-units=1",
        )
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTDOCFLAGS")
        .env_remove("RUSTC_WRAPPER")
        .env_remove("RUSTC_WORKSPACE_WRAPPER")
        .args([
            "rustc",
            "--locked",
            "--crate-type",
            "cdylib",
            "-Z",
            "build-std=core,alloc",
            "--target",
            target,
            "--no-default-features",
            "--features",
            features,
            "--release",
            "--lib",
            "-p",
            package,
        ]);
    if target == "aarch64-unknown-linux-gnu" && env::var("HOST").as_deref() != Ok(target) {
        let linker = env::var_os("CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER")
            .unwrap_or_else(|| "aarch64-linux-gnu-gcc".into());
        command.env("CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER", &linker);
        if env::var_os("CC_aarch64_unknown_linux_gnu").is_none() {
            command.env("CC_aarch64_unknown_linux_gnu", &linker);
        }
    }
    let status = command.status().expect("launch native module source build");
    assert!(
        status.success(),
        "native module source build failed: {package}"
    );

    let library = target_dir
        .join(target)
        .join("release")
        .join(format!("lib{}.so", package.replace('-', "_")));
    let bytes = fs::read(&library).expect("read freshly built native module");
    let compressed = sp_maybe_compressed_blob::compress_strongly(&bytes, 10 * 1024 * 1024)
        .expect("native module exceeds the module code size limit");
    fs::write(out.join(format!("{package}.so.zst")), compressed)
        .expect("write compressed native module");
}

fn main() {
    let out = env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo");
    let out = Path::new(&out);
    println!(
        "cargo:rerun-if-changed={}",
        out.join("always-check-native-sources").display()
    );
    let target = env::var("TARGET").expect("TARGET is set by Cargo");
    if !matches!(
        target.as_str(),
        "x86_64-unknown-linux-gnu" | "aarch64-unknown-linux-gnu"
    ) {
        // Non-GNU Linux targets (e.g. musl) and other platforms use the
        // in-process native backend instead; no module is built or embedded.
        return;
    }
    let manifest = env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set by Cargo");
    let root = Path::new(&manifest).parent().unwrap().parent().unwrap();
    let features = if env::var_os("CARGO_FEATURE_TESTING").is_some() {
        "native_module,testing"
    } else {
        "native_module"
    };
    build_module(
        root,
        out,
        &target,
        "polymesh-worker-protocol-dart-v1",
        features,
    );
    build_module(
        root,
        out,
        &target,
        "polymesh-worker-protocol-testing",
        "native_module,testing,version_v0",
    );
}
