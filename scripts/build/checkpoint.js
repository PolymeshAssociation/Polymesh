"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : new P(function (resolve) { resolve(result.value); }).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g;
    return g = { next: verb(0), "throw": verb(1), "return": verb(2) }, typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (_) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
// Required imports
// @ts-check
var api_1 = require("@polkadot/api");
var inquirer_1 = __importDefault(require("inquirer"));
var bn_js_1 = __importDefault(require("bn.js"));
var Alice = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
var customTypes = {
    "TokenBalance": "u128",
    "SecurityToken": {
        "name": "Vec<u8>",
        "total_supply": "u128",
        "owner": "AccountId"
    },
    "Restriction": {
        "name": "Vec<u8>",
        "token_id": "u32",
        "can_transfer": "bool"
    },
    "Whitelist": {
        "investor": "AccountId",
        "canSendAfter": "Moment",
        "canReceiveAfter": "Moment"
    },
    "Issuer": {
        "account": "AccountId",
        "access_level": "u16",
        "active": "bool"
    },
    "Investor": {
        "account": "AccountId",
        "access_level": "u16",
        "active": "bool",
        "jurisdiction": "u16"
    },
    "FeeOf": "Balance"
};
var initialQuestions = [
    {
        type: 'input',
        name: 'ticker',
        message: "Enter the token ticker: ",
        default: 'HW'
    },
    {
        type: 'input',
        name: 'checkpoint',
        message: "Enter the checkpoint number: ",
        default: 1
    }
];
var userAddress = [
    {
        type: 'input',
        name: 'address',
        message: "Enter user to address to query their checkpoint balance or 0 to exit: ",
        default: Alice
    }
];
function main(answers) {
    return __awaiter(this, void 0, void 0, function () {
        var wsProvider, api, tickerUpper, maxCheckpoint, _a, tickerInfo, _b, _c, cpTotalSupply, input, userBalance, latestUserCheckpoint, _d, bal, _e;
        return __generator(this, function (_f) {
            switch (_f.label) {
                case 0:
                    wsProvider = new api_1.WsProvider('ws://127.0.0.1:9944');
                    return [4 /*yield*/, api_1.ApiPromise.create({
                            types: customTypes,
                            provider: wsProvider
                        })];
                case 1:
                    api = _f.sent();
                    tickerUpper = answers.ticker.toUpperCase();
                    _a = bn_js_1.default.bind;
                    return [4 /*yield*/, api.query.asset.totalCheckpoints(tickerUpper)];
                case 2:
                    maxCheckpoint = new (_a.apply(bn_js_1.default, [void 0, _f.sent()]))().toNumber();
                    if (answers.checkpoint > maxCheckpoint || answers.checkpoint <= 0) {
                        throw new RangeError("Checkpoint does not exist");
                    }
                    _c = (_b = JSON).parse;
                    return [4 /*yield*/, api.query.asset.tokens(tickerUpper)];
                case 3:
                    tickerInfo = _c.apply(_b, [(_f.sent()).toString()]);
                    return [4 /*yield*/, api.query.asset.checkpointTotalSupply([tickerUpper, answers.checkpoint])];
                case 4:
                    cpTotalSupply = _f.sent();
                    console.log("Total supply of " + Buffer.from(tickerInfo.name).toString() + " token at checkpoint " + answers.checkpoint + " was " + cpTotalSupply);
                    return [4 /*yield*/, inquirer_1.default.prompt(userAddress)];
                case 5:
                    input = _f.sent();
                    if (!(input.address != '0')) return [3 /*break*/, 13];
                    userBalance = void 0;
                    _d = bn_js_1.default.bind;
                    return [4 /*yield*/, api.query.asset.latestUserCheckpoint([tickerUpper, input.address])];
                case 6:
                    latestUserCheckpoint = new (_d.apply(bn_js_1.default, [void 0, _f.sent()]))().toNumber();
                    if (!(answers.checkpoint <= latestUserCheckpoint)) return [3 /*break*/, 10];
                    maxCheckpoint = answers.checkpoint;
                    _f.label = 7;
                case 7:
                    if (!(maxCheckpoint > 0)) return [3 /*break*/, 9];
                    return [4 /*yield*/, api.query.asset.checkpointBalance([tickerUpper, input.address, maxCheckpoint])];
                case 8:
                    bal = _f.sent();
                    if (bal.isSome) {
                        userBalance = bal.value;
                        return [3 /*break*/, 9];
                    }
                    maxCheckpoint--;
                    return [3 /*break*/, 7];
                case 9: return [3 /*break*/, 12];
                case 10:
                    _e = bn_js_1.default.bind;
                    return [4 /*yield*/, api.query.asset.balanceOf([tickerUpper, input.address])];
                case 11:
                    // No checkpoint balance stored for user. We should return latest balance.
                    userBalance = new (_e.apply(bn_js_1.default, [void 0, _f.sent()]))().toNumber();
                    _f.label = 12;
                case 12:
                    console.log("Balance of " + input.address + " at checkpoint " + answers.checkpoint + " was " + userBalance);
                    _f.label = 13;
                case 13:
                    process.exit();
                    return [2 /*return*/];
            }
        });
    });
}
inquirer_1.default.prompt(initialQuestions).then(function (answers) { return main(answers).catch(function (err) {
    console.error(err);
    process.exit();
}); });
