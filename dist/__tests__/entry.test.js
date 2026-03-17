"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const vitest_1 = require("vitest");
const entry_1 = require("../entry");
(0, vitest_1.describe)("parseEntry", () => {
    (0, vitest_1.it)("returns null for empty lines", () => {
        (0, vitest_1.expect)((0, entry_1.parseEntry)("")).toBeNull();
        (0, vitest_1.expect)((0, entry_1.parseEntry)("   ")).toBeNull();
    });
    (0, vitest_1.it)("returns null for comment lines", () => {
        (0, vitest_1.expect)((0, entry_1.parseEntry)("# This is a comment")).toBeNull();
        (0, vitest_1.expect)((0, entry_1.parseEntry)("  # comment")).toBeNull();
    });
    (0, vitest_1.it)("parses a whole-file entry", () => {
        const entry = (0, entry_1.parseEntry)("src/db/migrations/001.sql");
        (0, vitest_1.expect)(entry).toEqual({ kind: "file", path: "src/db/migrations/001.sql", raw: "src/db/migrations/001.sql" });
    });
    (0, vitest_1.it)("parses a line-range entry", () => {
        const entry = (0, entry_1.parseEntry)("src/auth/login.ts:5-16");
        (0, vitest_1.expect)(entry).toEqual({ kind: "range", path: "src/auth/login.ts", start: 5, end: 16, raw: "src/auth/login.ts:5-16" });
    });
    (0, vitest_1.it)("parses a symbol entry", () => {
        const entry = (0, entry_1.parseEntry)("src/auth/login.ts::authenticate");
        (0, vitest_1.expect)(entry).toEqual({ kind: "symbol", path: "src/auth/login.ts", symbol: "authenticate", raw: "src/auth/login.ts::authenticate" });
    });
    (0, vitest_1.it)("prefers symbol over range when :: present", () => {
        const entry = (0, entry_1.parseEntry)("src/auth/login.ts::LoginPayload");
        (0, vitest_1.expect)(entry?.kind).toBe("symbol");
    });
    (0, vitest_1.it)("trims whitespace from entries", () => {
        const entry = (0, entry_1.parseEntry)("  src/auth/login.ts  ");
        (0, vitest_1.expect)(entry?.kind).toBe("file");
        (0, vitest_1.expect)(entry?.path).toBe("src/auth/login.ts");
    });
});
(0, vitest_1.describe)("parseGeneratedFile", () => {
    (0, vitest_1.it)("parses multiple entries", () => {
        const content = `src/db/migrations/001.sql
src/auth/login.ts:5-16
src/auth/login.ts::authenticate
src/auth/login.ts::LoginPayload`;
        const entries = (0, entry_1.parseGeneratedFile)(content);
        (0, vitest_1.expect)(entries).toHaveLength(4);
        (0, vitest_1.expect)(entries[0].kind).toBe("file");
        (0, vitest_1.expect)(entries[1].kind).toBe("range");
        (0, vitest_1.expect)(entries[2].kind).toBe("symbol");
        (0, vitest_1.expect)(entries[3].kind).toBe("symbol");
    });
    (0, vitest_1.it)("skips blank lines and comments", () => {
        const content = `# Whole file
src/db/migrations/001.sql

# Line ranges
src/auth/login.ts:5-16`;
        const entries = (0, entry_1.parseGeneratedFile)(content);
        (0, vitest_1.expect)(entries).toHaveLength(2);
    });
    (0, vitest_1.it)("returns empty array for empty file", () => {
        (0, vitest_1.expect)((0, entry_1.parseGeneratedFile)("")).toEqual([]);
        (0, vitest_1.expect)((0, entry_1.parseGeneratedFile)("\n\n\n")).toEqual([]);
    });
});
//# sourceMappingURL=entry.test.js.map