"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.findGeneratedFile = findGeneratedFile;
exports.readGeneratedFile = readGeneratedFile;
exports.readEntries = readEntries;
exports.writeGeneratedFile = writeGeneratedFile;
exports.addEntry = addEntry;
exports.removeEntry = removeEntry;
const fs = __importStar(require("fs"));
const path = __importStar(require("path"));
const entry_1 = require("./entry");
const GENERATED_FILENAME = "GENERATED";
function findGeneratedFile(cwd = process.cwd()) {
    return path.join(cwd, GENERATED_FILENAME);
}
function readGeneratedFile(filePath) {
    if (!fs.existsSync(filePath))
        return "";
    return fs.readFileSync(filePath, "utf8");
}
function readEntries(filePath) {
    return (0, entry_1.parseGeneratedFile)(readGeneratedFile(filePath));
}
function writeGeneratedFile(filePath, lines) {
    const content = lines.join("\n");
    fs.writeFileSync(filePath, content ? content + "\n" : "", "utf8");
}
function addEntry(filePath, raw) {
    const entry = (0, entry_1.parseEntry)(raw);
    if (!entry)
        throw new Error(`Invalid entry: ${raw}`);
    const content = readGeneratedFile(filePath);
    const lines = content ? content.split("\n").filter((l) => l.trim() !== "") : [];
    if (lines.includes(raw.trim())) {
        return;
    }
    lines.push(raw.trim());
    writeGeneratedFile(filePath, lines);
}
function removeEntry(filePath, raw) {
    const content = readGeneratedFile(filePath);
    if (!content)
        return false;
    const lines = content.split("\n").filter((l) => l.trim() !== "");
    const target = raw.trim();
    const idx = lines.indexOf(target);
    if (idx === -1)
        return false;
    lines.splice(idx, 1);
    writeGeneratedFile(filePath, lines);
    return true;
}
//# sourceMappingURL=store.js.map