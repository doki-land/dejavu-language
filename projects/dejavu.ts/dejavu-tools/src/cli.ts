#!/usr/bin/env node
import {readFileSync, writeFileSync} from "node:fs";
import cac from "cac";
import {engine, renderIr, type IrDocument} from "@dejavu/engine";

const cli = cac("dejavu");

cli.command("parse <file>", "Parse template and emit IR JSON")
    .option("--out <file>", "Write IR to file")
    .action((file: string, opts: { out?: string }) => {
        const src = readFileSync(file, "utf8");
        const ir = JSON.stringify(engine.parse(src), null, 2);
        if (opts.out) writeFileSync(opts.out, ir);
        else process.stdout.write(ir + "\n");
    });

cli.command("render <file>", "Render from IR JSON or source")
    .option("--from-ir", "Treat input as IR JSON")
    .option("--ctx <file>", "Context JSON file", {default: ""})
    .action((file: string, opts: { fromIr?: boolean; ctx?: string }) => {
        const raw = readFileSync(file, "utf8");
        const ctx = opts.ctx ? JSON.parse(readFileSync(opts.ctx, "utf8")) : {};
        const out = opts.fromIr
            ? renderIr(JSON.parse(raw) as IrDocument, ctx)
            : engine.renderSource(raw, ctx);
        process.stdout.write(out);
    });

cli.help();
cli.parse();
