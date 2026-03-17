import { describe, it, expect } from "vitest";
import { parseEntry, parseGeneratedFile } from "../entry";

describe("parseEntry", () => {
  it("returns null for empty lines", () => {
    expect(parseEntry("")).toBeNull();
    expect(parseEntry("   ")).toBeNull();
  });

  it("returns null for comment lines", () => {
    expect(parseEntry("# This is a comment")).toBeNull();
    expect(parseEntry("  # comment")).toBeNull();
  });

  it("parses a whole-file entry", () => {
    const entry = parseEntry("src/db/migrations/001.sql");
    expect(entry).toEqual({ kind: "file", path: "src/db/migrations/001.sql", raw: "src/db/migrations/001.sql" });
  });

  it("parses a line-range entry", () => {
    const entry = parseEntry("src/auth/login.ts:5-16");
    expect(entry).toEqual({ kind: "range", path: "src/auth/login.ts", start: 5, end: 16, raw: "src/auth/login.ts:5-16" });
  });

  it("parses a symbol entry", () => {
    const entry = parseEntry("src/auth/login.ts::authenticate");
    expect(entry).toEqual({ kind: "symbol", path: "src/auth/login.ts", symbol: "authenticate", raw: "src/auth/login.ts::authenticate" });
  });

  it("prefers symbol over range when :: present", () => {
    const entry = parseEntry("src/auth/login.ts::LoginPayload");
    expect(entry?.kind).toBe("symbol");
  });

  it("trims whitespace from entries", () => {
    const entry = parseEntry("  src/auth/login.ts  ");
    expect(entry?.kind).toBe("file");
    expect(entry?.path).toBe("src/auth/login.ts");
  });
});

describe("parseGeneratedFile", () => {
  it("parses multiple entries", () => {
    const content = `src/db/migrations/001.sql
src/auth/login.ts:5-16
src/auth/login.ts::authenticate
src/auth/login.ts::LoginPayload`;
    const entries = parseGeneratedFile(content);
    expect(entries).toHaveLength(4);
    expect(entries[0].kind).toBe("file");
    expect(entries[1].kind).toBe("range");
    expect(entries[2].kind).toBe("symbol");
    expect(entries[3].kind).toBe("symbol");
  });

  it("skips blank lines and comments", () => {
    const content = `# Whole file
src/db/migrations/001.sql

# Line ranges
src/auth/login.ts:5-16`;
    const entries = parseGeneratedFile(content);
    expect(entries).toHaveLength(2);
  });

  it("returns empty array for empty file", () => {
    expect(parseGeneratedFile("")).toEqual([]);
    expect(parseGeneratedFile("\n\n\n")).toEqual([]);
  });
});
