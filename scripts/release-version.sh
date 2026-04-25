#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

if [[ $# -ge 1 ]]; then
  VERSION="$1"
else
  read -r -p "请输入版本号: " VERSION
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

if git rev-parse --verify --quiet "${TAG}" >/dev/null; then
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
EOF

git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "Release ${TAG}"
git tag "${TAG}"

echo "已发布版本 ${VERSION}"
echo "提交信息: Release ${TAG}"
echo "已创建 tag: ${TAG}"
