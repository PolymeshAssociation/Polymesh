# Metadata Tools

A command-line utility for working with Substrate runtime metadata. This tool helps you download, compare, and verify metadata from Substrate-based chains.

## Features

- Download metadata from a running node
- Compare metadata between two sources (files or nodes)
- Get runtime version information
- Check compatibility between node metadata and stored metadata

## Usage

### Subcommands

#### diff

Compare two Substrate metadata sources for differences.

**Usage:**

```bash
metadata-tools diff <METADATA_A> <METADATA_B>
```

- `<METADATA_A>`: First metadata source to compare (file path or RPC URL).
- `<METADATA_B>`: Second metadata source to compare (file path or RPC URL).

#### download

Download metadata from a Substrate node and save it to a file.

**Usage:**

```bash
metadata-tools download <RPC_URL> [OUTPUT_FILE] [--output-folder OUTPUT_FOLDER]
```

- `<RPC_URL>`: RPC URL of the Substrate node.
- `[OUTPUT_FILE]`: (Optional) Output file path to save the metadata (default: `{spec_name}_{spec_version}.meta`).
- `--output-folder OUTPUT_FOLDER`: (Optional) Output folder to save metadata in a structured format: `{output_folder}/{spec_name}/{spec_version}.meta`.

#### runtime-version

Get runtime version from a Substrate node.

**Usage:**

```bash
metadata-tools runtime-version <RPC_URL> [--field FIELD]
```

- `<RPC_URL>`: RPC URL of the Substrate node.
- `--field FIELD`: (Optional) Specific field to display (`spec_name`, `spec_version`, `impl_version`, `transaction_version`, `state_version`).

#### check

Check compatibility between node metadata and stored metadata.

**Usage:**

```bash
metadata-tools check <METADATA_FOLDER> [RPC_URL]
```

- `<METADATA_FOLDER>`: Folder containing stored metadata files.
- `[RPC_URL]`: (Optional) RPC URL of the Substrate node (default: `ws://localhost:9944`).
