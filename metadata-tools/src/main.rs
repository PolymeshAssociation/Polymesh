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

async fn load_metadata(path_or_url: &str) -> Result<RuntimeMetadata> {
    if let Some(_) = is_rpc_url(path_or_url) {
        let response: String = make_rpc_request(
            path_or_url,
            "state_getMetadata",
            jsonrpsee::core::rpc_params![],
        )
        .await?;
        let data = hex::decode(response.trim_start_matches("0x"))?;
        let prefixed = RuntimeMetadataPrefixed::decode(&mut &data[..])
            .context("Failed to decode metadata from RPC response")?;
        Ok(prefixed.1)
    } else {
        let data =
            read(path_or_url).with_context(|| format!("Failed to read file: {}", path_or_url))?;
        let prefixed = RuntimeMetadataPrefixed::decode(&mut &data[..])
            .with_context(|| format!("Failed to decode metadata from: {}", path_or_url))?;
        Ok(prefixed.1)
    }
}

async fn generate_default_filename(rpc_url: &str) -> Result<String> {
    let version = get_runtime_version(rpc_url).await?;
    Ok(format!(
        "{}_{}.meta",
        version.spec_name, version.spec_version
    ))
}

async fn download_metadata(rpc_url: &str, output_file: Option<&str>) -> Result<()> {
    let metadata = load_metadata(rpc_url).await?;
    let encoded_metadata = codec::Encode::encode(&RuntimeMetadataPrefixed(1, metadata));

    let output_path = match output_file {
        Some(path) => path.to_string(),
        None => generate_default_filename(rpc_url).await?,
    };

    write(&output_path, encoded_metadata)
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

#[tokio::main]
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

            let ra = ReducedRuntime::from(&metadata_a);
            let rb = ReducedRuntime::from(&metadata_b);

            let results = ReducedDiffResult::new(ra, rb);
            println!("{results}");

            // Exit with error code 1 if metadata is not compatible
            if !results.compatible() {
                process::exit(1);
            }
        }
        Commands::Download {
            rpc_url,
            output_file,
        } => {
            download_metadata(&rpc_url, output_file.as_deref()).await?;
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
    }

    Ok(())
}
