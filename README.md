# thc1006.us

蔡秀吉的個人網站。內容來自私有 repo personal-homepage（NYCU 個人網站的原稿），`static/` 由同步腳本和 `pnpm run build` 產生，不要手改。main 只接受 PR。

## 更新內容

在 personal-homepage 改好並推上去之後，在新的分支上同步，再開 PR：

```sh
scripts/sync-homepage.sh   # 停下來時會說明怎麼處理
git add -A static scripts ts && git commit -m "同步 personal-homepage"
```

改了 `ts/script.ts` 的話，先跑 `pnpm run build` 再同步，頁面引用的版本碼才會跟著更新。

## 部署

在 PR 裡把 `k8s/deployment.yaml` 的版本號加一，合併之後在 hsinchu 的 main 上跑：

```sh
scripts/deploy.sh
```

退回上一版：開 PR 把版本號改回去，合併之後再跑一次。
