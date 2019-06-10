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
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (Object.hasOwnProperty.call(mod, k)) result[k] = mod[k];
    result["default"] = mod;
    return result;
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
// @ts-check
// Required imports
var api_1 = require("@polkadot/api");
var fs = __importStar(require("fs"));
var path = __importStar(require("path"));
var inquirer_1 = __importDefault(require("inquirer"));
var Utils = __importStar(require("web3-utils"));
var filePath = path.join(__dirname + "../../../polymesh_substrate/substrateui_dev.json");
var customTypes = JSON.parse(fs.readFileSync(filePath, "utf8").toString());
console.log("Welcome to a notification tracking device:\n");
var openingQuestions = [
    {
        type: 'list',
        name: 'operation',
        message: "Which operation do you want to perform: ",
        choices: [
            "Subscribe an event",
            "Get events of module"
        ],
        filter: function (val) {
            if (val == "Subscribe an event")
                return 0;
            else
                return 1;
        }
    }
];
var questionsOp1 = [
    {
        type: 'input',
        name: 'module',
        message: "What is the module name whose events you want to fetch: ",
        default: "asset",
        filter: function (val) {
            return val.toLowerCase();
        }
    },
    {
        type: 'number',
        name: 'from',
        message: "Enter the from block: ",
        default: 1,
        validate: function (val) {
            if (val <= 0)
                return "Please enter the valid block no.";
            else
                return true;
        }
    },
    {
        type: 'number',
        name: 'to',
        message: "Enter the to block: ",
        default: function () {
            return __awaiter(this, void 0, void 0, function () {
                var wsProvider, api, _a;
                return __generator(this, function (_b) {
                    switch (_b.label) {
                        case 0:
                            wsProvider = new api_1.WsProvider('ws://127.0.0.1:9944');
                            return [4 /*yield*/, api_1.ApiPromise.create({
                                    provider: wsProvider
                                })];
                        case 1:
                            api = _b.sent();
                            _a = parseInt;
                            return [4 /*yield*/, api.derive.chain.bestNumber()];
                        case 2: return [2 /*return*/, _a.apply(void 0, [(_b.sent()).toString()])];
                    }
                });
            });
        },
        validate: function (val) {
            if (val <= 0)
                return "Please enter the valid block no.";
            else
                return true;
        }
    }
];
var getEventAllowed = [
    {
        type: 'confirm',
        name: "isAllowed",
        message: "Do you want to filter the events by event name? ",
        default: false,
    }
];
var getEventName = [
    {
        type: 'input',
        name: "eventName",
        message: "Enter the event name: ",
        default: "Transfer",
    }
];
var questionsOp2 = [
    {
        type: 'input',
        name: 'module',
        message: "What is the module name whose events you want to fetch: ",
        default: "asset",
        filter: function (val) {
            return val.toLowerCase();
        }
    }
];
function moduleEvents(api) {
    return __awaiter(this, void 0, void 0, function () {
        var event_name, answers, filterByEvent, currentBlock, diff, k, blockNo, hash, events, i, typeList, j, value;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0: return [4 /*yield*/, inquirer_1.default.prompt(questionsOp1)];
                case 1:
                    answers = _a.sent();
                    return [4 /*yield*/, inquirer_1.default.prompt(getEventAllowed)];
                case 2:
                    filterByEvent = _a.sent();
                    if (!filterByEvent.isAllowed) return [3 /*break*/, 4];
                    return [4 /*yield*/, inquirer_1.default.prompt(getEventName)];
                case 3:
                    event_name = (_a.sent()).eventName;
                    _a.label = 4;
                case 4: return [4 /*yield*/, api.derive.chain.bestNumber()];
                case 5:
                    currentBlock = _a.sent();
                    if (!(answers.to >= answers.from && parseInt(currentBlock.toString()) >= answers.to)) return [3 /*break*/, 11];
                    diff = parseInt(answers.to) - parseInt(answers.from);
                    k = 0;
                    _a.label = 6;
                case 6:
                    if (!(k <= diff)) return [3 /*break*/, 10];
                    blockNo = answers.from + k;
                    return [4 /*yield*/, api.rpc.chain.getBlockHash(blockNo)];
                case 7:
                    hash = _a.sent();
                    console.log(hash.toString());
                    return [4 /*yield*/, api.query.system.events.at(hash.toString())];
                case 8:
                    events = _a.sent();
                    for (i = 0; i < Object.keys(events).length - 1; i++) {
                        console.log(blockNo);
                        if (events[i].event.data["_section"] == answers.module) {
                            typeList = events[i].event.data["_typeDef"];
                            if (events[i].event.data["_method"] != event_name && filterByEvent.isAllowed)
                                continue;
                            console.log("EventName - " + events[i].event.data["_method"] + " at block number " + blockNo);
                            for (j = 0; j < typeList.length; j++) {
                                value = events[i].event.data[j];
                                if (typeList[j].type == "Bytes")
                                    value = Utils.hexToString(Utils.bytesToHex(events[i].event.data[j]));
                                console.log(typeList[j].type + " : " + value);
                            }
                            console.log("***************************************");
                        }
                    }
                    _a.label = 9;
                case 9:
                    k++;
                    return [3 /*break*/, 6];
                case 10: return [3 /*break*/, 12];
                case 11: throw new Error("Invalid block numbers");
                case 12:
                    process.exit(0);
                    return [2 /*return*/];
            }
        });
    });
}
function subscribeEvents(api) {
    return __awaiter(this, void 0, void 0, function () {
        var answers, unsubscribe;
        var _this = this;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0: return [4 /*yield*/, inquirer_1.default.prompt(questionsOp2)];
                case 1:
                    answers = _a.sent();
                    return [4 /*yield*/, api.rpc.chain.subscribeNewHead(function (header) { return __awaiter(_this, void 0, void 0, function () {
                            var hash, events, i, typeList, j, value;
                            return __generator(this, function (_a) {
                                switch (_a.label) {
                                    case 0:
                                        console.log("Chain is at block: #" + header.blockNumber);
                                        return [4 /*yield*/, api.rpc.chain.getBlockHash(header.blockNumber)];
                                    case 1:
                                        hash = _a.sent();
                                        return [4 /*yield*/, api.query.system.events.at(hash.toString())];
                                    case 2:
                                        events = _a.sent();
                                        for (i = 0; i < Object.keys(events).length - 1; i++) {
                                            if (events[i].event.data["_section"] == answers.module) {
                                                typeList = events[i].event.data["_typeDef"];
                                                console.log("EventName - " + events[i].event.data["_method"] + " at block number " + header.blockNumber);
                                                for (j = 0; j < typeList.length; j++) {
                                                    value = events[i].event.data[j];
                                                    if (typeList[j].type == "Bytes")
                                                        value = Utils.hexToString(Utils.bytesToHex(events[i].event.data[j]));
                                                    console.log(typeList[j].type + " : " + value);
                                                }
                                                console.log("***************************************");
                                            }
                                        }
                                        return [2 /*return*/];
                                }
                            });
                        }); })];
                case 2:
                    unsubscribe = _a.sent();
                    return [2 /*return*/];
            }
        });
    });
}
function main() {
    return __awaiter(this, void 0, void 0, function () {
        var wsProvider, api, _a, chain, nodeName, nodeVersion;
        return __generator(this, function (_b) {
            switch (_b.label) {
                case 0:
                    wsProvider = new api_1.WsProvider('ws://127.0.0.1:9944');
                    return [4 /*yield*/, api_1.ApiPromise.create({
                            types: customTypes,
                            provider: wsProvider
                        })];
                case 1:
                    api = _b.sent();
                    return [4 /*yield*/, Promise.all([
                            api.rpc.system.chain(),
                            api.rpc.system.name(),
                            api.rpc.system.version()
                        ])];
                case 2:
                    _a = _b.sent(), chain = _a[0], nodeName = _a[1], nodeVersion = _a[2];
                    console.log("You are connected to chain " + chain + " using " + nodeName + " v" + nodeVersion);
                    inquirer_1.default.prompt(openingQuestions).then(function (answers) {
                        if (answers.operation == 1) {
                            moduleEvents(api);
                        }
                        else {
                            subscribeEvents(api);
                        }
                    });
                    return [2 /*return*/];
            }
        });
    });
}
main().catch(function (error) {
    console.error(error);
    process.exit(-1);
});
