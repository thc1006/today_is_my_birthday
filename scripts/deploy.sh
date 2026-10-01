#!/usr/bin/env bash
# 把 main 上的版本部署到 hsinchu 的 k3s。
#
# 版本號寫在 k8s/deployment.yaml，要先開 PR 改好並合併，再在這台機器的 main 上跑這支腳本。
# k3s 裡還沒有這個版本的映像檔時，會從目前的 commit 建置並匯入；已經有的話直接沿用，
# 不會重新建置覆蓋它。所以退回上一版的做法一樣：開 PR 把版本號改回去，合併後再跑一次。
set -euo pipefail
cd "$(dirname "$0")/.."

git fetch -q origin main
if [[ -n "$(git status --porcelain)" || "$(git rev-parse HEAD)" != "$(git rev-parse origin/main)" ]]; then
  echo "✗ 請在乾淨、而且和 origin/main 相同的 commit 上部署，線上才會等於 GitHub 上的 main。" >&2
  exit 1
fi

image=$(sed -n 's#^[[:space:]]*image:[[:space:]]*\(localhost/thc1006-web:[^[:space:]]*\)[[:space:]]*$#\1#p' k8s/deployment.yaml)
if [[ -z "$image" ]]; then
  echo "✗ k8s/deployment.yaml 裡找不到 localhost/thc1006-web 的 image。" >&2
  exit 1
fi
revision=$(git rev-parse HEAD)

if sudo k3s ctr -n k8s.io images ls -q | grep -qxF "$image"; then
  built=$(podman image inspect --format '{{index .Labels "org.opencontainers.image.revision"}}' "$image" 2>/dev/null || true)
  echo "! k3s 已經有 $image（建置自 ${built:-不明的 commit}），直接沿用，不重新建置。" >&2
  echo "  如果是要部署新的內容，請先開 PR 把 k8s/deployment.yaml 的版本號加一。" >&2
else
  source_dir=$(mktemp -d)
  trap 'rm -rf "$source_dir"' EXIT
  git archive HEAD | tar -x -C "$source_dir"
  podman build -t "$image" --label "org.opencontainers.image.revision=$revision" "$source_dir"
  podman save "$image" | sudo k3s ctr -n k8s.io images import -
fi

kubectl apply -f k8s/deployment.yaml
kubectl -n thc1006-web rollout status deploy/thc1006-web --timeout=180s
echo "✓ 線上是 $image，對應 main 的 ${revision:0:7}"
