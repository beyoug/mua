<!--
  HeaderToolbar.svelte
  标题栏工具栏 - 水平布局
-->
<script lang="ts">
	import { Download, CheckCircle, Pause, Settings, Plus } from '@lucide/svelte';

	type Tab = 'all' | 'active' | 'completed' | 'paused';
	
	interface Props {
		activeTab?: Tab;
		onTabChange?: (tab: Tab) => void;
		onAddClick?: () => void;
	}

	let { 
		activeTab = 'all',
		onTabChange,
		onAddClick
	}: Props = $props();

	const tabs = [
		{ id: 'all' as Tab, icon: Download, label: '全部' },
		{ id: 'active' as Tab, icon: Download, label: '进行中' },
		{ id: 'paused' as Tab, icon: Pause, label: '已暂停' },
		{ id: 'completed' as Tab, icon: CheckCircle, label: '已完成' },
	];

	function handleTabClick(tab: Tab) {
		onTabChange?.(tab);
	}
</script>

<div class="header-toolbar">
	<!-- 添加按钮 -->
	<button 
		class="toolbar-btn add-btn"
		onclick={() => onAddClick?.()}
		title="添加下载"
	>
		<Plus size={18} strokeWidth={2} />
	</button>

	<!-- 分隔线 -->
	<div class="divider"></div>

	<!-- 导航标签 -->
	<nav class="nav-tabs">
		{#each tabs as tab}
			<button
				class="toolbar-btn"
				class:active={activeTab === tab.id}
				onclick={() => handleTabClick(tab.id)}
			>
				<tab.icon size={14} strokeWidth={activeTab === tab.id ? 2.5 : 2} />
				<span>{tab.label}</span>
			</button>
		{/each}
	</nav>

	<!-- 分隔线 -->
	<div class="divider"></div>

	<!-- 设置按钮 -->
	<button class="toolbar-btn">
		<Settings size={14} strokeWidth={2} />
		<span>设置</span>
	</button>
</div>

<style>
	.header-toolbar {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 6px;
		
		background: rgba(255, 255, 255, 0.06);
		border: 1px solid rgba(255, 255, 255, 0.08);
		border-radius: 12px;
		backdrop-filter: blur(20px);
		-webkit-backdrop-filter: blur(20px);
	}

	.toolbar-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 12px;
		
		background: transparent;
		border: none;
		border-radius: 8px;
		color: rgba(255, 255, 255, 0.6);
		font-size: 13px;
		cursor: pointer;
		transition: all 0.2s ease;
		white-space: nowrap;
	}

	.toolbar-btn:hover {
		background: rgba(255, 255, 255, 0.1);
		color: rgba(255, 255, 255, 0.9);
	}

	.toolbar-btn.active {
		background: rgba(16, 185, 129, 0.15);
		color: rgb(52, 211, 153);
	}

	.add-btn {
		background: linear-gradient(135deg, rgb(16, 185, 129), rgb(20, 184, 166));
		color: white;
	}

	.add-btn:hover {
		background: linear-gradient(135deg, rgb(5, 150, 105), rgb(13, 148, 136));
		color: white;
	}

	.divider {
		width: 1px;
		height: 20px;
		background: rgba(255, 255, 255, 0.1);
		margin: 0 4px;
	}

	.nav-tabs {
		display: flex;
		gap: 2px;
	}
</style>

