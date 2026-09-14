use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use clap::{Parser, Subcommand};
use codec::{Decode, Encode};
use polymesh_worker::*;
use polymesh_worker_common::{
    BackendModuleKind, MODULE_CODE_SIZE_LIMIT, PROTOCOL_PDART, ProtocolInitializationMethod,
    ProtocolModuleConfig,
};
use sp_maybe_compressed_blob::*;

#[derive(Parser)]
#[command(about = "Build and inspect Polymesh worker protocol artifacts")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Compress a protocol module.
    Compress { module: PathBuf },
    /// Save the context produced by the built-in DART protocol.
    SaveContext,
    /// Build a SCALE-encoded DART release config from module artifacts.
    BuildReleaseConfig {
        #[arg(long, value_parser = parse_protocol_version)]
        protocol_version: ProtocolVersion,
        #[arg(long)]
        polkavm: PathBuf,
        #[arg(long)]
        wasm: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    /// Export a SCALE-encoded protocol config as JSON.
    ExportConfigJson {
        #[arg(long)]
        config: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    /// Check a release config against its module artifacts.
    CheckReleaseConfig {
        #[arg(long)]
        config: PathBuf,
        #[arg(long)]
        polkavm: PathBuf,
        #[arg(long)]
        wasm: PathBuf,
    },
}

fn compress_file(file_path: &Path) -> Result<()> {
    let file_contents = std::fs::read(file_path)
        .with_context(|| format!("Failed to read {}", file_path.display()))?;
    let compressed = compress_strongly(&file_contents, MODULE_CODE_SIZE_LIMIT)
        .with_context(|| format!("Failed to compress {}", file_path.display()))?;
    let compressed_file_path = PathBuf::from(format!("{}.zst", file_path.display()));
    std::fs::write(&compressed_file_path, compressed)
        .with_context(|| format!("Failed to write {}", compressed_file_path.display()))?;
    Ok(())
}

fn parse_protocol_version(value: &str) -> Result<ProtocolVersion, String> {
    let parts = value
        .split('.')
        .map(|part| part.parse::<u16>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "protocol version must use MAJOR.MINOR.PATCH".to_owned())?;
    if parts.len() != 3 {
        return Err("protocol version must use MAJOR.MINOR.PATCH".to_owned());
    }
    Ok(ProtocolVersion::new(parts[0], parts[1], parts[2]))
}

fn module_definition(
    module_kind: BackendModuleKind,
    module_version: u32,
    code: &[u8],
) -> BackendModuleDefinition {
    BackendModuleDefinition {
        module_kind,
        module_version,
        code_hash: sp_core::blake2_256(code),
    }
}

fn build_release_config(
    protocol_version: ProtocolVersion,
    polkavm_compressed: &[u8],
    wasm_compressed: &[u8],
) -> Result<ProtocolModuleConfig> {
    let protocol = Protocol::new(PROTOCOL_PDART, protocol_version);
    let polkavm = decompress_module(polkavm_compressed, "PolkaVM")?;
    let wasm = decompress_module(wasm_compressed, "Wasm")?;
    Ok(ProtocolModuleConfig {
        protocol,
        initialization_method: ProtocolInitializationMethod::SaveContextFromFirstInstance,
        modules: vec![
            module_definition(BackendModuleKind::PolkaVM, 2, polkavm_compressed),
            module_definition(BackendModuleKind::PolkaVM, 1, &polkavm),
            module_definition(BackendModuleKind::Wasm, 2, wasm_compressed),
            module_definition(BackendModuleKind::Wasm, 1, &wasm),
            module_definition(BackendModuleKind::Native, 1, &protocol.encode()),
        ],
    })
}

fn decompress_module(code: &[u8], kind: &str) -> Result<Vec<u8>> {
    decompress(code, MODULE_CODE_SIZE_LIMIT)
        .map(|code| code.to_vec())
        .with_context(|| format!("Failed to decompress {kind} module"))
}

fn read_config(path: &Path) -> Result<ProtocolModuleConfig> {
    let bytes = read_file(path)?;
    let mut input = bytes.as_slice();
    let config = ProtocolModuleConfig::decode(&mut input)
        .with_context(|| format!("Failed to decode {}", path.display()))?;
    ensure!(input.is_empty(), "{} has trailing bytes", path.display());
    Ok(config)
}

fn read_file(path: &Path) -> Result<Vec<u8>> {
    std::fs::read(path).with_context(|| format!("Failed to read {}", path.display()))
}

fn check_release_config(
    config: &ProtocolModuleConfig,
    polkavm_compressed: &[u8],
    wasm_compressed: &[u8],
) -> Result<()> {
    ensure!(
        config.protocol.id == PROTOCOL_PDART,
        "config is not for DART"
    );
    ensure!(
        config.modules.len() == 5,
        "config must contain exactly five modules"
    );

    let native_code = config.protocol.encode();
    let polkavm = decompress_module(polkavm_compressed, "PolkaVM")?;
    let wasm = decompress_module(wasm_compressed, "Wasm")?;
    for (kind, version, code) in [
        (BackendModuleKind::PolkaVM, 2, polkavm_compressed),
        (BackendModuleKind::PolkaVM, 1, polkavm.as_slice()),
        (BackendModuleKind::Wasm, 2, wasm_compressed),
        (BackendModuleKind::Wasm, 1, wasm.as_slice()),
        (BackendModuleKind::Native, 1, native_code.as_slice()),
    ] {
        let matching: Vec<_> = config
            .modules
            .iter()
            .filter(|module| module.module_kind == kind && module.module_version == version)
            .collect();
        ensure!(
            matching.len() == 1,
            "config must contain exactly one {kind:?} version {version} module"
        );
        let module = matching[0];
        ensure!(
            module.code_hash == sp_core::blake2_256(code),
            "{kind:?} code hash does not match"
        );
    }
    Ok(())
}

fn save_protocol_context() {
    let native_backend = polymesh_worker_native::NativeBackend;
    let backends = backend::Backends::new(Some(Box::new(native_backend)));

    let mut loader = StaticModules::new();
    let protocol = Protocol {
        id: PROTOCOL_PDART,
        version: ProtocolVersion {
            major: 0,
            minor: 1,
            patch: 0,
        },
    };

    // Load the module.
    let now = std::time::Instant::now();
    let module = backends
        .load_module(protocol, &mut loader)
        .expect("No backend available for the given protocol and version");
    println!("Module loaded in: {:?}", now.elapsed());

    // Instantiate the module.
    let now = std::time::Instant::now();
    let mut instance = module.instantiate().expect("Failed to instantiate module");
    println!("Module instantiated in: {:?}", now.elapsed());

    // Initialize the module.
    {
        let now = std::time::Instant::now();
        instance
            .initialize(None)
            .expect("Failed to initialize module");
        println!("Module initialized in: {:?}", now.elapsed());
    }

    // Save the module context.
    let saved_ctx = {
        let now = std::time::Instant::now();
        let save_result = instance
            .save_context()
            .expect("Worker error during context saving");
        println!("Context saved: {}", save_result.is_some());
        println!("Time taken for saving context: {:?}", now.elapsed());

        save_result.expect("Context saving is not supported by the backend")
    };
    let hash = sp_core::blake2_256(&saved_ctx);
    println!("Saved context size: {} bytes", saved_ctx.len());
    println!("Saved context hash: 0x{}", hex::encode(hash));

    // Test loading the context back into a new instance.
    {
        let now = std::time::Instant::now();
        instance
            .initialize(Some(saved_ctx.as_ref()))
            .expect("Failed to initialize with saved context");
        println!("Time taken for loading context: {:?}", now.elapsed());
    }

    // Save the context to a file.
    let saved_ctx_file_path = "polymesh-worker-protocol-dart-v1.context.bin";
    std::fs::write(&saved_ctx_file_path, saved_ctx)
        .expect("Failed to write the saved context to a file");
}

pub fn main() -> Result<()> {
    env_logger::init();

    match Cli::parse().command {
        Command::Compress { module } => compress_file(&module)?,
        Command::SaveContext => save_protocol_context(),
        Command::BuildReleaseConfig {
            protocol_version,
            polkavm,
            wasm,
            output,
        } => {
            let config =
                build_release_config(protocol_version, &read_file(&polkavm)?, &read_file(&wasm)?)?;
            std::fs::write(&output, config.encode())
                .with_context(|| format!("Failed to write {}", output.display()))?;
            println!(
                "Config hash: 0x{}",
                hex::encode(config.hash_using(sp_core::blake2_256))
            );
        }
        Command::ExportConfigJson { config, output } => {
            let config = read_config(&config)?;
            let json =
                serde_json::to_vec_pretty(&config).context("Failed to encode config JSON")?;
            std::fs::write(&output, json)
                .with_context(|| format!("Failed to write {}", output.display()))?;
        }
        Command::CheckReleaseConfig {
            config,
            polkavm,
            wasm,
        } => {
            let config = read_config(&config)?;
            check_release_config(&config, &read_file(&polkavm)?, &read_file(&wasm)?)?;
            println!("Config and module hashes match");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_config_contains_raw_and_compressed_module_versions() {
        let polkavm = compress_strongly(b"polkavm", MODULE_CODE_SIZE_LIMIT).unwrap();
        let wasm = compress_strongly(b"wasm", MODULE_CODE_SIZE_LIMIT).unwrap();
        let config = build_release_config(ProtocolVersion::new(1, 2, 3), &polkavm, &wasm).unwrap();

        check_release_config(&config, &polkavm, &wasm).unwrap();
        assert_eq!(config.modules[0].code_hash, sp_core::blake2_256(&polkavm));
        assert_eq!(config.modules[1].code_hash, sp_core::blake2_256(b"polkavm"));
        assert_eq!(config.modules[2].code_hash, sp_core::blake2_256(&wasm));
        assert_eq!(config.modules[3].code_hash, sp_core::blake2_256(b"wasm"));
        assert_eq!(
            config
                .modules
                .iter()
                .map(|module| (module.module_kind, module.module_version))
                .collect::<Vec<_>>(),
            vec![
                (BackendModuleKind::PolkaVM, 2),
                (BackendModuleKind::PolkaVM, 1),
                (BackendModuleKind::Wasm, 2),
                (BackendModuleKind::Wasm, 1),
                (BackendModuleKind::Native, 1),
            ]
        );
        let json = serde_json::to_value(&config).unwrap();
        assert!(json["modules"].as_array().unwrap().iter().all(|module| {
            let hash = module["code_hash"].as_str().unwrap();
            hash.starts_with("0x") && hash.len() == 66
        }));
        assert_eq!(
            serde_json::from_value::<ProtocolModuleConfig>(json)
                .unwrap()
                .encode(),
            config.encode()
        );
        assert_eq!(
            ProtocolModuleConfig::decode(&mut config.encode().as_slice())
                .unwrap()
                .encode(),
            config.encode()
        );
    }

    #[test]
    fn release_config_check_rejects_changed_code() {
        let polkavm = compress_strongly(b"polkavm", MODULE_CODE_SIZE_LIMIT).unwrap();
        let wasm = compress_strongly(b"wasm", MODULE_CODE_SIZE_LIMIT).unwrap();
        let config = build_release_config(ProtocolVersion::new(1, 0, 0), &polkavm, &wasm).unwrap();
        let changed = compress_strongly(b"changed", MODULE_CODE_SIZE_LIMIT).unwrap();
        assert!(check_release_config(&config, &changed, &wasm).is_err());
    }

    #[test]
    fn release_config_check_rejects_missing_module_version() {
        let polkavm = compress_strongly(b"polkavm", MODULE_CODE_SIZE_LIMIT).unwrap();
        let wasm = compress_strongly(b"wasm", MODULE_CODE_SIZE_LIMIT).unwrap();
        let mut config =
            build_release_config(ProtocolVersion::new(1, 0, 0), &polkavm, &wasm).unwrap();
        config.modules.pop();
        assert!(check_release_config(&config, &polkavm, &wasm).is_err());
    }
}
