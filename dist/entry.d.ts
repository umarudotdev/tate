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
export declare function parseEntry(line: string): Entry | null;
export declare function parseGeneratedFile(content: string): Entry[];
export declare function formatEntry(entry: Entry): string;
export declare function entryKey(entry: Entry): string;
//# sourceMappingURL=entry.d.ts.map