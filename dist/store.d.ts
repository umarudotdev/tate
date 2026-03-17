import { Entry } from "./entry";
export declare function findGeneratedFile(cwd?: string): string;
export declare function readGeneratedFile(filePath: string): string;
export declare function readEntries(filePath: string): Entry[];
export declare function writeGeneratedFile(filePath: string, lines: string[]): void;
export declare function addEntry(filePath: string, raw: string): void;
export declare function removeEntry(filePath: string, raw: string): boolean;
//# sourceMappingURL=store.d.ts.map