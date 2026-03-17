import * as fs from "fs";
import * as path from "path";
import { Entry, parseGeneratedFile, parseEntry } from "./entry";

const GENERATED_FILENAME = "GENERATED";

export function findGeneratedFile(cwd: string = process.cwd()): string {
  return path.join(cwd, GENERATED_FILENAME);
}

export function readGeneratedFile(filePath: string): string {
  if (!fs.existsSync(filePath)) return "";
  return fs.readFileSync(filePath, "utf8");
}

export function readEntries(filePath: string): Entry[] {
  return parseGeneratedFile(readGeneratedFile(filePath));
}

export function writeGeneratedFile(filePath: string, lines: string[]): void {
  const content = lines.join("\n");
  fs.writeFileSync(filePath, content ? content + "\n" : "", "utf8");
}

export function addEntry(filePath: string, raw: string): void {
  const entry = parseEntry(raw);
  if (!entry) throw new Error(`Invalid entry: ${raw}`);

  const content = readGeneratedFile(filePath);
  const lines = content ? content.split("\n").filter((l) => l.trim() !== "") : [];

  if (lines.includes(raw.trim())) {
    return;
  }

  lines.push(raw.trim());
  writeGeneratedFile(filePath, lines);
}

export function removeEntry(filePath: string, raw: string): boolean {
  const content = readGeneratedFile(filePath);
  if (!content) return false;

  const lines = content.split("\n").filter((l) => l.trim() !== "");
  const target = raw.trim();
  const idx = lines.indexOf(target);
  if (idx === -1) return false;

  lines.splice(idx, 1);
  writeGeneratedFile(filePath, lines);
  return true;
}
