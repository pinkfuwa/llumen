<div align="center">
  <img src="frontend/static/web-app-manifest-512x512.png" alt="llumen Logo" width="200" height="auto" />

  # Llumen

  [![License: MPL 2.0](https://img.shields.io/badge/License-MPL_2.0-brightgreen.svg)](https://www.mozilla.org/en-US/MPL/2.0/)
  [![Nightly Docker](https://github.com/pinkfuwa/llumen/actions/workflows/docker-nightly.yml/badge.svg)](https://github.com/pinkfuwa/llumen/actions/workflows/docker-nightly.yml)
  [![status check](https://github.com/pinkfuwa/llumen/actions/workflows/check.yml/badge.svg)](https://github.com/pinkfuwa/llumen/actions/workflows/check.yml)
  ![MSRV](https://img.shields.io/static/v1?label=MSRV&message=1.89&color=orange&logo=rust)
  [![Docs](https://img.shields.io/badge/Docs-github%20page-blue)](https://pinkfuwa.github.io/llumen/)
</div>

<div align="center">
  
  [![en](https://img.shields.io/badge/lang-en-green)](./README.md)
  [![zh-tw](https://img.shields.io/badge/lang-zh--TW-green)](./README_zh-TW.md)
  [![zh-cn](https://img.shields.io/badge/lang-zh--CN-green)](./README_zh-CN.md)
</div>

---

## 為什麼選擇 Llumen？

**大多數自託管介面是為伺服器打造的，而非個人裝置。** 它們功能強大，但往往需要大量資源和數小時的設定。

Llumen 注重隱私，也讓設定保持簡單。針對 Raspberry Pi、舊筆電、小型 VPS 等一般硬體進行最佳化，同時保留商業產品的大部分功能。

|  | 隱私 | 效能 | 設定 |
| :--- | :--- | :--- | :--- |
| **商業產品**（ChatGPT） | ❌ 僅限雲端 | ✅ 高 | ✅ 零設定 |
| **典型自託管**（Open WebUI） | ✅ 本地 | ✅ 高 | ❌ 設定繁複 |
| **llumen** | ✅ 本地 | ⚖️ 剛剛好 | ✅ 零設定 |

## 特色

| 特色 | 功能說明 |
| :--- | :--- |
| 速度 | 毫秒級冷啟動，即時串流 |
| 聊天模式 | 一般對話、網路搜尋、深度研究（Agent） |
| 多媒體 | PDF 上傳、LaTeX 公式顯示、圖片生成 |
| 通用 API | 支援相容 OpenAI API 的服務（OpenRouter、本地模型等） |
| 資源用量 | 執行檔大小 ~17MB，記憶體用量 <128MB |

[![Video preview](https://pinkfuwa.github.io/llumen/img/demo/llumen-light.webp)](https://github.com/user-attachments/assets/4d46e649-bd33-4850-af2b-59527cc11618)

## 快速開始

> **預設登入：** `admin` / `P@88w0rd`

### Docker（30 秒快速安裝）

```bash
docker run -it --rm \
  -e API_KEY="<YOUR_OPENROUTER_KEY>" \
  -p 80:80 \
  -v "$(pwd)/data:/data" \
  ghcr.io/pinkfuwa/llumen:latest
```

不需要設定檔，也不需要安裝 Python 依賴套件。

想試用最新功能，可以使用 `ghcr.io/pinkfuwa/llumen:nightly`。

docker-compose 範例請見 [./docs/sample](./docs/sample)。

### 原生執行檔

從 [Releases](https://github.com/pinkfuwa/llumen/releases) 下載 Windows/Linux 版本（包含 arm 架構）。

## 從原始碼編譯

請參閱 [BUILD.md](./BUILD.md) 取得詳細編譯說明。

## 設定（選用）

| 變數 | 描述 | 預設值 |
| :--- | :--- | :--- |
| `API_KEY` | OpenRouter/OpenAI API 金鑰 | *必填* |
| `API_BASE` | 自訂 API Endpoint | `https://openrouter.ai/api` |
| `DATA_PATH` | 儲存資料夾 | `.` |
| `BIND_ADDR` | socket address | `0.0.0.0:80` |

## 文件

https://pinkfuwa.github.io/llumen/


<div align="center">
  Built with ❤️ by pinkfuwa. Keep it simple, keep it fast.
</div>
