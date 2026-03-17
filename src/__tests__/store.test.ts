import { describe, it, expect, beforeEach, afterEach } from "vitest";
import * as fs from "fs";
import * as os from "os";
import * as path from "path";
import { readEntries, addEntry, removeEntry, writeGeneratedFile } from "../store";

let tmpDir: string;
let generatedPath: string;

beforeEach(() => {
  tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "tate-test-"));
  generatedPath = path.join(tmpDir, "GENERATED");
});

afterEach(() => {
  fs.rmSync(tmpDir, { recursive: true });
});

describe("readEntries", () => {
  it("returns empty array when GENERATED does not exist", () => {
    expect(readEntries(generatedPath)).toEqual([]);
  });

  it("reads entries from GENERATED file", () => {
    fs.writeFileSync(generatedPath, "src/foo.ts\nsrc/bar.ts::myFn\n", "utf8");
    const entries = readEntries(generatedPath);
    expect(entries).toHaveLength(2);
    expect(entries[0].kind).toBe("file");
    expect(entries[1].kind).toBe("symbol");
  });
});

describe("addEntry", () => {
  it("creates GENERATED file if it does not exist", () => {
    addEntry(generatedPath, "src/foo.ts");
    expect(fs.existsSync(generatedPath)).toBe(true);
    expect(fs.readFileSync(generatedPath, "utf8")).toBe("src/foo.ts\n");
  });

  it("appends an entry to GENERATED", () => {
    addEntry(generatedPath, "src/foo.ts");
    addEntry(generatedPath, "src/bar.ts::myFn");
    const content = fs.readFileSync(generatedPath, "utf8");
    expect(content).toBe("src/foo.ts\nsrc/bar.ts::myFn\n");
  });

  it("does not duplicate existing entries", () => {
    addEntry(generatedPath, "src/foo.ts");
    addEntry(generatedPath, "src/foo.ts");
    const content = fs.readFileSync(generatedPath, "utf8");
    expect(content).toBe("src/foo.ts\n");
  });

  it("adds line range entries", () => {
    addEntry(generatedPath, "src/auth/login.ts:5-16");
    const entries = readEntries(generatedPath);
    expect(entries).toHaveLength(1);
    expect(entries[0].kind).toBe("range");
  });
});

describe("removeEntry", () => {
  it("returns false when GENERATED does not exist", () => {
    expect(removeEntry(generatedPath, "src/foo.ts")).toBe(false);
  });

  it("returns false when entry is not found", () => {
    addEntry(generatedPath, "src/foo.ts");
    expect(removeEntry(generatedPath, "src/bar.ts")).toBe(false);
  });

  it("removes the entry and returns true", () => {
    addEntry(generatedPath, "src/foo.ts");
    addEntry(generatedPath, "src/bar.ts::myFn");
    expect(removeEntry(generatedPath, "src/foo.ts")).toBe(true);
    const entries = readEntries(generatedPath);
    expect(entries).toHaveLength(1);
    expect(entries[0].raw).toBe("src/bar.ts::myFn");
  });

  it("writes an empty file when last entry is removed", () => {
    addEntry(generatedPath, "src/foo.ts");
    removeEntry(generatedPath, "src/foo.ts");
    const content = fs.readFileSync(generatedPath, "utf8");
    expect(content).toBe("");
  });
});

describe("writeGeneratedFile", () => {
  it("writes lines with a trailing newline", () => {
    writeGeneratedFile(generatedPath, ["src/foo.ts", "src/bar.ts"]);
    const content = fs.readFileSync(generatedPath, "utf8");
    expect(content).toBe("src/foo.ts\nsrc/bar.ts\n");
  });

  it("writes empty string for empty array", () => {
    writeGeneratedFile(generatedPath, []);
    const content = fs.readFileSync(generatedPath, "utf8");
    expect(content).toBe("");
  });
});
