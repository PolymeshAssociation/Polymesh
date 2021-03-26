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
		const api = await init.createApi();
		const ticker = await init.generateRandomTicker();
		const portfolioName = await init.generateRandomTicker();
		const testEntities = await init.initMain(api.api);
		const alice = testEntities[0];
		const primaryDevSeed = await init.generateRandomKey();
		const secondaryDevSeed = await init.generateRandomKey();
		const primaryKeys = await init.generateKeys(api.api, 1, primaryDevSeed);
		const secondaryKeys = await init.generateKeys(api.api, 1, secondaryDevSeed);
		const issuerDids = await createIdentities(api.api, primaryKeys, alice);
		let extrinsics: LegacyPalletPermissions[] = [];
		let portfolios: PortfolioId[] = [];
		let assets: Ticker[] = [];
		let documents: Document[] = [];

		await distributePolyBatch(api.api, [primaryKeys[0]], init.transferAmount, alice);
		await issueTokenToDid(api.api, primaryKeys[0], ticker, 1000000, null);
		await addSecondaryKeys(api.api, secondaryKeys, primaryKeys);
		await authorizeJoinToIdentities(api.api, primaryKeys, issuerDids, secondaryKeys);
		await distributePolyBatch(api.api, [secondaryKeys[0]], init.transferAmount, alice);

		let portfolioOutput = await createPortfolio(api.api, portfolioName, secondaryKeys[0]);
		assert.equal(portfolioOutput, false);

		setExtrinsic(extrinsics, "Portfolio", "create_portfolio");
		await setPermissionToSigner(api.api, primaryKeys, secondaryKeys, extrinsics, portfolios, assets);
		portfolioOutput = await createPortfolio(api.api, portfolioName, secondaryKeys[0]);
		assert.equal(portfolioOutput, true);

		setExtrinsic(extrinsics, "Portfolio", "move_portfolio_funds");
		await setPermissionToSigner(api.api, primaryKeys, secondaryKeys, extrinsics, portfolios, assets);
		let portfolioFundsOutput = await movePortfolioFunds(api.api, primaryKeys[0], secondaryKeys[0], ticker, 100);
		assert.equal(portfolioFundsOutput, false);

		await setPortfolio(api.api, portfolios, primaryKeys[0], "Default");
		await setPortfolio(api.api, portfolios, secondaryKeys[0], "User");
		await setPermissionToSigner(api.api, primaryKeys, secondaryKeys, extrinsics, portfolios, assets);
		portfolioFundsOutput = await movePortfolioFunds(api.api, primaryKeys[0], secondaryKeys[0], ticker, 100);
		assert.equal(portfolioFundsOutput, true);

		setExtrinsic(extrinsics, "Asset", "add_documents");
		await setPermissionToSigner(api.api, primaryKeys, secondaryKeys, extrinsics, portfolios, assets);
		setDoc(documents, "www.google.com", {None: ""}, "google");
		let addDocsOutput = await addDocuments(api.api, ticker, documents, secondaryKeys[0]);
		assert.equal(addDocsOutput, false);

		setAsset(ticker, assets);
		await setPermissionToSigner(api.api, primaryKeys, secondaryKeys, extrinsics, portfolios, assets);
		addDocsOutput = await addDocuments(api.api, ticker, documents, secondaryKeys[0]);
		assert.equal(addDocsOutput, true);
}

main().catch((err) => console.log(`Error: ${err.message}`)).finally(() => process.exit());