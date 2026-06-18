#!/usr/bin/env bash
# 从 oa_web.git 拉取最新代码并重建 docker。本地 .env(密钥/VITE_API_BASE)不受影响。
set -euo pipefail
cd "$(dirname "$0")"

REPO="https://github.com/Piercekaoru/oa_web.git"
BRANCH="main"

if [ ! -d .git ]; then
  echo ">> 首次接入 git(保留本地 .env)..."
  git init
  git remote add origin "$REPO"
  git fetch origin "$BRANCH"
  git reset --hard "origin/$BRANCH"   # 只覆盖被跟踪文件;.env 未跟踪,保留
  git branch -M "$BRANCH"
else
  echo ">> 拉取最新代码..."
  git fetch origin "$BRANCH"
  git reset --hard "origin/$BRANCH"
fi

echo ">> 重建并重启 docker..."
docker compose up -d --build
docker image prune -f
echo ">> 完成。"
