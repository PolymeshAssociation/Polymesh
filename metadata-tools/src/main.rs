use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use codec::Decode;
use frame_metadata::{RuntimeMetadata, RuntimeMetadataPrefixed};
use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::HttpClientBuilder;
use jsonrpsee::ws_client::WsClientBuilder;
use serde::de::DeserializeOwned;
use sp_version::RuntimeVersion;
use std::fs::{read, write};
use std::path::PathBuf;
use std::process;
use substrate_differ::differs::reduced::reduced_diff_result::ReducedDiffResult;
use substrate_differ::differs::reduced::reduced_runtime::ReducedRuntime;

#[derive(Parser)]
#[command(name = "metadata-tools")]
#[command(about = "Compare or download Substrate metadata")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum RuntimeVersionField {
    #[value(name = "spec_name")]
    SpecName,
    #[value(name = "spec_version")]
    SpecVersion,
    #[value(name = "impl_version")]
    ImplVersion,
    #[value(name = "transaction_version")]
    TransactionVersion,
    #[value(name = "state_version")]
    StateVersion,
}

#[derive(Subcommand)]
enum Commands {
    /// Compare two Substrate metadata sources for differences
    Diff {
        /// First metadata source to compare (file path or RPC URL)
        #[arg(value_name = "METADATA_A")]
        metadata_a: String,

        /// Second metadata source to compare (file path or RPC URL)
        #[arg(value_name = "METADATA_B")]
        metadata_b: String,
    },
    /// Download metadata from a Substrate node and save it to a file
    Download {
        /// RPC URL of the Substrate node
        #[arg(value_name = "RPC_URL")]
        rpc_url: String,

        /// Output file path to save the metadata (default: {spec_name}_{spec_version}.meta)
        #[arg(value_name = "OUTPUT_FILE")]
        output_file: Option<String>,

        /// Output folder to save metadata in a structured format: {output_folder}/{spec_name}/{spec_version}.meta
        #[arg(long, value_name = "OUTPUT_FOLDER")]
        output_folder: Option<String>,
    },
    /// Get runtime version from a Substrate node
    RuntimeVersion {
        /// RPC URL of the Substrate node
        #[arg(value_name = "RPC_URL")]
        rpc_url: String,
        /// Select a specific field to display
        #[arg(long = "field", value_enum)]
        field: Option<RuntimeVersionField>,
    },
    /// Check compatibility between node metadata and stored metadata
    Check {
        /// Folder containing stored metadata files
        #[arg(value_name = "METADATA_FOLDER")]
        metadata_folder: String,

        /// RPC URL of the Substrate node (default: ws://localhost:9944)
        #[arg(value_name = "RPC_URL", default_value = "ws://localhost:9944")]
        rpc_url: String,
    },
}

fn is_rpc_url(url: &str) -> Option<&'static str> {
    if url.starts_with("ws://") || url.starts_with("wss://") {
        Some("ws")
    } else if url.starts_with("http://") || url.starts_with("https://") {
        Some("http")
    } else {
        None
    }
}

async fn make_rpc_request<R: DeserializeOwned>(
    url: &str,
    method: &str,
    params: jsonrpsee::core::params::ArrayParams,
) -> Result<R> {
    match is_rpc_url(url) {
        Some("ws") => {
            let client = WsClientBuilder::default().build(url).await?;
            client.request(method, params).await.map_err(Into::into)
        }
        Some("http") => {
            let client = HttpClientBuilder::default().build(url)?;
            client.request(method, params).await.map_err(Into::into)
        }
        _ => Err(anyhow::anyhow!("Not an RPC URL: {}", url)),
    }
}

async fn read_metadata(path_or_url: &str) -> Result<Vec<u8>> {
    if let Some(_) = is_rpc_url(path_or_url) {
        let response: String = make_rpc_request(
            path_or_url,
            "state_getMetadata",
            jsonrpsee::core::rpc_params![],
        )
        .await?;
        Ok(hex::decode(response.trim_start_matches("0x"))?)
    } else {
        Ok(read(path_or_url).with_context(|| format!("Failed to read file: {}", path_or_url))?)
    }
}

async fn load_metadata(path_or_url: &str) -> Result<RuntimeMetadata> {
    let data = read_metadata(path_or_url).await?;
    let prefixed = RuntimeMetadataPrefixed::decode(&mut &data[..])
        .with_context(|| format!("Failed to decode metadata from: {}", path_or_url))?;
    Ok(prefixed.1)
}

async fn generate_default_filename(rpc_url: &str, output_folder: Option<&str>) -> Result<String> {
    let version = get_runtime_version(rpc_url).await?;

    match output_folder {
        Some(folder) => {
            let spec_folder = format!("{}/{}", folder, version.spec_name);
            std::fs::create_dir_all(&spec_folder)?;
            Ok(format!("{}/{}.meta", spec_folder, version.spec_version))
        }
        None => Ok(format!(
            "{}_{}.meta",
            version.spec_name, version.spec_version
        )),
    }
}

async fn download_metadata(
    rpc_url: &str,
    output_file: Option<&str>,
    output_folder: Option<&str>,
) -> Result<()> {
    let metadata = read_metadata(rpc_url).await?;

    let output_path = match output_file {
        Some(path) => path.to_string(),
        None => generate_default_filename(rpc_url, output_folder).await?,
    };

    write(&output_path, metadata)
        .with_context(|| format!("Failed to write file: {}", output_path))?;
    println!("{output_path}");
    Ok(())
}

async fn get_runtime_version(rpc_url: &str) -> Result<RuntimeVersion> {
    make_rpc_request(
        rpc_url,
        "state_getRuntimeVersion",
        jsonrpsee::core::rpc_params![],
    )
    .await
}

fn find_latest_metadata(metadata_folder: &str, spec_name: &str) -> Result<Option<PathBuf>> {
    let spec_folder = PathBuf::from(metadata_folder).join(spec_name);
    if !spec_folder.exists() {
        return Ok(None);
    }

    let mut versions: Vec<(u32, PathBuf)> = Vec::new();
    for entry in std::fs::read_dir(spec_folder)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            if let Ok(version) = stem.parse::<u32>() {
                versions.push((version, path));
            }
        }
    }

    if versions.is_empty() {
        Ok(None)
    } else {
        versions.sort_by_key(|(v, _)| *v);
        Ok(Some(versions.last().unwrap().1.clone()))
    }
}

fn find_version_metadata(
    metadata_folder: &str,
    spec_name: &str,
    spec_version: u32,
) -> Result<Option<PathBuf>> {
    let path = PathBuf::from(metadata_folder)
        .join(spec_name)
        .join(format!("{}.meta", spec_version));

    if path.exists() {
        Ok(Some(path))
    } else {
        Ok(None)
    }
}

fn diff_metadata(metadata_a: RuntimeMetadata, metadata_b: RuntimeMetadata) -> Result<()> {
    let ra = ReducedRuntime::from(&metadata_a);
    let rb = ReducedRuntime::from(&metadata_b);

    let results = ReducedDiffResult::new(ra, rb);
    println!("{results}");

    if !results.compatible() {
        process::exit(1);
    }
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Diff {
            metadata_a,
            metadata_b,
        } => {
            let metadata_a = load_metadata(&metadata_a).await?;
            let metadata_b = load_metadata(&metadata_b).await?;
            diff_metadata(metadata_a, metadata_b)?;
        }
        Commands::Download {
            rpc_url,
            output_file,
            output_folder,
        } => {
            download_metadata(&rpc_url, output_file.as_deref(), output_folder.as_deref()).await?;
        }
        Commands::RuntimeVersion { rpc_url, field } => {
            let version = get_runtime_version(&rpc_url).await?;

            match field {
                Some(RuntimeVersionField::SpecName) => println!("{}", version.spec_name),
                Some(RuntimeVersionField::SpecVersion) => println!("{}", version.spec_version),
                Some(RuntimeVersionField::ImplVersion) => println!("{}", version.impl_version),
                Some(RuntimeVersionField::TransactionVersion) => {
                    println!("{}", version.transaction_version)
                }
                Some(RuntimeVersionField::StateVersion) => println!("{}", version.state_version),
                None => {
                    println!("spec_name: {}", version.spec_name);
                    println!("spec_version: {}", version.spec_version);
                    println!("impl_version: {}", version.impl_version);
                    println!("transaction_version: {}", version.transaction_version);
                    println!("state_version: {}", version.state_version);
                }
            }
        }
        Commands::Check {
            metadata_folder,
            rpc_url,
        } => {
            let version = get_runtime_version(&rpc_url).await?;
            let current_metadata = load_metadata(&rpc_url).await?;

            let stored_path =
                find_version_metadata(&metadata_folder, &version.spec_name, version.spec_version)?
                    .or_else(|| {
                        find_latest_metadata(&metadata_folder, &version.spec_name)
                            .ok()
                            .flatten()
                    })
                    .context("No metadata file found")?;

            println!(
                "Comparing node spec version {} with stored spec version {}",
                version.spec_version,
                stored_path.file_stem().unwrap().to_str().unwrap()
            );
            println!("Using stored metadata: {}", stored_path.display());
            let stored_metadata = load_metadata(stored_path.to_str().unwrap()).await?;

            diff_metadata(stored_metadata, current_metadata)?;
        }
    }

    Ok(())
}
