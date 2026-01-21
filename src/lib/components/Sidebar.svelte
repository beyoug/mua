<!--
  Sidebar.svelte
  侧边栏导航组件 - 包含 Logo、导航、统计面板
-->
<script lang="ts">
	import { Download, CheckCircle, History, Settings, TrendingDown, Plus } from '@lucide/svelte';

	type NavItem = 'active' | 'completed' | 'history';

	interface Props {
		activeNav?: NavItem;
		onNavChange?: (nav: NavItem) => void;
		onSettingsClick?: () => void;
		onAddClick?: () => void;
		stats?: {
			totalSpeed: string;
			activeCount: number;
			completedCount: number;
		};
	}

	let {
		activeNav = 'active',
		onNavChange,
		onSettingsClick,
		onAddClick,
		stats = { totalSpeed: '0 B/s', activeCount: 0, completedCount: 0 }
	}: Props = $props();

	const navItems = [
		{ id: 'active' as NavItem, icon: Download, label: '进行中' },
		{ id: 'completed' as NavItem, icon: CheckCircle, label: '已完成' },
		{ id: 'history' as NavItem, icon: History, label: '历史记录' }
	];
</script>

<aside class="sidebar">
	<!-- Logo 区域 -->
	<div class="logo-section">
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
		<div class="stat-item">
			<TrendingDown size={16} />
			<span class="stat-value">{stats.totalSpeed}</span>
		</div>
		<div class="stat-row">
			<span class="stat-label">活跃</span>
			<span class="stat-count">{stats.activeCount}</span>
		</div>
		<div class="stat-row">
			<span class="stat-label">已完成</span>
			<span class="stat-count">{stats.completedCount}</span>
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
		width: 200px;
		height: calc(100vh - 24px);
		background: var(--bg-sidebar);
		backdrop-filter: blur(24px) saturate(180%);
		-webkit-backdrop-filter: blur(24px) saturate(180%);
		border: 1px solid var(--border-color);
		border-radius: 20px;
		box-shadow: 
			0 8px 32px rgba(0, 0, 0, 0.12),
			0 1px 2px rgba(255, 255, 255, 0.1) inset;
		display: flex;
		flex-direction: column;
		padding: 20px 0;
		position: fixed;
		left: 12px;
		top: 12px;
		z-index: 5;
	}

	.logo-section {
		padding: 8px 20px 24px;
		-webkit-app-region: drag;
	}

	.logo {
		display: flex;
		align-items: center;
		gap: 10px;
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
		font-size: 20px;
		font-weight: 600;
		color: var(--text-primary);
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
		background: var(--border-light);
		border-color: var(--accent-primary);
		color: var(--accent-text);
	}

	.logo-text {
		font-size: 20px;
		font-weight: 600;
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
		gap: 12px;
		padding: 10px 12px;
		background: transparent;
		border: none;
		border-radius: 10px;
		color: var(--text-muted);
		font-size: 14px;
		cursor: pointer;
		transition: all 0.2s ease;
		text-align: left;
	}

	.nav-item:hover {
		background: var(--border-light);
		color: var(--text-primary);
	}

	.nav-item.active {
		background: var(--accent-active-bg);
		color: var(--accent-text);
	}

	.divider {
		height: 1px;
		background: var(--border-color);
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
		color: var(--accent-text);
		font-size: 14px;
		font-weight: 500;
		margin-bottom: 8px;
	}

	.stat-row {
		display: flex;
		justify-content: space-between;
		align-items: center;
		font-size: 13px;
	}

	.stat-label {
		color: var(--text-muted);
	}

	.stat-count {
		color: var(--text-secondary);
		font-weight: 500;
	}

	.sidebar-footer {
		padding: 16px 12px 0;
		border-top: 1px solid var(--border-color);
		margin-top: 16px;
	}

	.settings-btn {
		display: flex;
		align-items: center;
		gap: 12px;
		width: 100%;
		padding: 10px 12px;
		background: transparent;
		border: none;
		border-radius: 10px;
		color: var(--text-muted);
		font-size: 14px;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.settings-btn:hover {
		background: var(--border-light);
		color: var(--text-primary);
	}
</style>
