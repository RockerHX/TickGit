import { spawnSync } from "node:child_process";

const isWindows = process.platform === "win32";

function run(command, args, options = {}) {
  return spawnSync(command, args, {
    shell: isWindows,
    stdio: "inherit",
    ...options,
  });
}

const gitCheck = run("git", ["rev-parse", "--is-inside-work-tree"], {
  stdio: "ignore",
});

if (gitCheck.status !== 0) {
  process.exit(0);
}

const result = run("git", ["config", "core.hooksPath", ".githooks"]);

if (result.status !== 0) {
  process.exit(result.status ?? 1);
}
