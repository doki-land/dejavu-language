#!/usr/bin/env node
import { readFileSync, writeFileSync } from "node:fs";
import cac from "cac";
import { parseToIr } from "@dejavu/language";

const cli = cac("dejavu");

cli.command("parse <file>", "Parse template and emit IR JSON")
    .option("--out <file>", "Write IR to file")
    .action((file: string, opts: { out?: string }) => {
        const src = readFileSync(file, "utf8");
        const ir = JSON.stringify(parseToIr(src), null, 2);
        if (opts.out) writeFileSync(opts.out, ir);
        else process.stdout.write(ir + "\n");
    });

cli.command("render <file>", "Render from IR JSON or source (JIT — temporarily disabled)")
    .option("--from-ir", "Treat input as IR JSON")
    .option("--ctx <file>", "Context JSON file", { default: "" })
    .action(() => {
        console.error(
            "JIT renderIr was removed from @dejavu-engine in B3b. Use dejavu-codegen AOT output instead.",
        );
        process.exit(1);
    });

cli.help();
cli.parse();
