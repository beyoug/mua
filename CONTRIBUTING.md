# 🤝 贡献指南 (Contributing Guide)

感谢你参与 Mua 的建设。我们欢迎 bug 修复、稳定性优化、文档改进和体验增强类贡献。

## 🛠️ 开发环境

- Node.js >= 20
- pnpm >= 9
- Rust 稳定版
- Tauri 编译依赖（按平台安装）：<https://tauri.app/v2/guides/getting-started/prerequisites>

## ▶️ 本地启动

```bash
git clone https://github.com/beyoug/mua.git
cd mua
pnpm install
pnpm tauri:dev
```

## ✅ 提交前检查

```bash
pnpm check
cargo fmt --all -- --check
cargo check
```

如果你改动了 Rust 逻辑，建议额外执行：

```bash
cargo clippy --all-targets -- -D warnings
```

## 🧾 提交规范

推荐使用 Conventional Commits：

- `feat`: 新功能
- `fix`: 缺陷修复
- `refactor`: 重构（不改变外部行为）
- `docs`: 文档更新
- `style`: 样式/格式调整
- `chore`: 工具链或工程配置调整

示例：

```text
fix(store): prevent task flicker after delete
```

## 🤖 AI 协作说明

本项目允许使用 AI 工具辅助开发，但请遵循：

- 不直接合入未经理解的生成代码
- 提交前完成本地检查与必要验证
- 在 PR 描述中明确说明 AI 参与范围（如方案草拟、重构建议、测试用例草稿）

## 🌍 Sidecar 集成说明

Mua 使用 `aria2c` 作为 sidecar。请将对应平台二进制放在 `src-tauri/` 目录，并按以下文件名规则命名：

`aria2c-<target-triple>[.exe]`

| 平台 | Target Triple | 文件名 |
|---|---|---|
| macOS (Intel) | `x86_64-apple-darwin` | `aria2c-x86_64-apple-darwin` |
| macOS (Apple Silicon) | `aarch64-apple-darwin` | `aria2c-aarch64-apple-darwin` |
| Windows (x64) | `x86_64-pc-windows-msvc` | `aria2c-x86_64-pc-windows-msvc.exe` |
| Linux (x64) | `x86_64-unknown-linux-gnu` | `aria2c-x86_64-unknown-linux-gnu` |

获取方式：

1. 从 aria2 官方 Releases 下载：<https://github.com/aria2/aria2/releases>
2. 或自行编译 aria2：<https://github.com/aria2/aria2>

## 🚀 贡献流程

1. 新建分支：`git checkout -b feat/your-change`
2. 完成开发并通过检查
3. 提交并推送分支
4. 发起 Pull Request，描述变更背景、验证方式和影响范围

## 🐞 Issue / PR 建议内容

- 问题复现步骤（尽量最小化）
- 预期行为与实际行为
- 平台信息（OS、架构）
- 日志或截图（如有）

## 💬 交流

问题和建议请通过 GitHub Issues 提交：<https://github.com/beyoug/mua/issues>
