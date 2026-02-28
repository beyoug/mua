<!--
  Sidebar.svelte
  侧边栏导航组件 - 包含 Logo、导航、统计面板
-->
<script lang="ts">
	import {
		Download,
		CheckCircle,
		History,
		Settings,
		TrendingDown,
		Plus,
	} from "@lucide/svelte";

	import { getCurrentWindow } from "@tauri-apps/api/window";

	type NavItem = "active" | "complete" | "history";

	interface Props {
		activeNav?: NavItem;
		onNavChange?: (nav: NavItem) => void;
		onSettingsClick?: () => void;
		onAddClick?: () => void;
		stats?: {
			totalSpeed: { value: string; unit: string };
			activeCount: number;
			completeCount: number;
		};
		blurred?: boolean;
	}

	let {
		activeNav = "active",
		onNavChange,
		onSettingsClick,
		onAddClick,
		stats = {
			totalSpeed: { value: "0", unit: "B/s" },
			activeCount: 0,
			completeCount: 0,
		},
		blurred = false,
	}: Props = $props();

	function startDrag() {
		getCurrentWindow().startDragging();
	}

	const navItems = [
		{ id: "active" as NavItem, icon: Download, label: "进行中" },
		{ id: "complete" as NavItem, icon: CheckCircle, label: "已完成" },
		{ id: "history" as NavItem, icon: History, label: "历史记录" },
	];
</script>

<aside class="sidebar" class:blurred>
	<!-- Logo 区域 -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="logo-section" onmousedown={startDrag}>
		<div class="logo">
			<img src="/logo.png" alt="Mua" class="logo-icon-img" />
			<div class="logo-text-group">
				<span class="logo-text">Mua</span>
				<span class="logo-slogan">Simple & Fast</span>
			</div>
		</div>
	</div>

	<!-- 添加任务按钮 -->
	<div class="add-section">
		<button class="add-btn" onclick={() => onAddClick?.()}>
			<Plus size={18} strokeWidth={2.5} />
			<span>添加任务</span>
		</button>
	</div>

	<!-- 导航菜单 -->
	<nav class="nav-menu">
		{#each navItems as item}
			<button
				class="nav-item"
				class:active={activeNav === item.id}
				onclick={() => onNavChange?.(item.id)}
			>
				<item.icon
					size={18}
					strokeWidth={activeNav === item.id ? 2.5 : 2}
				/>
				<span>{item.label}</span>
			</button>
		{/each}
	</nav>

	<!-- 分隔线 -->
	<div class="divider"></div>

	<!-- 统计面板 -->
	<div class="stats-panel">
		<div class="stat-item" title="总下载速度">
			<TrendingDown size={16} />
			<span class="stat-value"
				>{stats.totalSpeed?.value ?? "0"}
				{stats.totalSpeed?.unit ?? "B/s"}</span
			>
		</div>
		<div class="stat-row">
			<span class="stat-label">活跃</span>
			<span class="stat-count">{stats.activeCount}</span>
		</div>
		<div class="stat-row">
			<span class="stat-label">已完成</span>
			<span class="stat-count">{stats.completeCount}</span>
		</div>
	</div>

	<!-- 底部设置 -->
	<div class="sidebar-footer">
		<button class="settings-btn" onclick={() => onSettingsClick?.()}>
			<Settings size={18} />
			<span>设置</span>
		</button>
	</div>
</aside>

<style>
	.sidebar {
		width: var(--sidebar-width);
		height: calc(100vh - var(--layout-gap) * 2);
		background:
			linear-gradient(
				168deg,
				color-mix(in srgb, var(--glass-bg) 90%, var(--accent-primary) 10%),
				color-mix(in srgb, var(--glass-bg) 96%, transparent)
			),
			var(--glass-bg);
		backdrop-filter: var(--glass-blur) var(--glass-saturate);
		-webkit-backdrop-filter: var(--glass-blur) var(--glass-saturate);
		border: none;
		border-radius: 16px;
		box-shadow:
			var(--glass-shadow),
			inset 0 1px 0 color-mix(in srgb, #ffffff 12%, transparent);
		display: flex;
		flex-direction: column;
		/* 顶部 padding: 红绿灯空间(约28px) + 原有间距 */
		padding: 0 0 16px;
		position: fixed;
		left: 12px;
		top: 12px;
		z-index: 5;
		transition: filter 0.3s ease;
		overflow: hidden;
	}

	.sidebar::before {
		content: "";
		position: absolute;
		inset: 0;
		pointer-events: none;
		background:
			radial-gradient(
				120% 50% at 12% -8%,
				color-mix(in srgb, var(--accent-primary) 16%, transparent),
				transparent 68%
			),
			linear-gradient(
				180deg,
				color-mix(in srgb, var(--border-subtle) 42%, transparent),
				transparent 34%
			);
		opacity: 0.34;
	}

	.sidebar.blurred {
		filter: blur(2px);
	}

	.logo-section {
		padding: 42px 16px 18px;
		-webkit-app-region: drag;
		position: relative; /* 确保 overlay 绝对定位相对于此 */
		margin: 4px 10px 0;
		border-radius: 12px;
		background: transparent;
	}

	.logo {
		display: flex;
		align-items: center;
		gap: 10px;
		position: relative;
		pointer-events: none; /* 让鼠标事件穿透 */
		z-index: 0;
	}

	.logo-icon-img {
		width: 48px;
		height: 48px;
		border-radius: 8px;
		object-fit: contain;
		filter: drop-shadow(0 6px 12px color-mix(in srgb, var(--accent-glow) 20%, transparent));
	}

	.logo-text-group {
		display: flex;
		flex-direction: column;
		justify-content: center;
		height: 48px; /* Match close to icon height for alignment */
	}

	.logo-text {
		font-size: 22px;
		font-weight: 700;
		color: var(--text-primary);
		letter-spacing: -0.018em;
		line-height: 1;
		margin-bottom: 2px;
	}

	.logo-slogan {
		font-size: 10px;
		font-weight: 500;
		color: var(--text-muted);
		letter-spacing: 0.04em;
		text-transform: uppercase;
	}

	.add-section {
		padding: 8px 10px 10px;
	}

	.add-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		width: 100%;
		min-height: 38px;
		padding: 8px 12px;
		background: transparent;
		border: 1px dashed color-mix(in srgb, var(--accent-primary) 26%, transparent);
		border-radius: 11px;
		color: var(--text-secondary);
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition:
			background 0.24s cubic-bezier(0.22, 1, 0.36, 1),
			border-color 0.24s cubic-bezier(0.22, 1, 0.36, 1),
			color 0.24s ease,
			box-shadow 0.24s ease,
			transform 0.24s cubic-bezier(0.22, 1, 0.36, 1);
		box-shadow: none;
	}

	.add-btn:hover {
		background:
			linear-gradient(
				122deg,
				color-mix(in srgb, var(--accent-primary) 14%, transparent),
				color-mix(in srgb, var(--accent-primary) 6%, transparent)
			),
			transparent;
		border-color: color-mix(in srgb, var(--accent-primary) 38%, transparent);
		color: var(--text-primary);
		transform: translateY(-0.5px);
		box-shadow:
			var(--hover-ring-medium),
			0 10px 18px -14px color-mix(in srgb, var(--accent-glow) 56%, transparent);
	}

	.add-btn:focus-visible,
	.settings-btn:focus-visible,
	.nav-item:focus-visible {
		outline: none;
		box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent-primary) 20%, transparent);
	}

	.nav-menu {
		display: flex;
		flex-direction: column;
		gap: 6px;
		padding: 0 10px;
		flex: 1;
		position: relative;
		margin-top: 2px;
	}

	.nav-menu::before {
		content: "";
		position: absolute;
		inset: 0;
		border-radius: 12px;
		background: transparent;
		pointer-events: none;
	}

	.nav-item {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 10px 12px;
		background: transparent;
		border: none;
		border-radius: 11px;
		color: var(--text-secondary);
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition:
			background 0.2s cubic-bezier(0.22, 1, 0.36, 1),
			color 0.2s ease,
			box-shadow 0.24s ease,
			transform 0.2s cubic-bezier(0.22, 1, 0.36, 1);
		text-align: left;
		position: relative;
		z-index: 1;
		overflow: hidden;
	}

	.nav-item::before {
		content: "";
		position: absolute;
		left: 0;
		top: 3px;
		bottom: 3px;
		width: 3px;
		border-radius: 999px;
		background: linear-gradient(
			180deg,
			color-mix(in srgb, var(--accent-secondary) 84%, white),
			color-mix(in srgb, var(--accent-primary) 84%, transparent)
		);
		opacity: 0;
		transform: scaleY(0.45);
		transition:
			opacity 0.2s ease,
			transform 0.2s cubic-bezier(0.22, 1, 0.36, 1);
	}

	.nav-item :global(svg) {
		opacity: 0.8;
		transition: transform 0.18s ease, opacity 0.18s ease;
	}

	.nav-item:hover {
		background: color-mix(in srgb, var(--accent-primary) 8%, transparent);
		color: var(--text-primary);
		transform: none;
		box-shadow:
			inset 0 1px 0 color-mix(in srgb, #ffffff 7%, transparent);
	}

	.nav-item:hover :global(svg) {
		opacity: 1;
		transform: none;
	}

	.nav-item.active {
		background:
			linear-gradient(
				110deg,
				color-mix(in srgb, var(--accent-primary) 26%, transparent),
				color-mix(in srgb, var(--accent-primary) 10%, transparent)
			),
			transparent;
		color: var(--accent-text);
		font-weight: 600;
		box-shadow:
			inset 0 1px 0 color-mix(in srgb, #ffffff 8%, transparent),
			0 8px 14px -16px color-mix(in srgb, var(--accent-glow) 34%, transparent);
	}

	.nav-item.active::before {
		opacity: 1;
		transform: scaleY(1);
	}

	.nav-item.active :global(svg) {
		opacity: 1;
		transform: none;
	}

	:global(html.light.theme-default) .nav-item.active {
		background:
			linear-gradient(
				110deg,
				color-mix(in srgb, var(--accent-primary) 20%, white),
				color-mix(in srgb, var(--accent-primary) 10%, white)
			),
			white;
		color: var(--accent-active);
		box-shadow: inset 0 1px 0 color-mix(in srgb, white 72%, transparent);
	}

	.divider {
		height: 1px;
		background: color-mix(in srgb, var(--border-subtle) 52%, transparent);
		margin: 12px 20px 10px;
		position: relative;
	}

	.divider::after {
		content: "";
		position: absolute;
		left: 50%;
		top: 50%;
		width: 16px;
		height: 1px;
		transform: translate(-50%, -50%);
		background: color-mix(in srgb, var(--accent-primary) 24%, transparent);
		filter: blur(1px);
		opacity: 0.26;
	}

	.stats-panel {
		margin: 0 10px;
		padding: 10px 12px;
		background: color-mix(in srgb, var(--surface-hover) 68%, transparent);
		border: none;
		border-radius: 12px;
		display: flex;
		flex-direction: column;
		gap: 6px;
		box-shadow: inset 0 1px 0 color-mix(in srgb, #ffffff 8%, transparent);
	}

	.stat-item {
		display: flex;
		align-items: center;
		gap: 8px;
		color: color-mix(in srgb, var(--text-secondary) 90%, var(--accent-text) 10%);
		font-size: 12px;
		font-weight: 560;
		margin-bottom: 6px;
		letter-spacing: 0.02em;
	}

	.stat-value {
		font-family: var(--font-base);
		font-variant-numeric: tabular-nums;
		font-weight: 620;
		color: color-mix(in srgb, var(--accent-text) 64%, var(--text-primary));
		font-size: 11.5px;
		text-shadow: none;
	}

	.stat-row {
		display: flex;
		justify-content: space-between;
		align-items: center;
		font-size: 11px;
		font-weight: 400;
	}

	.stat-label {
		color: var(--text-muted);
	}

	.stat-count {
		color: color-mix(in srgb, var(--text-primary) 90%, var(--accent-text) 10%);
		opacity: 0.88;
		font-weight: 600;
		font-variant-numeric: tabular-nums;
		padding: 2px 8px;
		border-radius: 999px;
		background: color-mix(in srgb, var(--accent-primary) 10%, transparent);
		box-shadow: none;
	}

	.sidebar-footer {
		padding: 12px 10px 0;
		margin-top: 12px;
		position: relative;
	}

	.sidebar-footer::before {
		content: "";
		position: absolute;
		left: 10px;
		right: 10px;
		top: -6px;
		height: 1px;
		background: color-mix(in srgb, var(--border-subtle) 42%, transparent);
	}

	.settings-btn {
		display: flex;
		align-items: center;
		gap: 10px;
		width: 100%;
		min-height: 36px;
		padding: 8px 12px;
		background: transparent;
		border: 1px dashed color-mix(in srgb, var(--accent-primary) 24%, transparent);
		border-radius: 11px;
		color: var(--text-secondary);
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition:
			background 0.24s cubic-bezier(0.22, 1, 0.36, 1),
			border-color 0.24s cubic-bezier(0.22, 1, 0.36, 1),
			color 0.24s ease,
			box-shadow 0.24s ease,
			transform 0.24s cubic-bezier(0.22, 1, 0.36, 1);
		box-shadow: none;
	}

	.settings-btn:hover {
		background: color-mix(in srgb, var(--accent-primary) 8%, transparent);
		border-color: color-mix(in srgb, var(--accent-primary) 34%, transparent);
		color: var(--text-primary);
		transform: none;
		box-shadow:
			var(--hover-ring-soft);
	}
</style>
