# thc1006.us

蔡秀吉的個人網站，頁面由 Rust 產生。首頁的資料在 `content/home.toml`，版面在 `templates/`，樣式在 `static/style.css`。改完跑 `cargo test`，它會把每一頁實際組一次。main 只接受 PR。

## 部署

在 PR 裡把 `k8s/deployment.yaml` 的版本號加一，合併之後在 hsinchu 的 main 上跑：

```sh
scripts/deploy.sh
```

退回上一版：開 PR 把版本號改回去，合併之後再跑一次。
