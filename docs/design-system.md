# Mua 设计系统规范（Design System）

> 适用范围：当前默认主题 `theme-default`。  
> 目标：新增/调整页面时，视觉语言与首页、设置面板、任务弹窗保持一致，避免“局部好看、整体割裂”。

---

## 1. 当前风格基调（2026-03）

### 1.1 核心定位

1. **Glass Base（玻璃基底）**：容器通过轻透明 + 内高光表现层级，不依赖粗边框。
2. **Dark Minimal（暗色克制）**：暗色模式优先降低 glow 强度，避免“块太多/光太重”。
3. **Subtle Tech（轻科技）**：科技感主要来自：
   - 轻渐变背景
   - 细信号条（active rail）
   - 低强度 ring / 阴影
4. **Single Interaction Language（统一交互语言）**：hover / active / focus / disabled 反馈强度一致。

### 1.2 当前主张

- **不追求强霓虹**，追求“看久不累”的专业工具感。
- **选中态优先用背景+信号条**，尽量避免明显外边框。
- **按钮主次分明**：主按钮可稍亮，次按钮保持透明/虚线语义。

---

## 2. Token 与语义约束

### 2.1 语义色

- Accent：`--accent-primary` / `--accent-secondary`
- 文本：`--text-primary` / `--text-secondary` / `--text-muted` / `--text-tertiary`
- 状态：`--semantic-success` / `--semantic-warning` / `--semantic-danger`

### 2.2 必须遵守

- 统一使用 `semantic-danger`，不再新增 `semantic-error` 别名。
- 不在业务组件硬编码品牌色（如 `#3b82f6`）。
- 不新增重复 token（先复用 `theme-base.css`）。
- 字体规范：
  - 正文统一 `var(--font-base)`
  - 等宽内容统一 `var(--font-mono)`
  - 不在组件内直接写 `'JetBrains Mono'` 等具体字体名

---

## 3. 布局与密度基线

### 3.1 间距节奏

- 微间距：`4 / 6 / 8`
- 常规：`10 / 12 / 14 / 16`
- 区块：`20 / 24 / 32`

> 避免“孤立微调值”长期存在（如只为视觉补丁添加 `padding-top: 2px`）。

### 3.2 圆角层级

- 控件：`8 ~ 11`
- 组块：`12`
- 主容器：`14 ~ 18`

### 3.3 命中区（Hit Area）

- icon 按钮：统一 `32x32`
- 弹窗 footer 主次按钮：高度一致（当前基线 `36px`）

---

## 4. 字体层级（已收口版本）

- 面板标题：`15~16px / 600`
- 设置项名称：`13px / 500~600`
- 描述正文：`12px / 400~500`
- hint / warning / error 文案：`11px`
- badge / pill：`10~11px / 600`

辅助文本行高建议：`1.3 ~ 1.45`。

---

## 5. 控件系统（优先复用）

来源：`src/styles/theme-base.css`

- 输入：`ui-field`
- 组合输入：`ui-input-group`
- 主按钮：`ui-btn-primary`
- 次按钮：`ui-btn-secondary`
- 图标按钮：`ui-btn-icon`
- 小操作：`ui-mini-action`
- 徽标：`ui-badge`

### 5.1 状态反馈强度（当前标准）

- **Hover**：背景提升为主，发光为辅（弱）
- **Active**：不做大位移，避免“跳动感”
- **Focus**：统一 `--focus-ring`
- **Disabled**：降低可见度并去掉位移/滤镜

补充（复用优先）：

- Hover ring 统一复用：
  - 轻：`--hover-ring-soft`
  - 中：`--hover-ring-medium`
- 禁止在多个组件重复写同构 ring 表达式（例如重复的 `0 0 0 2px color-mix(...)`）

---

## 6. 页面模板规范（按当前实现）

### 6.1 首页侧栏（Sidebar）

当前规范：

- Add / Settings 按钮：透明 + 虚线边框语义
- 导航 active：
  - 背景轻渐变
  - 左侧信号条（贴左，带圆弧端）
  - 不使用明显外框边线
- 暗色模式：
  - 减少 glow
  - 简化块状叠层

### 6.2 设置面板（SettingsPanel + 子页）

- section 与 setting-list 使用统一玻璃分层
- 每个 setting-item 的密度与间距统一
- 说明文案风格统一（字号/行高/透明度）
- 侧栏导航与首页保持相同 active 语言，但强度可稍弱

### 6.3 添加任务弹窗（Add Task）

- 头部、内容、footer 横向节奏一致
- Basic 与 Advanced 都使用统一分组卡片语言
- Footer 主次按钮同宽同高，语义清晰
- 次按钮（高级设置）与首页按钮语义一致（透明 + 虚线）

### 6.4 详情/日志/BT 等复杂面板

- 局部强调允许使用 inset ring
- 谨慎使用外发光（优先 dark minimal）
- 优先“信息清晰”而非“效果炫技”

---

## 7. 暗色模式专用约束（新增）

1. glow 强度比浅色低一档（建议降低 20%~40%）。
2. 避免多个独立“重背景块”同时出现。
3. 选中态优先靠：背景差 + 信号条 + 字重，不靠亮边框。
4. 文本对比优先保证可读性，再考虑装饰光效。

---

## 8. 视觉验收清单（PR 前自检）

- [ ] 是否复用了 `ui-*` 基类，而非重复造控件？
- [ ] section / item / footer 间距是否在统一节奏内？
- [ ] icon 按钮是否统一为 `32x32`？
- [ ] hint / error / info 文案是否都在 11px 体系？
- [ ] 暗色模式是否出现“多块叠层 + 过亮 glow”？
- [ ] active 状态是否避免明显边框圈？
- [ ] 是否避免硬编码颜色值？
- [ ] `pnpm check` 是否通过？

---

## 9. 维护流程建议

1. 先改 token / 基类，再改局部组件。
2. 一次只调整一个视觉目标（例如“降低 glow”），避免多目标叠加难回归。
3. 每次风格调整后同步更新本文件，防止规范滞后。
4. 清理原则：
   - 能用变量就不用硬编码
   - 能复用基类就不新增局部同构样式
   - 能靠选择器优先级解决就避免 `!important`
