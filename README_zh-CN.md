<div align="center">
  <img src="frontend/static/web-app-manifest-512x512.png" alt="llumen Logo" width="200" height="auto" />

  # Llumen

  [![License: MPL 2.0](https://img.shields.io/badge/License-MPL_2.0-brightgreen.svg)](https://www.mozilla.org/en-US/MPL/2.0/)
  [![Nightly Docker](https://github.com/pinkfuwa/llumen/actions/workflows/docker-nightly.yml/badge.svg)](https://github.com/pinkfuwa/llumen/actions/workflows/docker-nightly.yml)
  [![status check](https://github.com/pinkfuwa/llumen/actions/workflows/check.yml/badge.svg)](https://github.com/pinkfuwa/llumen/actions/workflows/check.yml)
  ![MSRV](https://img.shields.io/static/v1?label=MSRV&message=1.89&color=orange&logo=rust)
</div>

<div align="center">
  
  [![en](https://img.shields.io/badge/lang-en-green)](./README.md)
  [![zh-tw](https://img.shields.io/badge/lang-zh--TW-green)](./README_zh-TW.md)
  [![zh-cn](https://img.shields.io/badge/lang-zh--CN-green)](./README_zh-CN.md)
</div>

---

## 为什么选择 Llumen？

**大多数自托管界面是为服务器打造的，而非个人设备。** 它们功能强大，但往往需要大量资源和数小时的配置。

Llumen 走出了一条不同的路：**保有隐私，却不复杂**。您获得真正需要的功能，并针对一般硬件(树莓派、旧笔记本、小型 VPS)进行优化，同时保留商业产品的大部分功能。

|  | 隐私 | 性能 | 配置 |
| :--- | :--- | :--- | :--- |
| **商业产品** (ChatGPT) | ❌ 仅限云端 | ✅ 高 | ✅ 零配置 |
| **典型自托管** (Open WebUI) | ✅ 本地 | ✅ 高 | ❌ 地狱配置 |
| **llumen** | ✅ 本地 | ⚖️ 刚刚好 | ✅ 零配置 |

## 特色

| 特色 | 您将获得 |
| :--- | :--- |
| 速度 | 毫秒级冷启动，即时串流 |
| 聊天模式 | 一般对话、网络搜索，以及深度研究(Agent) |
| 丰富媒体 | PDF 上传、LaTeX 渲染、图片生成 |
| 通用 API | 任何兼容 OpenAI Server（OpenRouter、本地模型等） |
| 极小占用 | ~17MB 可执行文件，<128MB 内存使用 |

[![视频预览](./docs/video.webp)](https://github.com/user-attachments/assets/4d46e649-bd33-4850-af2b-59527cc11618)

## 快速开始

> **默认登录：** `admin` / `P@88w0rd`

### Docker (30 秒快速安装)

```bash
docker run -it --rm \
  -e API_KEY="<YOUR_OPENROUTER_KEY>" \
  -p 80:80 \
  -v "$(pwd)/data:/data" \
  ghcr.io/pinkfuwa/llumen:latest
```

就这样。不需要配置文件。不需要 Python 依赖。

**想尝鲜最新功能？** 使用 `ghcr.io/pinkfuwa/llumen:nightly`

请参阅 [./docs/sample](./docs/sample) 查看 docker-compose 示例。

### 原生可执行文件

从 [Releases](https://github.com/pinkfuwa/llumen/releases) 下载 Windows/Linux 版本（包含 arm 架构）。

## 配置 (可选)

| 变量 | 描述 | 默认值 |
| :--- | :--- | :--- |
| `API_KEY` | OpenRouter/OpenAI API 密钥 | *必填* |
| `API_BASE` | 自定义 API Endpoint | `https://openrouter.ai/api` |
| `DATA_PATH` | 存储文件夹 | `.` |
| `BIND_ADDR` | 网络 Socket | `0.0.0.0:80` |

## 文档

https://pinkfuwa.github.io/llumen/


<div align="center">
  Built with ❤️ by pinkfuwa. Keep it simple, keep it fast.
</div>
