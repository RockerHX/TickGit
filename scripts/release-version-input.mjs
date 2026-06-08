#!/usr/bin/env node

import { createInterface } from "node:readline";

const CANCEL_EXIT_CODE = 130;
const VERSION_PATTERN = /^[0-9]+\.[0-9]+\.[0-9]+$/;

function normalizeVersion(value) {
  return value
    .trim()
    .normalize("NFKC")
    .replace(/[。．｡·]/g, ".")
    .replace(/\s+/g, "");
}

function isValidVersion(value) {
  return VERSION_PATTERN.test(value);
}

function isCancelInput(value) {
  return value.trim().toLowerCase() === "q";
}

function printUsageAndExit() {
  console.error(
    "Usage: node scripts/release-version-input.mjs --normalize <version> | --prompt",
  );
  process.exit(1);
}

function promptVersion() {
  let finished = false;
  let closed = false;

  const rl = createInterface({
    input: process.stdin,
    output: process.stderr,
    terminal: process.stdin.isTTY && process.stderr.isTTY,
  });

  function closeInterface() {
    if (!closed) {
      rl.close();
    }
  }

  function cancel() {
    if (finished) {
      return;
    }

    finished = true;
    process.stderr.write("\n已取消版本号输入。\n");
    closeInterface();
    process.exit(CANCEL_EXIT_CODE);
  }

  function complete(version) {
    if (finished) {
      return;
    }

    finished = true;
    process.stdout.write(`${version}\n`);
    closeInterface();
  }

  function prompt() {
    rl.setPrompt("请输入版本号（输入 q 取消）: ");
    rl.prompt();
  }

  process.once("SIGINT", cancel);
  rl.on("SIGINT", cancel);
  rl.on("close", () => {
    closed = true;

    if (!finished) {
      cancel();
    }
  });

  rl.on("line", (rawVersion) => {
    if (isCancelInput(rawVersion)) {
      cancel();
      return;
    }

    const normalizedVersion = normalizeVersion(rawVersion);

    if (isValidVersion(normalizedVersion)) {
      complete(normalizedVersion);
      return;
    }

    process.stderr.write(`版本号格式无效：${rawVersion}\n`);
    if (normalizedVersion !== rawVersion) {
      process.stderr.write(`已自动归一化为：${normalizedVersion}\n`);
    }
    process.stderr.write("请使用 x.y.z 格式，例如 1.0.1；输入 q 可取消。\n");
    prompt();
  });

  prompt();
}

const [command, ...args] = process.argv.slice(2);

if (command === "--normalize") {
  if (args.length !== 1) {
    printUsageAndExit();
  }

  process.stdout.write(`${normalizeVersion(args[0])}\n`);
} else if (command === "--prompt") {
  promptVersion();
} else {
  printUsageAndExit();
}
