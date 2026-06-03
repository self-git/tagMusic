#!/usr/bin/env bash
#
# TagCast DMG 打包脚本（macOS）
# 流程：前置质量门禁（测试 + 类型检查 + clippy） → tauri build → 输出 DMG 路径。
# 用法：bash scripts/build-dmg.sh   或   npm run build:dmg
#
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "==> TagCast 打包开始"

if [[ "$(uname)" != "Darwin" ]]; then
  echo "✗ DMG 打包仅支持 macOS（当前：$(uname)）" >&2
  exit 1
fi

echo "==> [1/4] 前端单元测试"
npm run test

echo "==> [2/4] TypeScript 类型检查"
npm run type-check

echo "==> [3/4] Rust 测试 + clippy"
( cd src-tauri && cargo test && cargo clippy --all-targets -- -D warnings )

echo "==> [4/4] tauri build（编译 release + 生成 .app/.dmg）"
# CI=true 让 Tauri 给 bundle_dmg.sh 传 --skip-jenkins，跳过需要 Finder 自动化权限、
# 在终端环境下常报错（Failed running AppleScript → exit 64）的窗口美化步骤。
# 代价仅为 DMG 少了自定义窗口背景/图标布局，DMG 本体完整可用。
CI=true npm run tauri:build

DMG_DIR="src-tauri/target/release/bundle/dmg"
DMG_PATH="$(ls -t "$DMG_DIR"/*.dmg 2>/dev/null | head -n1 || true)"

if [[ -n "${DMG_PATH:-}" ]]; then
  echo ""
  echo "✓ 打包完成"
  echo "  DMG: $ROOT/$DMG_PATH"
  echo "  APP: $ROOT/src-tauri/target/release/bundle/macos/TagCast.app"
else
  echo "✗ 未在 $DMG_DIR 找到 .dmg，请检查上方 tauri build 输出" >&2
  exit 1
fi
