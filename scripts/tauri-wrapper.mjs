#!/usr/bin/env node

import { existsSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";

const rootDir = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const tauriCli = resolve(rootDir, "node_modules/@tauri-apps/cli/tauri.js");
const args = process.argv.slice(2);
const isDevCommand = args[0] === "dev";

if (!existsSync(tauriCli)) {
  console.error("找不到 @tauri-apps/cli，请先执行 pnpm install");
  process.exit(1);
}

function spawnTauri(stdio) {
  return spawn(process.execPath, [tauriCli, ...args], {
    cwd: rootDir,
    env: process.env,
    stdio,
    detached: isDevCommand && process.platform !== "win32",
  });
}

if (!isDevCommand || !process.stdin.isTTY) {
  const child = spawnTauri("inherit");
  child.on("error", (error) => {
    console.error(error.message);
    process.exit(1);
  });
  child.on("exit", (code, signal) => {
    if (signal) {
      process.kill(process.pid, signal);
      return;
    }

    process.exit(code ?? 0);
  });
} else {
  const child = spawnTauri(["pipe", "inherit", "inherit"]);
  let stopping = false;
  let forceKillTimer = null;

  child.stdin.on("error", () => {
    // 退出过程中子进程可能先关闭 stdin，忽略残留按键转发失败即可。
  });

  const restoreStdin = () => {
    if (process.stdin.isTTY) {
      process.stdin.setRawMode(false);
    }
    process.stdin.pause();
  };

  const killChild = (signal) => {
    if (process.platform === "win32") {
      child.kill(signal);
      return;
    }

    // Tauri dev 会拉起 Vite / Cargo 等子进程，按进程组发送信号才能一起退出。
    try {
      process.kill(-child.pid, signal);
    } catch {
      child.kill(signal);
    }
  };

  const stopChild = () => {
    if (stopping) {
      return;
    }

    stopping = true;
    process.stdout.write("\n正在退出 Tauri dev...\n");
    restoreStdin();
    killChild("SIGINT");

    forceKillTimer = setTimeout(() => {
      killChild("SIGTERM");
    }, 2500);
  };

  console.log("按 q 退出 Tauri dev。");
  process.stdin.setRawMode(true);
  process.stdin.resume();
  process.stdin.on("data", (chunk) => {
    const input = chunk.toString("utf8");

    if (input === "q" || input === "Q" || input === "\u0003") {
      stopChild();
      return;
    }

    if (child.stdin.writable) {
      child.stdin.write(chunk);
    }
  });

  child.on("exit", (code, signal) => {
    restoreStdin();
    if (forceKillTimer) {
      clearTimeout(forceKillTimer);
    }

    if (stopping) {
      process.exit(0);
    }

    if (signal) {
      process.kill(process.pid, signal);
      return;
    }

    process.exit(code ?? 0);
  });

  child.on("error", (error) => {
    restoreStdin();
    console.error(error.message);
    process.exit(1);
  });
}
