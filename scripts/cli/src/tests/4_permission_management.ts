import * as init from "../util/init";
import { createIdentities, authorizeJoinToIdentities, setPermissionToSigner } from "../helpers/identity_helper";
import { distributePolyBatch } from "../helpers/poly_helper";
import { addSecondaryKeys } from "../helpers/key_management_helper";
import { setAsset, setExtrinsic, setPortfolio, setDoc } from "../helpers/permission_helper";
import { createPortfolio, movePortfolioFunds } from "../helpers/portfolio_helper";
import { addDocuments, issueTokenToDid } from "../helpers/asset_helper";
import { Document, LegacyPalletPermissions, PortfolioId, Ticker } from "../types";
import { assert } from "chai";

async function main(): Promise<void> {
	const ticker = init.generateRandomTicker();
	const portfolioName = init.generateRandomTicker();
	const testEntities = await init.initMain();
	const alice = testEntities[0];
	const primaryDevSeed = init.generateRandomKey();
	const secondaryDevSeed = init.generateRandomKey();
	const primaryKeys = await init.generateKeys(1, primaryDevSeed);
	const secondaryKeys = await init.generateKeys(1, secondaryDevSeed);
	await createIdentities(alice, primaryKeys);
	let extrinsics: LegacyPalletPermissions[] = [];
	let portfolios: PortfolioId[] = [];
	let assets: Ticker[] = [];
	let documents: Document[] = [];

	await distributePolyBatch(alice, [primaryKeys[0]], init.transferAmount);
	await issueTokenToDid(primaryKeys[0], ticker, 1000000, null);
	await addSecondaryKeys(primaryKeys, secondaryKeys);
	await authorizeJoinToIdentities(secondaryKeys, primaryKeys);
	await distributePolyBatch(alice, [secondaryKeys[0]], init.transferAmount);

	let portfolioOutput = await createPortfolio(secondaryKeys[0], portfolioName);
	assert.equal(portfolioOutput, false);

	setExtrinsic(extrinsics, "Portfolio", "create_portfolio");
	await setPermissionToSigner(primaryKeys, secondaryKeys, extrinsics, portfolios, assets);
	portfolioOutput = await createPortfolio(secondaryKeys[0], portfolioName);
	assert.equal(portfolioOutput, true);

	setExtrinsic(extrinsics, "Portfolio", "move_portfolio_funds");
	await setPermissionToSigner(primaryKeys, secondaryKeys, extrinsics, portfolios, assets);
	let portfolioFundsOutput = await movePortfolioFunds(secondaryKeys[0], primaryKeys[0], ticker, 100);
	assert.equal(portfolioFundsOutput, false);

	await setPortfolio(portfolios, primaryKeys[0], "Default");
	await setPortfolio(portfolios, secondaryKeys[0], "User");
	await setPermissionToSigner(primaryKeys, secondaryKeys, extrinsics, portfolios, assets);
	portfolioFundsOutput = await movePortfolioFunds(secondaryKeys[0], primaryKeys[0], ticker, 100);
	assert.equal(portfolioFundsOutput, true);

	setExtrinsic(extrinsics, "Asset", "add_documents");
	await setPermissionToSigner(primaryKeys, secondaryKeys, extrinsics, portfolios, assets);
	setDoc(documents, "www.google.com", { None: "" }, "google");
	let addDocsOutput = await addDocuments(secondaryKeys[0], ticker, documents);
	assert.equal(addDocsOutput, false);

	setAsset(ticker, assets);
	await setPermissionToSigner(primaryKeys, secondaryKeys, extrinsics, portfolios, assets);
	addDocsOutput = await addDocuments(secondaryKeys[0], ticker, documents);
	assert.equal(addDocsOutput, true);
}

main()
	.catch((err) => console.log(`Error: ${err.message}`))
	.finally(() => process.exit());
