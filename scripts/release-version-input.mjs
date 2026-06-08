#!/usr/bin/env node

import { createInterface } from "node:readline/promises";

function normalizeVersion(value) {
  return value
    .trim()
    .normalize("NFKC")
    .replace(/[。．｡·]/g, ".")
    .replace(/\s+/g, "");
}

function isValidVersion(value) {
  return /^[0-9]+\.[0-9]+\.[0-9]+$/.test(value);
}

function printUsageAndExit() {
  console.error(
    "Usage: node scripts/release-version-input.mjs --normalize <version> | --prompt",
  );
  process.exit(1);
}

async function promptVersion() {
  const rl = createInterface({
    input: process.stdin,
    output: process.stderr,
    terminal: process.stderr.isTTY,
  });

  const cancel = () => {
    process.stderr.write("\n已取消版本号输入。\n");
    rl.close();
    process.exit(130);
  };

  process.once("SIGINT", cancel);

  try {
    while (true) {
      const rawVersion = await rl.question("请输入版本号: ");
      const normalizedVersion = normalizeVersion(rawVersion);

      if (isValidVersion(normalizedVersion)) {
        process.stdout.write(`${normalizedVersion}\n`);
        return;
      }

      process.stderr.write(`版本号格式无效：${rawVersion}\n`);
      if (normalizedVersion !== rawVersion) {
        process.stderr.write(`已自动归一化为：${normalizedVersion}\n`);
      }
      process.stderr.write("请使用 x.y.z 格式，例如 1.0.1\n");
    }
  } finally {
    process.removeListener("SIGINT", cancel);
    rl.close();
  }
}

const [command, ...args] = process.argv.slice(2);

if (command === "--normalize") {
  if (args.length !== 1) {
    printUsageAndExit();
  }

  process.stdout.write(`${normalizeVersion(args[0])}\n`);
} else if (command === "--prompt") {
  await promptVersion();
} else {
  printUsageAndExit();
}
