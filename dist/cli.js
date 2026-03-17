#!/usr/bin/env node
"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const commander_1 = require("commander");
const store_1 = require("./store");
const program = new commander_1.Command();
program
    .name("tate")
    .description("Track AI-generated code in your repository")
    .version("1.0.0");
program
    .command("list [pathPrefix]")
    .description("Show all entries, optionally filtered by path prefix")
    .action((pathPrefix) => {
    const filePath = (0, store_1.findGeneratedFile)();
    const entries = (0, store_1.readEntries)(filePath);
    const filtered = pathPrefix
        ? entries.filter((e) => e.path.startsWith(pathPrefix))
        : entries;
    if (filtered.length === 0) {
        if (pathPrefix) {
            process.stdout.write(`No entries matching "${pathPrefix}"\n`);
        }
        else {
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
    .action((entry) => {
    const filePath = (0, store_1.findGeneratedFile)();
    (0, store_1.addEntry)(filePath, entry);
    process.stdout.write(`Added: ${entry.trim()}\n`);
});
program
    .command("promote <entry>")
    .description("Remove an entry from GENERATED (mark as understood)")
    .action((entry) => {
    const filePath = (0, store_1.findGeneratedFile)();
    const removed = (0, store_1.removeEntry)(filePath, entry);
    if (removed) {
        process.stdout.write(`Promoted: ${entry.trim()}\n`);
    }
    else {
        process.stderr.write(`Entry not found: ${entry.trim()}\n`);
        process.exit(1);
    }
});
program.parse(process.argv);
//# sourceMappingURL=cli.js.map