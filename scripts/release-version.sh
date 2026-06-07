#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

normalize_version() {
  VERSION_INPUT="$1" python <<'EOF'
import os
import unicodedata

value = os.environ["VERSION_INPUT"].strip()
value = unicodedata.normalize("NFKC", value)
value = (
    value.replace("。", ".")
    .replace("．", ".")
    .replace("｡", ".")
    .replace("·", ".")
    .replace(" ", "")
)
print(value)
EOF
}

read_version_interactively() {
  while true; do
    read -r -p "请输入版本号: " raw_version
    normalized_version="$(normalize_version "${raw_version}")"

    if [[ "${normalized_version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
      printf '%s\n' "${normalized_version}"
      return
    fi

    echo "版本号格式无效：${raw_version}" >&2
    if [[ "${normalized_version}" != "${raw_version}" ]]; then
      echo "已自动归一化为：${normalized_version}" >&2
    fi
    echo "请使用 x.y.z 格式，例如 1.0.1" >&2
  done
}

if [[ $# -ge 1 ]]; then
  VERSION="$(normalize_version "$1")"
else
  VERSION="$(read_version_interactively)"
fi

if [[ ! "${VERSION}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "版本号格式无效：${VERSION}" >&2
  echo "请使用 x.y.z 格式，例如 1.0.1" >&2
  exit 1
fi

if ! git diff --quiet || ! git diff --cached --quiet; then
  echo "工作区存在未提交改动，请先提交或暂存后再执行版本发布脚本。" >&2
  exit 1
fi

TAG="v${VERSION}"
CURRENT_BRANCH="$(git branch --show-current)"

if [[ -z "${CURRENT_BRANCH}" ]]; then
  echo "无法确定当前分支，请在分支上执行该脚本。" >&2
  exit 1
fi

if [[ "${CURRENT_BRANCH}" != "develop" ]]; then
  echo "请在 develop 分支执行该脚本；真正的 Release 只由 main 分支 push 触发。" >&2
  exit 1
fi

git fetch origin \
  '+refs/heads/develop:refs/remotes/origin/develop' \
  '+refs/tags/*:refs/tags/*' \
  --force

if [[ "$(git rev-parse HEAD)" != "$(git rev-parse origin/develop)" ]]; then
  echo "本地 develop 必须与 origin/develop 完全一致后才能执行版本脚本。" >&2
  echo "请先推送代码并等待 origin/develop 最新 CI 通过，再执行该脚本。" >&2
  exit 1
fi

if git rev-parse --verify --quiet "refs/tags/${TAG}" >/dev/null; then
  echo "Tag ${TAG} 已存在，请更换版本号。" >&2
  exit 1
fi

VERSION="${VERSION}" node <<'EOF'
const fs = require("fs");

const version = process.env.VERSION;
const packagePath = "package.json";
const tauriPath = "src-tauri/tauri.conf.json";
const cargoPath = "src-tauri/Cargo.toml";
const cargoLockPath = "src-tauri/Cargo.lock";
const releaseWorkflowPath = ".github/workflows/release.yml";

const packageJson = JSON.parse(fs.readFileSync(packagePath, "utf8"));
packageJson.version = version;
fs.writeFileSync(packagePath, `${JSON.stringify(packageJson, null, 2)}\n`);

const tauriJson = JSON.parse(fs.readFileSync(tauriPath, "utf8"));
tauriJson.version = version;
fs.writeFileSync(tauriPath, `${JSON.stringify(tauriJson, null, 2)}\n`);

const cargoToml = fs.readFileSync(cargoPath, "utf8").replace(
  /^version = "[^"]+"$/m,
  `version = "${version}"`,
);
fs.writeFileSync(cargoPath, cargoToml);

const cargoLock = fs.readFileSync(cargoLockPath, "utf8").replace(
  /(\[\[package\]\]\nname = "tickgit"\nversion = )"[^"]+"/m,
  `$1"${version}"`,
);
fs.writeFileSync(cargoLockPath, cargoLock);

const releaseWorkflowSource = fs.readFileSync(releaseWorkflowPath, "utf8");
const releaseWorkflow = releaseWorkflowSource.replace(
  /^run-name: Release v[0-9]+\.[0-9]+\.[0-9]+$/m,
  `run-name: Release v${version}`,
);
if (releaseWorkflow === releaseWorkflowSource) {
  throw new Error(
    `${releaseWorkflowPath} is missing a literal "run-name: Release vX.Y.Z" line.`,
  );
}
fs.writeFileSync(releaseWorkflowPath, releaseWorkflow);
EOF

pnpm exec prettier --write package.json src-tauri/tauri.conf.json

git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml src-tauri/Cargo.lock .github/workflows/release.yml
git commit -m "Release ${TAG}"
git tag "${TAG}"
git push origin "${CURRENT_BRANCH}"
git push origin "${TAG}"

echo "已发布版本 ${VERSION}"
echo "提交信息: Release ${TAG}"
echo "已创建 tag: ${TAG}"
echo "已推送分支: ${CURRENT_BRANCH}"
