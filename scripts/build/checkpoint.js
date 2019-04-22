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
        var wsProvider, api, tickerUpper, tickerInfo, _a, _b, cpTotalSupply, input, userBalance, maxCheckpoint, _c, fetchingOldBalance, bal, bal, _d;
        return __generator(this, function (_e) {
            switch (_e.label) {
                case 0:
                    wsProvider = new api_1.WsProvider('ws://127.0.0.1:9944');
                    return [4 /*yield*/, api_1.ApiPromise.create({
                            types: customTypes,
                            provider: wsProvider
                        })];
                case 1:
                    api = _e.sent();
                    tickerUpper = answers.ticker.toUpperCase();
                    _b = (_a = JSON).parse;
                    return [4 /*yield*/, api.query.asset.tokens(tickerUpper)];
                case 2:
                    tickerInfo = _b.apply(_a, [(_e.sent()).toString()]);
                    return [4 /*yield*/, api.query.asset.checkpointTotalSupply([tickerUpper, answers.checkpoint])];
                case 3:
                    cpTotalSupply = _e.sent();
                    console.log("Total supply of " + new Buffer(tickerInfo.name).toString() + " token at checkpoint " + answers.checkpoint + " was " + cpTotalSupply);
                    return [4 /*yield*/, inquirer_1.default.prompt(userAddress)];
                case 4:
                    input = _e.sent();
                    if (!(input.address != '0')) return [3 /*break*/, 18];
                    userBalance = void 0;
                    _c = bn_js_1.default.bind;
                    return [4 /*yield*/, api.query.asset.totalCheckpoints(tickerUpper)];
                case 5:
                    maxCheckpoint = new (_c.apply(bn_js_1.default, [void 0, _e.sent()]))().toNumber();
                    fetchingOldBalance = false;
                    if (!(answers.checkpoint < maxCheckpoint)) return [3 /*break*/, 14];
                    _e.label = 6;
                case 6:
                    if (!(maxCheckpoint >= answers.checkpoint)) return [3 /*break*/, 8];
                    return [4 /*yield*/, api.query.asset.checkpointBalance([tickerUpper, input.address, maxCheckpoint])];
                case 7:
                    bal = _e.sent();
                    if (bal.isSome) {
                        fetchingOldBalance = true;
                        return [3 /*break*/, 8];
                    }
                    maxCheckpoint--;
                    return [3 /*break*/, 6];
                case 8:
                    if (!fetchingOldBalance) return [3 /*break*/, 12];
                    maxCheckpoint = answers.checkpoint;
                    _e.label = 9;
                case 9:
                    if (!(maxCheckpoint > 0)) return [3 /*break*/, 11];
                    console.log(maxCheckpoint);
                    return [4 /*yield*/, api.query.asset.checkpointBalance([tickerUpper, input.address, maxCheckpoint])];
                case 10:
                    bal = _e.sent();
                    if (bal.isSome) {
                        userBalance = bal.value;
                        return [3 /*break*/, 11];
                    }
                    maxCheckpoint--;
                    return [3 /*break*/, 9];
                case 11: return [3 /*break*/, 13];
                case 12:
                    maxCheckpoint = 0;
                    _e.label = 13;
                case 13: return [3 /*break*/, 15];
                case 14:
                    maxCheckpoint = 0;
                    _e.label = 15;
                case 15:
                    if (!(maxCheckpoint == 0)) return [3 /*break*/, 17];
                    _d = bn_js_1.default.bind;
                    return [4 /*yield*/, api.query.asset.balanceOf([tickerUpper, input.address])];
                case 16:
                    userBalance = new (_d.apply(bn_js_1.default, [void 0, _e.sent()]))().toNumber();
                    _e.label = 17;
                case 17:
                    console.log("Balance of " + input.address + " at checkpoint " + answers.checkpoint + " was " + userBalance);
                    _e.label = 18;
                case 18:
                    process.exit();
                    return [2 /*return*/];
            }
        });
    });
}
inquirer_1.default.prompt(initialQuestions).then(function (answers) { return main(answers).catch(console.error); });
