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
		const issuerDids = await createIdentities(primaryKeys, alice);
		let extrinsics: LegacyPalletPermissions[] = [];
		let portfolios: PortfolioId[] = [];
		let assets: Ticker[] = [];
		let documents: Document[] = [];

		await distributePolyBatch([primaryKeys[0]], init.transferAmount, alice);
		await issueTokenToDid(primaryKeys[0], ticker, 1000000, null);
		await addSecondaryKeys(secondaryKeys, primaryKeys);
		await authorizeJoinToIdentities(primaryKeys, issuerDids, secondaryKeys);
		await distributePolyBatch([secondaryKeys[0]], init.transferAmount, alice);

		let portfolioOutput = await createPortfolio(portfolioName, secondaryKeys[0]);
		assert.equal(portfolioOutput, false);

		setExtrinsic(extrinsics, "Portfolio", "create_portfolio");
		await setPermissionToSigner(primaryKeys, secondaryKeys, extrinsics, portfolios, assets);
		portfolioOutput = await createPortfolio(portfolioName, secondaryKeys[0]);
		assert.equal(portfolioOutput, true);

		setExtrinsic(extrinsics, "Portfolio", "move_portfolio_funds");
		await setPermissionToSigner(primaryKeys, secondaryKeys, extrinsics, portfolios, assets);
		let portfolioFundsOutput = await movePortfolioFunds(primaryKeys[0], secondaryKeys[0], ticker, 100);
		assert.equal(portfolioFundsOutput, false);

		await setPortfolio(portfolios, primaryKeys[0], "Default");
		await setPortfolio(portfolios, secondaryKeys[0], "User");
		await setPermissionToSigner(primaryKeys, secondaryKeys, extrinsics, portfolios, assets);
		portfolioFundsOutput = await movePortfolioFunds(primaryKeys[0], secondaryKeys[0], ticker, 100);
		assert.equal(portfolioFundsOutput, true);

		setExtrinsic(extrinsics, "Asset", "add_documents");
		await setPermissionToSigner(primaryKeys, secondaryKeys, extrinsics, portfolios, assets);
		setDoc(documents, "www.google.com", {None: ""}, "google");
		let addDocsOutput = await addDocuments(ticker, documents, secondaryKeys[0]);
		assert.equal(addDocsOutput, false);

		setAsset(ticker, assets);
		await setPermissionToSigner(primaryKeys, secondaryKeys, extrinsics, portfolios, assets);
		addDocsOutput = await addDocuments(ticker, documents, secondaryKeys[0]);
		assert.equal(addDocsOutput, true);
}

main().catch((err) => console.log(`Error: ${err.message}`)).finally(() => process.exit());