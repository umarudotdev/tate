"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.parseEntry = parseEntry;
exports.parseGeneratedFile = parseGeneratedFile;
exports.formatEntry = formatEntry;
exports.entryKey = entryKey;
const RANGE_RE = /^(.+):(\d+)-(\d+)$/;
const SYMBOL_RE = /^(.+)::(.+)$/;
function parseEntry(line) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("#"))
        return null;
    const symbolMatch = SYMBOL_RE.exec(trimmed);
    if (symbolMatch) {
        return { kind: "symbol", path: symbolMatch[1], symbol: symbolMatch[2], raw: trimmed };
    }
    const rangeMatch = RANGE_RE.exec(trimmed);
    if (rangeMatch) {
        const start = parseInt(rangeMatch[2], 10);
        const end = parseInt(rangeMatch[3], 10);
        return { kind: "range", path: rangeMatch[1], start, end, raw: trimmed };
    }
    return { kind: "file", path: trimmed, raw: trimmed };
}
function parseGeneratedFile(content) {
    return content
        .split("\n")
        .map(parseEntry)
        .filter((e) => e !== null);
}
function formatEntry(entry) {
    return entry.raw;
}
function entryKey(entry) {
    return entry.raw;
}
//# sourceMappingURL=entry.js.map