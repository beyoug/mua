<!--
  Sidebar.svelte
  侧边栏导航组件 - 包含 Logo、导航、统计面板
-->
<script lang="ts">
	import { Download, CheckCircle, History, Settings, TrendingDown, Plus } from '@lucide/svelte';

	import { getCurrentWindow } from '@tauri-apps/api/window';

	type NavItem = 'active' | 'complete' | 'history';

	interface Props {
		activeNav?: NavItem;
		onNavChange?: (nav: NavItem) => void;
		onSettingsClick?: () => void;
		onAddClick?: () => void;
		stats?: {
			totalSpeed: string;
			activeCount: number;
			completeCount: number;
		};
	}

	let {
		activeNav = 'active',
		onNavChange,
		onSettingsClick,
		onAddClick,
		stats = { totalSpeed: '0 B/s', activeCount: 0, completeCount: 0 }
	}: Props = $props();

    function startDrag() {
        getCurrentWindow().startDragging();
    }


	const navItems = [
		{ id: 'active' as NavItem, icon: Download, label: '进行中' },
		{ id: 'complete' as NavItem, icon: CheckCircle, label: '已完成' },
		{ id: 'history' as NavItem, icon: History, label: '历史记录' }
	];
</script>

<aside class="sidebar">
	<!-- Logo 区域 -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="logo-section" onmousedown={startDrag}>
		<div class="logo">
			<span class="logo-icon">↓</span>
			<span class="logo-text">Mua</span>
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
				<item.icon size={18} strokeWidth={activeNav === item.id ? 2.5 : 2} />
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
            <span class="stat-value">{stats.totalSpeed?.replace('|', ' ') || '0 B/s'}</span>
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
		background: var(--glass-bg);
		backdrop-filter: var(--glass-blur) var(--glass-saturate);
		-webkit-backdrop-filter: var(--glass-blur) var(--glass-saturate);
		border: 1px solid var(--glass-border);
		border-radius: 16px;
		box-shadow: var(--glass-shadow);
		display: flex;
		flex-direction: column;
		/* 顶部 padding: 红绿灯空间(约28px) + 原有间距 */
		padding: 0 0 16px; 
		position: fixed;
		left: 12px;
		top: 12px;
		z-index: 5;
	}

	.logo-section {
		/* 红绿灯区域主要由 Sidebar 顶部空间承载，这里保留适当内边距 */
		padding: 46px 20px 24px;
		-webkit-app-region: drag;
		position: relative; /* 确保 overlay 绝对定位相对于此 */
	}

	.logo {
		display: flex;
		align-items: center;
		gap: 10px;
		position: relative;
		pointer-events: none; /* 让鼠标事件穿透 */
		z-index: 0;
	}

	.logo-icon {
		width: 32px;
		height: 32px;
		background: linear-gradient(135deg, var(--accent-primary), var(--accent-secondary));
		border-radius: 8px;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 18px;
		font-weight: bold;
		color: white;
	}

	.logo-text {
		font-size: 18px;
		font-weight: 600;
		color: var(--text-primary);
		letter-spacing: -0.02em;
	}

	.add-section {
		padding: 0 12px 12px;
	}

	.add-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		width: 100%;
		padding: 10px 12px;
		background: transparent;
		border: 1px dashed var(--border-color);
		border-radius: 10px;
		color: var(--text-muted);
		font-size: 14px;
		font-weight: 400;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.add-btn:hover {
		background: var(--surface-hover);
		border-color: var(--border-strong);
		color: var(--text-primary);
	}

	.nav-menu {
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding: 0 12px;
		flex: 1;
	}

	.nav-item {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 9px 12px;
		background: transparent;
		border: none;
		border-radius: 10px;
		border-left: 2px solid transparent;
		color: var(--text-secondary);
		font-size: 13px;
		font-weight: 400;
		cursor: pointer;
		transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
		text-align: left;
	}

	.nav-item:hover {
		background: var(--surface-hover);
		color: var(--text-primary);
		border-left-color: var(--border-strong);
	}

	.nav-item.active {
		background: var(--surface-active);
		color: var(--accent-primary); /* 使用主题色，深浅模式自适应 */
		font-weight: 500;
		border-left-color: var(--accent-primary); /* 活跃状态使用主题色左边框 */
	}

	.divider {
		height: 1px;
		background: var(--border-normal); /* 简约细线分割 */
		margin: 16px 20px;
	}

	.stats-panel {
		padding: 0 20px;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.stat-item {
		display: flex;
		align-items: center;
		gap: 8px;
		color: var(--text-primary);
		font-size: 13px;
		font-weight: 500;
		margin-bottom: 6px;
	}

    .stat-value {
        font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
        font-variant-numeric: tabular-nums;
        font-weight: 600;
        color: var(--text-secondary);
        font-size: 11px;
    }

	.stat-row {
		display: flex;
		justify-content: space-between;
		align-items: center;
		font-size: 12px;
		font-weight: 400;
	}

	.stat-label {
		color: var(--text-muted);
	}

	.stat-count {
		color: var(--accent-text);
		opacity: 0.7; /* 弱化显示，不过分强调 */
		font-weight: 500;
	}

	.sidebar-footer {
		padding: 16px 12px 0;
		border-top: 1px solid var(--border-normal); /* 简约顶部分割线 */
		margin-top: 16px;
	}

	.settings-btn {
		display: flex;
		align-items: center;
		gap: 10px;
		width: 100%;
		padding: 9px 12px;
		background: transparent;
		border: none;
		border-radius: 10px;
		color: var(--text-secondary);
		font-size: 13px;
		font-weight: 400;
		cursor: pointer;
		transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
	}

	.settings-btn:hover {
		background: var(--surface-hover);
		color: var(--text-primary);
	}
</style>
