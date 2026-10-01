# thc1006.us

蔡秀吉的個人網站。頁面在 `static/`，直接改 HTML；`static/common.js` 由 `ts/common.ts` 編譯，改了它要跑 `pnpm run build`。main 只接受 PR。

## 部署

在 PR 裡把 `k8s/deployment.yaml` 的版本號加一，合併之後在 hsinchu 的 main 上跑：

```sh
scripts/deploy.sh
```

退回上一版：開 PR 把版本號改回去，合併之後再跑一次。
