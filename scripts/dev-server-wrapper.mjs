#!/usr/bin/env node

import { existsSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";

const rootDir = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const viteCli = resolve(rootDir, "node_modules/vite/bin/vite.js");
const args = process.argv.slice(2);
const viteArgs = args.length > 0 ? args : ["dev"];

if (!existsSync(viteCli)) {
  console.error("找不到 Vite，请先执行 pnpm install");
  process.exit(1);
}

let stopping = false;
let childExited = false;

const child = spawn(process.execPath, [viteCli, ...viteArgs], {
  cwd: rootDir,
  env: process.env,
  stdio: "inherit",
});

function stop(signal) {
  if (stopping || childExited) {
    return;
  }

  stopping = true;
  child.kill(signal);
}

process.on("SIGINT", () => stop("SIGINT"));
process.on("SIGTERM", () => stop("SIGTERM"));

child.on("error", (error) => {
  console.error(error.message);
  process.exit(1);
});

child.on("exit", (code, signal) => {
  childExited = true;

  if (stopping || signal === "SIGINT" || signal === "SIGTERM") {
    process.exit(0);
  }

  process.exit(code ?? 0);
});
