# llumen

> [!IMPORTANT]
> 這是 [llumen](https://github.com/pinkfuwa/llumen) 的一個分支，專為 [梅竹黑客松](https://2025.meichuhackathon.org/) 設計。

llumen 是一個輕量級、可自行託管的 LLM 聊天應用程式（前端 + 後端），旨在為自行託管的用戶提供開箱即用的體驗。

## 主要亮點

- 良好的開箱即用體驗
- 閃電般快速且高品質的使用者介面
- 支援程式碼和數學公式的 Markdown 渲染。
- 為不同使用情境提供多種模式（普通、啟用網路搜尋、代理人模式）。

## 螢幕截圖與影片

TODO: 影片連結

![新聊天](./screenshots/new-chat.png)
![聊天室](./screenshots/chatroom.png)
![設定](./screenshots/setting.png)

## 快速入門

> [!TIP]
> 使用 [reasoning-proxy](https://github.com/Eason0729/reasoning-proxy) 來透過標準的 OpenAI 端點解鎖進階功能。

### Docker (建議)

- 本儲存庫包含一個多階段的 `Dockerfile`，它會建置前端和後端，並產生一個小型的映像檔來提供靜態檔案和運行伺服器。
- 範例：建置並運行容器（預設綁定 80 埠）。

```bash
docker run -it --rm \
  -e API_KEY="<您的_OPENROUTER_API_金鑰>" \
  -p 80:80 \
  -v "$(pwd)/data:/data"
  docker pull ghcr.io/pinkfuwa/llumen:latest
```

### 不使用 Docker

> [!TIP]
> 預先建置的二進位檔案可能不是最新的。

我們也在 release 中提供了預先建置的二進位檔案，下載並解壓縮即可。

## 環境變數

- `API_KEY` (必要) — OpenRouter 或同等提供商的 API 金鑰。
- `DATABASE_URL` — 資料庫連接字串。在 Docker 中的預設值為：`sqlite://data/db.sqlite?mode=rwc`。
- `BIND_ADDR` — 後端綁定的位址（在 Docker 中的預設值為：`0.0.0.0:80`）。

## 更多文件在哪裡

- 開發、從原始碼建置的步驟、進階類型生成以及其他以開發者為中心的文件都已移至 `DEV.md`。如果您想在本地建置或貢獻程式碼，請先閱讀 `DEV.md`。
- 後端原始碼：`backend/` (Rust)。
- 前端原始碼：`frontend/` (SvelteKit)。
- 資料庫遷移：`backend/migration/`。
