// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use jsonrpsee::http_client::HttpClientBuilder;
use pallet_revive::evm::{Account, ReceiptInfo};
use pallet_revive_eth_rpc::example::TransactionBuilder;
use sp_core::H160;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let tx_payload: Vec<u8> = vec![
		0x00, // System pallet index.
		0x07, // Remark With Event call index.
		0x10, // Compact-encoded length of the remark (16).
		0x54, 0x45, 0x53, 0x54, // Remark 'T', 'E', 'S', 'T'
	];

	let client = Arc::new(HttpClientBuilder::default().build("http://localhost:8545")?);

	let alith = Account::default();
	// Revive pallet address.
	let dest = H160::from_slice(&hex::decode("6d6f646c70792f70616464720000000000000000")?);

	println!("\n\n=== Eth calling System.Remark  ===\n\n");

	let tx = TransactionBuilder::new(client)
		.signer(alith)
		.input(tx_payload)
		.to(dest)
		.send()
		.await?;
	println!("Transaction hash: {:?}", tx.hash());

	let ReceiptInfo { block_number, gas_used, status, .. } = tx.wait_for_receipt().await?;
	println!("Receipt: ");
	println!("- Block number: {block_number}");
	println!("- Gas used: {gas_used}");
	println!("- Success: {status:?}");

	Ok(())
}
