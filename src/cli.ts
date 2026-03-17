#!/usr/bin/env node
import { Command } from "commander";
import { findGeneratedFile, readEntries, addEntry, removeEntry } from "./store";
import { Entry } from "./entry";

const program = new Command();

program
  .name("tate")
  .description("Track AI-generated code in your repository")
  .version("1.0.0");

program
  .command("list [pathPrefix]")
  .description("Show all entries, optionally filtered by path prefix")
  .action((pathPrefix?: string) => {
    const filePath = findGeneratedFile();
    const entries = readEntries(filePath);

    const filtered: Entry[] = pathPrefix
      ? entries.filter((e) => e.path.startsWith(pathPrefix))
      : entries;

    if (filtered.length === 0) {
      if (pathPrefix) {
        process.stdout.write(`No entries matching "${pathPrefix}"\n`);
      } else {
        process.stdout.write("No entries in GENERATED\n");
      }
      return;
    }

    for (const entry of filtered) {
      process.stdout.write(entry.raw + "\n");
    }
  });

program
  .command("add <entry>")
  .description("Add an entry to GENERATED")
  .action((entry: string) => {
    const filePath = findGeneratedFile();
    addEntry(filePath, entry);
    process.stdout.write(`Added: ${entry.trim()}\n`);
  });

program
  .command("promote <entry>")
  .description("Remove an entry from GENERATED (mark as understood)")
  .action((entry: string) => {
    const filePath = findGeneratedFile();
    const removed = removeEntry(filePath, entry);
    if (removed) {
      process.stdout.write(`Promoted: ${entry.trim()}\n`);
    } else {
      process.stderr.write(`Entry not found: ${entry.trim()}\n`);
      process.exit(1);
    }
  });

program.parse(process.argv);
