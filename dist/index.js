"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.supportsSymbolExtraction = exports.extractSymbols = exports.removeEntry = exports.addEntry = exports.writeGeneratedFile = exports.readEntries = exports.readGeneratedFile = exports.findGeneratedFile = exports.entryKey = exports.formatEntry = exports.parseGeneratedFile = exports.parseEntry = void 0;
var entry_1 = require("./entry");
Object.defineProperty(exports, "parseEntry", { enumerable: true, get: function () { return entry_1.parseEntry; } });
Object.defineProperty(exports, "parseGeneratedFile", { enumerable: true, get: function () { return entry_1.parseGeneratedFile; } });
Object.defineProperty(exports, "formatEntry", { enumerable: true, get: function () { return entry_1.formatEntry; } });
Object.defineProperty(exports, "entryKey", { enumerable: true, get: function () { return entry_1.entryKey; } });
var store_1 = require("./store");
Object.defineProperty(exports, "findGeneratedFile", { enumerable: true, get: function () { return store_1.findGeneratedFile; } });
Object.defineProperty(exports, "readGeneratedFile", { enumerable: true, get: function () { return store_1.readGeneratedFile; } });
Object.defineProperty(exports, "readEntries", { enumerable: true, get: function () { return store_1.readEntries; } });
Object.defineProperty(exports, "writeGeneratedFile", { enumerable: true, get: function () { return store_1.writeGeneratedFile; } });
Object.defineProperty(exports, "addEntry", { enumerable: true, get: function () { return store_1.addEntry; } });
Object.defineProperty(exports, "removeEntry", { enumerable: true, get: function () { return store_1.removeEntry; } });
var symbols_1 = require("./symbols");
Object.defineProperty(exports, "extractSymbols", { enumerable: true, get: function () { return symbols_1.extractSymbols; } });
Object.defineProperty(exports, "supportsSymbolExtraction", { enumerable: true, get: function () { return symbols_1.supportsSymbolExtraction; } });
//# sourceMappingURL=index.js.map