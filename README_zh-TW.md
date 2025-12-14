<div align="center">
  <img src="frontend/static/web-app-manifest-512x512.png" alt="llumen Logo" width="200" height="auto" />

  # Llumen

  ### :rocket: 臃腫 AI 介面的解決方案。
  ### 打造的輕量、高效能聊天應用程式。

  [![License: MPL 2.0](https://img.shields.io/badge/License-MPL_2.0-brightgreen.svg)](https://www.mozilla.org/en-US/MPL/2.0/)
  [![Nightly Docker](https://github.com/pinkfuwa/llumen/actions/workflows/docker-nightly.yml/badge.svg)](https://github.com/pinkfuwa/llumen/actions/workflows/docker-nightly.yml)
  [![status check](https://github.com/pinkfuwa/llumen/actions/workflows/check.yml/badge.svg)](https://github.com/pinkfuwa/llumen/actions/workflows/check.yml)
  ![MSRV](https://img.shields.io/static/v1?label=MSRV&message=1.89&color=orange&logo=rust)
  [![en](https://img.shields.io/badge/lang-en-green)](./README.md)

  <img src="https://skillicons.dev/icons?i=rust,ts,svelte,tailwind" alt="Technology Stack" width="auto" height="70" />
</div>

---

## :bulb: 為什麼我們打造 llumen

### 「自託管」的取捨：強大但複雜

如果您曾經嘗試在普通設備上自託管 LLM 介面，您一定經歷過這些掙扎：
1.  **臃腫：** 基於 Python 的容器光是待機就吃掉數 GB 的記憶體。
2.  **延遲：** 等待伺服器啟動要 30 秒以上，載入聊天記錄又要再等一分鐘。
3.  **設定地獄：** 為了讓像「標題生成」這樣簡單的功能穩定運作，得花上數小時更改預設 prompt。

### 解決方案：良好工程實現

我們拒絕接受「強大」就必須「笨重」的觀念。我們打造 llumen 是為了填補商業產品（易於設定但無隱私）與進階使用者工具（隱私但笨重且複雜）之間的缺口。

| 特色 | 典型的「進階使用者」UI | **llumen** |
| :--- | :--- | :--- |
| **資源佔用** | 巨大 (GBs) | **極小** (12MB) |
| **記憶體使用量** | 高 (除錯噩夢) | **< 128MB** |
| **設定時間** | 數小時的設定 | **零設定** |

## :sparkles: 特色

別被它的大小騙了。Llumen 雖然資源需求輕量，但功能卻相當強大。

- :electric_plug: **相容 OpenAI：** 適用於 OpenRouter、本地模型或任何相容 OpenAI 的伺服器。
- :rocket: **極致快速：** 瞬間啟動。無需等待。
- :brain: **智慧且深入：** 內建「深度搜尋」功能，整合網路搜尋。
- :art: **豐富媒體支援：** 支援 PDF 上傳、圖片生成，並可渲染複雜的 LaTeX/程式碼。
- :handshake: **隨處執行：** 支援 Windows、Linux、Docker，並針對 **Arm64** 進行最佳化（沒錯，它在 Raspberry Pi 上也能飛快運行）。

[![影片預覽](./docs/video.webp)](https://github.com/user-attachments/assets/4d46e649-bd33-4850-af2b-59527cc11618)

## :zap: 從 :zero: 開始

親自體驗它的速度。如果您有 Docker，只需 30 秒即可開始聊天。

> [!IMPORTANT]
> **預設帳號：**
> - 使用者：`admin`
> - 密碼：`P@88w0rd`

### :whale: Docker

```bash
docker run -it --rm \
  -e API_KEY="<YOUR_OPENROUTER_API_KEY>" \
  -p 80:80 \
  -v "$(pwd)/data:/data" \
  ghcr.io/pinkfuwa/llumen:latest
```

*就這樣。* 不需要複雜設定。不需要安裝依賴套件。

請參閱 [./docs/sample](./docs/sample) 查看 docker-compose 範例。

### :package: 其他方式
想要奇怪的部署方式？我們也支援執行檔。請看 [Releases](https://github.com/pinkfuwa/llumen/releases) 下載 Windows 和 Linux 的二進位檔案。

## :key: 設定

它開箱即用，但如果您想進行調整：

- `API_KEY`（必填）— 您的 OpenRouter/ OpenAI API Key。
- `OPENAI_API_BASE` — 自訂API（預設：`https://openrouter.ai/api`）。
- `DATABASE_URL` — SQLite URL（預設：`sqlite://data/db.sqlite?mode=rwc`）。
- `BIND_ADDR` — 網路介面（預設：`0.0.0.0:80`）。

## :book: 文件

- **使用者指南**：[./docs/user/README.md](./docs/user/README.md) - 完整功能與用法。
- **開發者專區**：
  - 從程式開始建置：`./docs/chore/README.md`
  - 架構文件：`./docs/dev/README.md`

<div align="center">
  Built with ❤️ by pinkfuwa. Keep it simple, keep it fast.
</div>
