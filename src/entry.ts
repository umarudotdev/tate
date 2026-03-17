export type EntryKind = "file" | "range" | "symbol";

export interface FileEntry {
  kind: "file";
  path: string;
  raw: string;
}

export interface RangeEntry {
  kind: "range";
  path: string;
  start: number;
  end: number;
  raw: string;
}

export interface SymbolEntry {
  kind: "symbol";
  path: string;
  symbol: string;
  raw: string;
}

export type Entry = FileEntry | RangeEntry | SymbolEntry;

const RANGE_RE = /^(.+):(\d+)-(\d+)$/;
const SYMBOL_RE = /^(.+)::(.+)$/;

export function parseEntry(line: string): Entry | null {
  const trimmed = line.trim();
  if (!trimmed || trimmed.startsWith("#")) return null;

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

export function parseGeneratedFile(content: string): Entry[] {
  return content
    .split("\n")
    .map(parseEntry)
    .filter((e): e is Entry => e !== null);
}

export function formatEntry(entry: Entry): string {
  return entry.raw;
}

export function entryKey(entry: Entry): string {
  return entry.raw;
}
