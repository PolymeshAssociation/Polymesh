use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use codec::Decode;
use frame_metadata::{RuntimeMetadata, RuntimeMetadataPrefixed};
use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::HttpClientBuilder;
use jsonrpsee::ws_client::WsClientBuilder;
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

        /// Output file path to save the metadata
        #[arg(value_name = "OUTPUT_FILE")]
        output_file: String,
    },
}

async fn load_metadata(path_or_url: &str) -> Result<RuntimeMetadata> {
    if path_or_url.starts_with("http://") || path_or_url.starts_with("https://") {
        let client = HttpClientBuilder::default().build(path_or_url)?;
        let response: String = client
            .request("state_getMetadata", jsonrpsee::core::rpc_params![])
            .await?;
        let data = hex::decode(response.trim_start_matches("0x"))?;
        let prefixed = RuntimeMetadataPrefixed::decode(&mut &data[..])
            .context("Failed to decode metadata from RPC response")?;
        Ok(prefixed.1)
    } else if path_or_url.starts_with("ws://") || path_or_url.starts_with("wss://") {
        let client = WsClientBuilder::default().build(path_or_url).await?;
        let response: String = client
            .request("state_getMetadata", jsonrpsee::core::rpc_params![])
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

async fn download_metadata(rpc_url: &str, output_file: &str) -> Result<()> {
    let metadata = load_metadata(rpc_url).await?;
    let encoded_metadata = codec::Encode::encode(&RuntimeMetadataPrefixed(1, metadata));
    write(output_file, encoded_metadata)
        .with_context(|| format!("Failed to write file: {}", output_file))?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
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
            download_metadata(&rpc_url, &output_file).await?;
        }
    }

    Ok(())
}
