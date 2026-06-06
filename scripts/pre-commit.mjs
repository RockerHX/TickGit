import { existsSync } from "node:fs";
import { spawnSync } from "node:child_process";

const isWindows = process.platform === "win32";

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    shell: isWindows,
    stdio: "inherit",
    ...options,
  });

  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

function output(command, args) {
  const result = spawnSync(command, args, {
    encoding: "utf8",
    shell: isWindows,
    stdio: ["ignore", "pipe", "inherit"],
  });

  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }

  return result.stdout;
}

function gitNameList(args) {
  return output("git", [...args, "-z"])
    .split("\0")
    .filter(Boolean);
}

const stagedFiles = gitNameList([
  "diff",
  "--cached",
  "--name-only",
  "--diff-filter=ACMR",
]).filter((file) => existsSync(file));

if (stagedFiles.length === 0) {
  console.log("No staged files. Skipping pre-commit checks.");
  process.exit(0);
}

const unstagedFiles = new Set(
  gitNameList(["diff", "--name-only", "--diff-filter=ACMR"]),
);
const partiallyStagedFiles = stagedFiles.filter((file) =>
  unstagedFiles.has(file),
);

if (partiallyStagedFiles.length > 0) {
  console.error(
    [
      "Pre-commit aborted because these files have both staged and unstaged changes:",
      ...partiallyStagedFiles.map((file) => `  - ${file}`),
      "",
      "To avoid accidentally committing unstaged changes, stage or split them first.",
      "You can also run `pnpm format` manually, then `git add` the intended changes.",
    ].join("\n"),
  );
  process.exit(1);
}

console.log("Formatting staged files with Prettier...");
run("pnpm", [
  "exec",
  "prettier",
  "--write",
  "--ignore-unknown",
  "--",
  ...stagedFiles,
]);
run("git", ["add", "--", ...stagedFiles]);

console.log("Running local code checks...");
run("pnpm", ["check"]);
