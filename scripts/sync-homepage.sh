#!/usr/bin/env bash
# 把網站內容從 personal-homepage 同步到 static/。
#
# personal-homepage 是 NYCU 個人網站的原稿（私有 repo），thc1006.us 用的是同一份內容。
# 網站內容請在 personal-homepage 改；static/ 由這支腳本產生，在這裡手改的東西下次同步就會不見。
#
# 平常的用法：
#   scripts/sync-homepage.sh            抓 personal-homepage 最新的 main（需要 gh 已登入）
#   scripts/sync-homepage.sh <clone>    改用本機的 clone，只取它 HEAD 那個 commit 的內容
# 跑完用 git diff 看改了什麼，再 commit 和部署，步驟見 README。
#
# 遇到這些情況，腳本會停下來並說明怎麼處理，static/ 不會被動到：
#   - clone 比上次同步的版本舊（確定要退回舊版時，加 ALLOW_STALE=1）
#   - 上游改了 script.js：這裡用的是改寫成 TypeScript 的 ts/script.ts，要先跟著改
#   - 上游改了首頁「延伸」那一行：本站多加的三個連結（scripts/homepage-local.patch）要重做
#   - 上游的內容檢查沒過、檔案清單或網址寫法變了，或出現不該公開的檔案
#
# 只會取對方 deploy.sh 實際上線的檔案；tools/、README 這些留在私有 repo，不會公開到這裡。
set -euo pipefail
unset CDPATH

# clone 路徑要在切換目錄之前解析，相對路徑才會以執行時所在的目錄為準。
SRC_ARG=""
if [[ $# -ge 1 ]]; then SRC_ARG=$(cd "$1" && pwd); fi
cd "$(dirname "$0")/.."

LOCK=scripts/homepage-upstream.lock
LOCAL_PATCH=scripts/homepage-local.patch
NYCU_BLOG=https://people.cs.nycu.edu.tw/~hctsai1006/blog

# 所有步驟都在暫存目錄裡做，全部成功才換進 static/。
STAGE=$(mktemp -d)
cleanup=("$STAGE")
trap 'rm -rf "${cleanup[@]}"' EXIT

if [[ -n "$SRC_ARG" ]]; then
  SRC=$SRC_ARG
  if ! git -C "$SRC" diff --quiet HEAD; then
    echo "✗ $SRC 有還沒 commit 的修改。同步只取 commit 過的內容，請先 commit 或還原再跑。" >&2
    exit 1
  fi
else
  SRC=$(mktemp -d)
  cleanup+=("$SRC")
  # 要有歷史才能檢查有沒有落後；--filter=blob:none 只抓歷史，檔案內容用到時才下載。
  gh repo clone thc1006/personal-homepage "$SRC" -- --filter=blob:none --quiet
fi
COMMIT=$(git -C "$SRC" rev-parse HEAD)

# 上次同步的 commit 必須是這次 HEAD 的祖先，否則同步下去會把網站往回推。
LAST=$(sed -n 's/^COMMIT=//p' "$LOCK" | tr -d '\r')
if [[ -n "$LAST" && "$LAST" != "$COMMIT" && -z "${ALLOW_STALE:-}" ]] &&
  ! git -C "$SRC" merge-base --is-ancestor "$LAST" HEAD 2>/dev/null; then
  echo "✗ 這個 clone 的 HEAD（${COMMIT:0:7}）不包含上次同步的 ${LAST:0:7}，同步下去會把網站往回推。" >&2
  echo "  先更新 clone（git pull）；確定要退回舊版，加上 ALLOW_STALE=1 再跑。" >&2
  exit 1
fi

SCRIPT_SHA256=$(sed -n 's/^SCRIPT_SHA256=//p' "$LOCK" | tr -d '\r')
actual=$(git -C "$SRC" show HEAD:script.js | sha256sum | cut -d' ' -f1)
if [[ "$actual" != "$SCRIPT_SHA256" ]]; then
  echo "✗ 上游的 script.js 改過了，ts/script.ts 要先跟上。" >&2
  echo "  1. 看上游改了什麼：在 personal-homepage 的 clone 裡跑 git log -p -- script.js" >&2
  echo "  2. 照著修改 ts/script.ts，跑 pnpm run build" >&2
  echo "  3. 把 $LOCK 的 SCRIPT_SHA256 換成下面這個值，再重新同步：" >&2
  echo "     $actual" >&2
  exit 1
fi
if [[ ! -f static/script.js ]]; then
  echo "✗ 找不到 static/script.js，先跑 pnpm run build。" >&2
  exit 1
fi

# 上游部署到 NYCU 之前會跑這兩支檢查，沒過就不部署；這裡同樣把關。
if command -v node >/dev/null 2>&1; then
  for check in tools/invariants.js tools/ai-tells.mjs; do
    if [[ -f "$SRC/$check" ]] && ! (cd "$SRC" && node "$check" >/dev/null 2>&1); then
      echo "✗ 上游的 $check 沒有通過，NYCU 那邊也不會部署這個版本。請先在 personal-homepage 修好。" >&2
      exit 1
    fi
  done
else
  echo "! 這台機器沒有 node，沒跑上游的 invariants.js 與 ai-tells.mjs；請確認這個 commit 在 personal-homepage 那邊通過了。" >&2
fi

# 要同步哪些檔案，照對方 deploy.sh 的清單；對方新增頁面時這裡不用跟著改。
# 只讀出清單、不執行 deploy.sh：它是程式碼，用 eval 讀會連裡面的命令一起執行。
# 只讀得懂單行、只指定一次的寫法；用了 += 或重複指定時停下，免得漏掉檔案。
for name in TOP_FILES SUBDIRS RETIRED_PATHS; do
  count=$(grep -cE "^[[:space:]]*${name}\+?=" "$SRC/deploy.sh" || true)
  if [[ "$count" -gt 1 || ("$name" != RETIRED_PATHS && "$count" -ne 1) ]]; then
    echo "✗ deploy.sh 對 $name 的寫法這支腳本讀不懂（找到 $count 處指定），請更新 deploy_list。" >&2
    exit 1
  fi
done
deploy_list() { sed -n "s/^$1=(\(.*\))[[:space:]]*\$/\1/p" "$SRC/deploy.sh"; }
read -ra TOP_FILES <<<"$(deploy_list TOP_FILES)"
read -ra SUBDIRS <<<"$(deploy_list SUBDIRS)"
read -ra RETIRED_PATHS <<<"$(deploy_list RETIRED_PATHS)"
if [[ ${#TOP_FILES[@]} -eq 0 || ${#SUBDIRS[@]} -eq 0 ]]; then
  echo "✗ 讀不到 deploy.sh 裡的 TOP_FILES 或 SUBDIRS，對方可能改了寫法。請更新這支腳本的 deploy_list。" >&2
  exit 1
fi
paths=()
for item in "${TOP_FILES[@]}" "${SUBDIRS[@]}"; do
  # 名稱開頭必須是英數字。. 會匯出整個 repo，隱藏檔多半是設定，都不該出現在網站上。
  if [[ ! "$item" =~ ^[A-Za-z0-9][A-Za-z0-9._-]*$ ]]; then
    echo "✗ deploy.sh 的清單裡有不該公開的項目：$item" >&2
    exit 1
  fi
  # 和 deploy.sh 一樣跳過不存在的項目；script.js 由 ts/script.ts 編譯，不從上游拿。
  if [[ "$item" != script.js ]] && git -C "$SRC" cat-file -e "HEAD:$item" 2>/dev/null; then
    paths+=("$item")
  fi
done

# 用 git 的檔案樹檢查，不看解開後的檔案：Windows 的 tar 會把符號連結換成目標檔案的複本。
tree=$(git -C "$SRC" ls-tree -r --full-tree HEAD -- "${paths[@]}")
links=$(awk -F'\t' '$1 ~ /^120000 / {print $2}' <<<"$tree")
if [[ -n "$links" ]]; then
  echo "✗ 上游有符號連結。伺服器會跟著連結把目標檔案送出去，所以不同步：" >&2
  echo "$links" >&2
  exit 1
fi
hidden=$(awk -F'\t' '{print $2}' <<<"$tree" | grep -E '(^|/)\.' || true)
if [[ -n "$hidden" ]]; then
  echo "✗ 上游要同步的資料夾裡有隱藏檔，伺服器會把它們當一般檔案送出去：" >&2
  echo "$hidden" >&2
  exit 1
fi

# 從 commit 匯出，不讀工作樹，所以 clone 裡沒追蹤的草稿不會被帶上線。
git -C "$SRC" archive HEAD -- "${paths[@]}" | tar -x -C "$STAGE"

# 上游停用的檔案，deploy.sh 會從 NYCU 伺服器刪掉；這裡同樣不公開。
for retired in "${RETIRED_PATHS[@]}"; do
  if [[ ! "$retired" =~ ^[A-Za-z0-9][A-Za-z0-9._/-]*$ || "$retired" == *..* || "$retired" == */.* ]]; then
    echo "✗ deploy.sh 的 RETIRED_PATHS 有看不懂的項目：$retired" >&2
    exit 1
  fi
  rm -f -- "$STAGE/$retired"
done

for f in index.html 404.html style.css; do
  if [[ ! -f "$STAGE/$f" ]]; then
    echo "✗ 上游少了 $f。404.html 會被編進伺服器執行檔，style.css 是每一頁都要用的樣式。" >&2
    exit 1
  fi
done

# 對方的網站掛在 /~hctsai1006/ 底下，這裡在網域根目錄，網址要依序改寫：
#   1. 完整網址 https://people.cs.nycu.edu.tw/~hctsai1006/ → https://thc1006.us/（canonical、og、JSON-LD）
#   2. 頁面上顯示的 people.cs.nycu.edu.tw/~hctsai1006/ → thc1006.us/
#   3. /~hctsai1006/blog → NYCU 上的完整網址，blog 不在 personal-homepage 裡，這裡沒有
#   4. 其餘的 /~hctsai1006/ → /
# 指向 blog 的網址在每一條都保留原樣或改成 NYCU 的完整網址，不會被改成 thc1006.us。
find "$STAGE" -name '*.html' -exec perl -pi -e '
  s{\Qhttps://people.cs.nycu.edu.tw/~hctsai1006/\E(?!blog\b)}{https://thc1006.us/}g;
  s{\Qpeople.cs.nycu.edu.tw/~hctsai1006/\E(?!blog\b)}{thc1006.us/}g;
  s{(["=])/~hctsai1006/blog\b}{${1}https://people.cs.nycu.edu.tw/~hctsai1006/blog}g;
  s{(["=])/~hctsai1006/(?!blog\b)}{${1}/}g;
' {} +

if [[ -n "${SKIP_LOCAL_PATCH:-}" ]]; then
  echo "! 依 SKIP_LOCAL_PATCH 略過 $LOCAL_PATCH：static/ 會少本站多加的連結，只用來重做 patch，不要 commit。" >&2
elif ! grep -q '^--- ' "$LOCAL_PATCH"; then
  : # patch 檔只剩說明、沒有本體，沒有東西要套。
elif patch -p1 -d "$STAGE" --dry-run --forward --fuzz=0 --quiet <"$LOCAL_PATCH" >/dev/null 2>&1; then
  patch -p1 -d "$STAGE" --forward --fuzz=0 --no-backup-if-mismatch --quiet <"$LOCAL_PATCH" >/dev/null
elif patch -p1 -d "$STAGE" --dry-run -R --fuzz=0 --quiet <"$LOCAL_PATCH" >/dev/null 2>&1; then
  echo "! 上游已經有 $LOCAL_PATCH 要加的內容，這次不用套。請把 patch 檔的本體刪掉，只留說明。" >&2
else
  echo "✗ $LOCAL_PATCH 套不上，上游改到了首頁「延伸」那一行。重做的步驟寫在 patch 檔開頭。" >&2
  exit 1
fi

# 改寫後，/~hctsai1006/ 只該出現在指向 NYCU blog 的網址。出現別的寫法，代表上游換了格式，
# 要在上面補一條改寫規則。CSS 這類不改寫的檔案也一起檢查。
leftover=$(grep -rnI '~hctsai1006' "$STAGE" | sed "s#${NYCU_BLOG//./\\.}##g" | grep '~hctsai1006' || true)
if [[ -n "$leftover" ]]; then
  echo "✗ 改寫後還有指向 /~hctsai1006/ 的網址，請在這支腳本補上對應的改寫規則：" >&2
  echo "$leftover" | cut -c1-200 >&2
  exit 1
fi

# 頁面用 ?v= 讓瀏覽器在檔案變了時抓新版。上游的值是手動維護的，script.js 在這裡又是另外編的，
# 所以都改成實際檔案內容的雜湊。CI 會檢查這兩個值和檔案對得上。
css_v=$(sha256sum "$STAGE/style.css" | cut -c1-8)
js_v=$(sha256sum static/script.js | cut -c1-8)
find "$STAGE" -name '*.html' -exec perl -pi -e "s{(/style\\.css\\?v=)[0-9a-f]+}{\${1}$css_v}g; s{(/script\\.js\\?v=)[0-9a-f]+}{\${1}$js_v}g" {} +
unversioned=$(grep -rhoE '(href|src)="/(style\.css|script\.js)[^"]*"' "$STAGE" --include='*.html' | grep -vE "\?v=($css_v|$js_v)\"$" || true)
if [[ -n "$unversioned" ]]; then
  echo "✗ 有頁面引用 style.css 或 script.js 時沒帶 ?v= 版本碼，改版後瀏覽器會繼續用舊檔：" >&2
  echo "$unversioned" | sort -u >&2
  exit 1
fi

find static -mindepth 1 -maxdepth 1 ! -name script.js -exec rm -rf {} +
cp -R "$STAGE"/. static/

cat >"$LOCK" <<EOF
# static/ 是從 personal-homepage 的哪個 commit 同步來的，以及 ts/script.ts 對應上游哪一版 script.js。
# COMMIT 由 scripts/sync-homepage.sh 自動更新；SCRIPT_SHA256 在改完 ts/script.ts 之後手動更新。
COMMIT=$COMMIT
SCRIPT_SHA256=$SCRIPT_SHA256
EOF

echo "✓ static/ 已同步到 personal-homepage ${COMMIT:0:7}"
echo "  用 git diff --stat static/ 看改了什麼，沒問題就 commit；部署步驟見 README。"
