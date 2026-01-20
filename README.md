# Mua

<p align="center">
  <img src="src-tauri/icons/128x128.png" alt="Mua Logo" width="128" height="128">
</p>

<p align="center">
  <strong>现代化的 Aria2 下载管理器</strong>
</p>

<p align="center">
  基于 Tauri 2.0 + SvelteKit + Svelte 5 构建的跨平台桌面应用
</p>

---

## ✨ 特性

- 🚀 **高性能下载** - 基于 aria2c 多线程下载引擎
- 🎨 **现代化 UI** - shadcn-svelte 组件库 + Tailwind CSS v4
- 🔄 **实时状态** - TanStack Query 驱动的响应式数据流
- 📦 **轻量打包** - Tauri 2.0 打包体积远小于 Electron
- 💻 **跨平台** - 支持 macOS、Windows、Linux

## 🛠️ 技术栈

| 层级 | 技术 | 版本 |
|------|------|------|
| 构建框架 | Tauri | 2.0 |
| 前端框架 | SvelteKit (SPA) | Svelte 5 |
| 状态管理 | TanStack Query | v6 |
| UI 组件 | shadcn-svelte | latest |
| 样式 | Tailwind CSS | v4 |
| 后端 | Rust | latest |
| 下载引擎 | aria2c | JSON-RPC 2.0 |

## 📦 快速开始

### 环境要求

- **Node.js** >= 20
- **pnpm** >= 9
- **Rust** >= 1.77

### 安装依赖

```bash
pnpm install
```

### 开发模式

```bash
pnpm tauri dev
```

### 构建生产版本

```bash
pnpm tauri build
```

## 📁 项目结构

```
Mua/
├── src/                      # 前端源码
│   ├── routes/               # SvelteKit 路由
│   │   ├── +layout.svelte    # 根布局 (TanStack Query Provider)
│   │   ├── +layout.ts        # SPA 模式配置
│   │   └── +page.svelte      # 首页
│   └── lib/
│       ├── components/ui/    # shadcn-svelte 组件
│       └── utils.ts          # 工具函数
├── src-tauri/                # Rust 后端
│   ├── src/lib.rs            # Tauri 入口
│   ├── tauri.conf.json       # Tauri 配置
│   └── Cargo.toml            # Rust 依赖
├── build/                    # 前端构建输出
├── components.json           # shadcn-svelte 配置
└── svelte.config.js          # SvelteKit 配置
```

## 🎨 添加 UI 组件

使用 shadcn-svelte CLI 添加组件：

```bash
pnpm dlx shadcn-svelte@latest add button
pnpm dlx shadcn-svelte@latest add card
pnpm dlx shadcn-svelte@latest add progress
```

## 📄 许可证

MIT License

## 🙏 致谢

- [Tauri](https://tauri.app/) - 跨平台桌面应用框架
- [SvelteKit](https://kit.svelte.dev/) - 现代 Web 框架
- [shadcn-svelte](https://www.shadcn-svelte.com/) - 精美的 UI 组件库
- [aria2](https://aria2.github.io/) - 强大的下载引擎
- [Motrix](https://motrix.app/) - 设计灵感来源
