# Mua 开发规范（Service-First）

> 目标：统一实现方式、目录结构、命名与技术细节，降低维护成本，避免“临时方案长期化”。

## 1. 核心原则

1. **Service-First**：业务逻辑在 `src/lib/services/*`，UI 只做展示与交互编排。
2. **单向依赖**：`components/routes -> services -> api -> tauri command`。
3. **类型集中**：领域类型放 `src/lib/types/*`，不要放在 transport 层（`api/*`）。
4. **边界清晰**：组件/页面不得直接调用 `invoke`，不得直接依赖 transport。
5. **最小改动**：Bugfix 优先最小修复，避免“修 bug 顺便重构”。

---

## 2. 目录与分层规范

## 2.1 前端目录约定

```text
src/lib/
  api/               # 纯 transport 封装（tauri invoke）
  services/          # 领域服务（业务规则、状态、编排）
    download/
    settings/
    theme/
    aria2/
    aria2Config/
    torrent/
  components/        # 纯 UI
  types/             # 领域类型
  utils/             # 纯工具函数
```

## 2.2 后端目录约定（Tauri/Rust）

```text
src-tauri/src/
  core/
    commands/        # tauri command 入口（薄）
    events.rs        # 事件常量
    error.rs         # 统一错误类型
    sync.rs          # 后台同步
    ...
  aria2/             # aria2 进程与 rpc 客户端
  ui/                # tray/UI 相关
```

---

## 3. 命名规范

1. 文件名：`kebab-case`（Svelte 组件除外）。
2. Svelte 组件：`PascalCase.svelte`。
3. Service 方法：**动作语义**（如 `pauseTask`、`resumeAll`），避免 `handleXxx` 业务命名泄漏到领域层。
4. 事件名常量：`EVENT_*`，统一在 `src/lib/api/events.ts` 与 `src-tauri/src/core/events.rs`。
5. Command 名保持 snake_case，与 tauri handler 一致。

---

## 4. API/Service 边界强约束

## 4.1 禁止事项

- 组件/路由禁止：`import { invoke } from '@tauri-apps/api/core'`
- 组件/路由禁止直接从 `src/lib/api/*` 导入 command 包装
- 禁止在 `api/*` 中定义领域类型（DTO 类型放 `types/*`）

## 4.2 允许事项

- `services/*` 调用 `api/*`
- `components/routes` 只调用 `services/*`
- `utils/*` 保持纯函数、无副作用

---

## 5. 状态管理规范

1. 领域状态由 service 维护（如 download service）。
2. 若使用 Svelte store，作为 service 内部实现，不作为 UI 直接“业务入口”。
3. 导出只暴露 `index.ts` 公共面；避免外部 import `service.ts` 深路径。

---

## 6. 事件与同步规范

1. 事件名必须来自常量，不允许裸字符串。
2. 同步事件优先结构化 payload（如 `snapshot/delta`）。
3. 对增量事件必须有失序/版本回退策略（如 revision/seq 校验 + full resync）。

---

## 7. 错误处理规范

## 7.1 前端

- 禁止空 catch（`catch (_) {}`）
- 必须记录日志（`createLogger`）或显式注释忽略原因

## 7.2 后端

- 统一使用 `AppResult<T>` / `AppError`
- 命令返回形状保持域内一致（推荐操作类命令返回 `AppResult<()>`）
- 错误识别逻辑应收敛到 `error.rs`（避免命令层重复 string contains）

---

## 8. 生命周期与资源清理

1. 所有 `addEventListener/listen/setInterval` 必须有对应 cleanup。
2. 推荐 `init() -> disposer()` 模式。
3. 全局监听（document/window/mediaQuery）必须明确销毁时机。

---

## 9. 代码风格与质量门禁

1. 每次改动后必须通过：

```bash
pnpm check
cargo check
```

2. 禁止：
- `as any`
- `@ts-ignore`
- `@ts-expect-error`

3. 新增模块时必须包含 `index.ts` 作为公共导出面。

---

## 10. PR / 提交约束

1. 提交应聚焦单一意图（原子提交）。
2. PR 描述至少包含：
- 背景/目的
- 影响范围
- 验证结果（`pnpm check`、`cargo check`）
3. 变更涉及协议（event payload/command return）需注明兼容策略。

---

## 11. 推荐实施顺序（新功能）

1. 先定义类型（`types/*`）
2. 再定义 transport（`api/*`）
3. 再实现 service（`services/*`）
4. 最后接入 UI（`components/routes`）

---

## 12. 快速自检清单

- [ ] 组件是否直接调用了 api/transport？（应为否）
- [ ] 事件名是否用了常量？
- [ ] listener/interval 是否有 cleanup？
- [ ] 类型是否在 `types/*` 而不是 `api/*`？
- [ ] `pnpm check` 和 `cargo check` 是否通过？
