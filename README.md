# Mua

<p align="center">
  <img src="src-tauri/icons/128x128.png" alt="Mua Logo" width="112" height="112" />
</p>

<p align="center">
  <strong>一个基于 Tauri 2 + Svelte 5 + aria2 的桌面下载管理器</strong>
</p>

<p align="center">
  <a href="./LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License" /></a>
  <img src="https://img.shields.io/badge/Tauri-2-orange.svg" alt="Tauri" />
  <img src="https://img.shields.io/badge/Svelte-5-red.svg" alt="Svelte" />
  <img src="https://img.shields.io/badge/Rust-stable-black.svg" alt="Rust" />
  <img src="https://img.shields.io/badge/PRs-welcome-brightgreen.svg" alt="PRs Welcome" />
</p>

Mua 致力于提供稳定、轻量、可扩展的下载体验：前端负责高响应交互，Rust 后端负责任务持久化、同步和 aria2 sidecar 管理。

> 说明：本项目在实际开发过程中引入了 AI 辅助进行方案讨论、代码生成与重构校验。所有合入代码仍以人工评审与功能验证为准。

## ✨ 功能特性

- 多协议下载管理（HTTP/HTTPS、Magnet、Torrent）
- 轻量跨平台架构（aria2 sidecar 部分平台需自行集成）
- 现代化界面与主题化视觉系统
- 可定制下载环境（支持自定义 aria2 内核与配置）

## 📸 运行截图

<table>
  <tr>
    <td width="50%"><img src="screenshot/c71244eaa77b4b6498a696b7e102b7d6.png" alt="Mua Screenshot 1" /></td>
    <td width="50%"><img src="screenshot/7182d009a63a4eac9b73eb02f5cd61b8.png" alt="Mua Screenshot 2" /></td>
  </tr>
  <tr>
    <td width="50%"><img src="screenshot/92992fda5cea46918fa5143fbe2a2c58.png" alt="Mua Screenshot 3" /></td>
    <td width="50%"><img src="screenshot/47f54c62edba47d9b4ae9ec74d13ae49.png" alt="Mua Screenshot 4" /></td>
  </tr>
  <tr>
    <td width="50%"><img src="screenshot/c0e2cd4e85724daea7ae28dc6acdd788.png" alt="Mua Screenshot 5" /></td>
    <td width="50%"><img src="screenshot/e44e96c70f4a4bbc95fda4188367278f.png" alt="Mua Screenshot 6" /></td>
  </tr>
</table>

## 🛠️ 技术栈

| 层级 | 技术 |
|---|---|
| 桌面框架 | Tauri 2 |
| 前端 | SvelteKit + Svelte 5 |
| 样式 | Tailwind CSS v4 |
| 后端 | Rust |
| 下载引擎 | aria2 (JSON-RPC) |

## 🚀 安装与开发

### 📦 环境要求

- Node.js >= 20
- pnpm >= 9
- Rust 稳定版
- Tauri 构建环境

### ▶️ 本地运行

```bash
pnpm install
pnpm tauri:dev
```

### ✅ 常用检查

```bash
pnpm check
cargo check
```

## ⬇️ 发布与下载

- 发布包请查看 GitHub Releases：<https://github.com/beyoug/mua/releases>
- 如需扩展更多平台的 aria2 sidecar，请参考 [贡献指南](./CONTRIBUTING.md)

## 🤝 贡献指南

欢迎提交 Issue 和 PR。开始前请先阅读：`CONTRIBUTING.md`

建议提交内容包括：

- 复现路径明确的问题报告
- 稳定性改进（同步、错误处理、状态一致性）
- UI/交互优化（确保浅色/深色和三套主题一致）

## 🗺️ 路线图

- [x] 基础下载任务管理
- [x] 任务同步与本地持久化
- [x] Torrent 配置流程
- [ ] 浏览器扩展联动
- [ ] 国际化（i18n）

## 🙏 致谢

- [Tauri](https://tauri.app/) - 跨平台桌面应用框架
- [SvelteKit](https://kit.svelte.dev/) - 现代 Web 框架
- [aria2](https://aria2.github.io/) - 下载引擎
- [Antigravity](https://antigravity.google/) - AI 协作开发伙伴
- [Claude Code](https://www.anthropic.com/claude-code) - AI 辅助开发工具
- [OpenAI](https://openai.com/) - AI 能力支持

## 📄 许可证

本项目基于 [MIT](./LICENSE) 开源。
