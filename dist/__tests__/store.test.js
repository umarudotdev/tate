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
const vitest_1 = require("vitest");
const fs = __importStar(require("fs"));
const os = __importStar(require("os"));
const path = __importStar(require("path"));
const store_1 = require("../store");
let tmpDir;
let generatedPath;
(0, vitest_1.beforeEach)(() => {
    tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "tate-test-"));
    generatedPath = path.join(tmpDir, "GENERATED");
});
(0, vitest_1.afterEach)(() => {
    fs.rmSync(tmpDir, { recursive: true });
});
(0, vitest_1.describe)("readEntries", () => {
    (0, vitest_1.it)("returns empty array when GENERATED does not exist", () => {
        (0, vitest_1.expect)((0, store_1.readEntries)(generatedPath)).toEqual([]);
    });
    (0, vitest_1.it)("reads entries from GENERATED file", () => {
        fs.writeFileSync(generatedPath, "src/foo.ts\nsrc/bar.ts::myFn\n", "utf8");
        const entries = (0, store_1.readEntries)(generatedPath);
        (0, vitest_1.expect)(entries).toHaveLength(2);
        (0, vitest_1.expect)(entries[0].kind).toBe("file");
        (0, vitest_1.expect)(entries[1].kind).toBe("symbol");
    });
});
(0, vitest_1.describe)("addEntry", () => {
    (0, vitest_1.it)("creates GENERATED file if it does not exist", () => {
        (0, store_1.addEntry)(generatedPath, "src/foo.ts");
        (0, vitest_1.expect)(fs.existsSync(generatedPath)).toBe(true);
        (0, vitest_1.expect)(fs.readFileSync(generatedPath, "utf8")).toBe("src/foo.ts\n");
    });
    (0, vitest_1.it)("appends an entry to GENERATED", () => {
        (0, store_1.addEntry)(generatedPath, "src/foo.ts");
        (0, store_1.addEntry)(generatedPath, "src/bar.ts::myFn");
        const content = fs.readFileSync(generatedPath, "utf8");
        (0, vitest_1.expect)(content).toBe("src/foo.ts\nsrc/bar.ts::myFn\n");
    });
    (0, vitest_1.it)("does not duplicate existing entries", () => {
        (0, store_1.addEntry)(generatedPath, "src/foo.ts");
        (0, store_1.addEntry)(generatedPath, "src/foo.ts");
        const content = fs.readFileSync(generatedPath, "utf8");
        (0, vitest_1.expect)(content).toBe("src/foo.ts\n");
    });
    (0, vitest_1.it)("adds line range entries", () => {
        (0, store_1.addEntry)(generatedPath, "src/auth/login.ts:5-16");
        const entries = (0, store_1.readEntries)(generatedPath);
        (0, vitest_1.expect)(entries).toHaveLength(1);
        (0, vitest_1.expect)(entries[0].kind).toBe("range");
    });
});
(0, vitest_1.describe)("removeEntry", () => {
    (0, vitest_1.it)("returns false when GENERATED does not exist", () => {
        (0, vitest_1.expect)((0, store_1.removeEntry)(generatedPath, "src/foo.ts")).toBe(false);
    });
    (0, vitest_1.it)("returns false when entry is not found", () => {
        (0, store_1.addEntry)(generatedPath, "src/foo.ts");
        (0, vitest_1.expect)((0, store_1.removeEntry)(generatedPath, "src/bar.ts")).toBe(false);
    });
    (0, vitest_1.it)("removes the entry and returns true", () => {
        (0, store_1.addEntry)(generatedPath, "src/foo.ts");
        (0, store_1.addEntry)(generatedPath, "src/bar.ts::myFn");
        (0, vitest_1.expect)((0, store_1.removeEntry)(generatedPath, "src/foo.ts")).toBe(true);
        const entries = (0, store_1.readEntries)(generatedPath);
        (0, vitest_1.expect)(entries).toHaveLength(1);
        (0, vitest_1.expect)(entries[0].raw).toBe("src/bar.ts::myFn");
    });
    (0, vitest_1.it)("writes an empty file when last entry is removed", () => {
        (0, store_1.addEntry)(generatedPath, "src/foo.ts");
        (0, store_1.removeEntry)(generatedPath, "src/foo.ts");
        const content = fs.readFileSync(generatedPath, "utf8");
        (0, vitest_1.expect)(content).toBe("");
    });
});
(0, vitest_1.describe)("writeGeneratedFile", () => {
    (0, vitest_1.it)("writes lines with a trailing newline", () => {
        (0, store_1.writeGeneratedFile)(generatedPath, ["src/foo.ts", "src/bar.ts"]);
        const content = fs.readFileSync(generatedPath, "utf8");
        (0, vitest_1.expect)(content).toBe("src/foo.ts\nsrc/bar.ts\n");
    });
    (0, vitest_1.it)("writes empty string for empty array", () => {
        (0, store_1.writeGeneratedFile)(generatedPath, []);
        const content = fs.readFileSync(generatedPath, "utf8");
        (0, vitest_1.expect)(content).toBe("");
    });
});
//# sourceMappingURL=store.test.js.map