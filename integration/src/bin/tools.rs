use anyhow::{anyhow, Result};
use integration::PolymeshTester;
use sp_weights::Weight;
use std::env;
use std::fs;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Err(anyhow!("Usage: {} <command> [args...]", args[0]));
    }

    match args[1].as_str() {
        "upgrade_chain" => {
            if args.len() != 3 {
                return Err(anyhow!("Usage: {} upgrade_chain <wasm_file>", args[0]));
            }
            upgrade_chain(&args[2]).await
        }
        #[cfg(feature = "current_release")]
        "curve_tree_leaf_path" => {
            if args.len() < 4 || args.len() > 5 {
                return Err(anyhow!(
                    "Usage: {} curve_tree_leaf_path <tree: asset|account|fee> <leaf_index> [<block_number>]",
                    args[0]
                ));
            }
            let tree = &args[2];
            let leaf_index = args[3].parse()?;

            let block_number = if args.len() >= 5 {
                Some(args[4].parse()?)
            } else {
                None
            };

            match tree.as_str() {
                "asset" => asset_leaf_path(leaf_index, block_number).await,
                "account" => account_leaf_path(leaf_index, block_number).await,
                "fee" => fee_account_leaf_path(leaf_index, block_number).await,
                _ => Err(anyhow!("Unknown tree type: {}", tree)),
            }
        }
        _ => Err(anyhow!("Unknown command: {}", args[1])),
    }
}

async fn upgrade_chain(wasm_path: &str) -> Result<()> {
    let tester = PolymeshTester::new().await?;
    let mut sudo = tester.sudo.clone().expect("No Sudo user");

    // Read the WASM file
    let code = fs::read(wasm_path)?;

    // Set code call
    let set_code = tester.api.call().system().set_code(code.clone())?;

    // Create and submit the upgrade transaction
    let mut res = tester
        .api
        .call()
        .sudo()
        .sudo_unchecked_weight(set_code.into(), Weight::from_parts(1_000_000_000, 0))?
        .submit_and_watch(&mut sudo)
        .await?;

    // Wait for finalization
    res.wait_finalized().await?;
    println!("Chain upgrade completed successfully");

    Ok(())
}

#[cfg(feature = "current_release")]
async fn asset_leaf_path(leaf_index: u64, block_number: Option<u32>) -> Result<()> {
    use integration::confidential_assets_helper::*;

    // Initialize tester to get client API.
    let tester = PolymeshTester::new().await?;

    let tree = AssetCurveTree::new(&tester.api).await?;

    let leaf = tree
        .get_leaf(leaf_index, block_number)
        .await?
        .ok_or_else(|| anyhow!("Leaf index {} not found", leaf_index))?;
    println!("Leaf at index {}: {:?}", leaf_index, leaf);
    let path = tree.get_path_to_leaf(leaf_index, 0, block_number).await?;
    println!("Path to leaf index {}:", leaf_index);
    print_curve_tree_path(
        &path,
        &format!("leaf index {} at block: {:?}", leaf_index, block_number),
    );
    let root = tree.fetch_root(block_number).await?;
    println!("Root at block {:?}: {:?}", block_number, root);

    Ok(())
}

#[cfg(feature = "current_release")]
async fn account_leaf_path(leaf_index: u64, block_number: Option<u32>) -> Result<()> {
    use integration::confidential_assets_helper::*;

    // Initialize tester to get client API.
    let tester = PolymeshTester::new().await?;

    let tree = AccountCurveTree::new(&tester.api).await?;

    let leaf = tree
        .get_leaf(leaf_index, block_number)
        .await?
        .ok_or_else(|| anyhow!("Leaf index {} not found", leaf_index))?;
    println!("Leaf at index {}: {:?}", leaf_index, leaf);
    let path = tree.get_path_to_leaf(leaf_index, 0, block_number).await?;
    println!("Path to leaf index {}:", leaf_index);
    print_curve_tree_path(
        &path,
        &format!("leaf index {} at block: {:?}", leaf_index, block_number),
    );
    let root = tree.fetch_root(block_number).await?;
    println!("Root at block {:?}: {:?}", block_number, root);

    Ok(())
}

#[cfg(feature = "current_release")]
async fn fee_account_leaf_path(leaf_index: u64, block_number: Option<u32>) -> Result<()> {
    use integration::confidential_assets_helper::*;

    // Initialize tester to get client API.
    let tester = PolymeshTester::new().await?;

    let tree = FeeAccountCurveTree::new(&tester.api).await?;

    let leaf = tree
        .get_leaf(leaf_index, block_number)
        .await?
        .ok_or_else(|| anyhow!("Leaf index {} not found", leaf_index))?;
    println!("Leaf at index {}: {:?}", leaf_index, leaf);
    let path = tree.get_path_to_leaf(leaf_index, 0, block_number).await?;
    println!("Path to leaf index {}:", leaf_index);
    print_curve_tree_path(
        &path,
        &format!("Fee leaf index {} at block: {:?}", leaf_index, block_number),
    );
    let root = tree.fetch_root(block_number).await?;
    println!("Root at block {:?}: {:?}", block_number, root);

    Ok(())
}
