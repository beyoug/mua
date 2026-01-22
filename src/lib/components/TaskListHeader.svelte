<!--
  TaskListHeader.svelte
  任务列表头部组件 - 显示标题、统计和全局操作按钮
-->
<script lang="ts">
	import { Play, Pause, Trash2 } from '@lucide/svelte';
	import type { DownloadTask } from '$lib/types/download';

	interface Props {
		title: string;
		taskCount: number;
		tasks: DownloadTask[];
		hasDownloading: boolean;
		hasPaused: boolean;
		hasRemovable: boolean;
		isSelectionMode: boolean;
		selectedCount: number;
		onGlobalPause?: () => void;
		onGlobalResume?: () => void;
		onTrashClick?: () => void;
	}

	let {
		title,
		taskCount,
		tasks,
		hasDownloading,
		hasPaused,
		hasRemovable,
		isSelectionMode,
		selectedCount,
		onGlobalPause,
		onGlobalResume,
		onTrashClick
	}: Props = $props();

	const trashTooltip = $derived(() => {
		if (isSelectionMode) {
			return selectedCount === 0 ? '全选本次显示的任务' : `删除选中 (${selectedCount})`;
		}
		return '批量管理';
	});
</script>

<header class="floating-header">
	<div class="header-left">
		<h1>{title}</h1>
		<span class="task-count">{taskCount} 个任务</span>
	</div>
	
	<div class="header-actions">
		{#if !isSelectionMode}
			{#if hasDownloading}
				<button class="icon-btn" onclick={onGlobalPause} title="全部暂停">
					<Pause size={18} fill="currentColor" />
				</button>
			{:else if hasPaused}
				<button class="icon-btn" onclick={onGlobalResume} title="全部开始">
					<Play size={18} fill="currentColor" />
				</button>
			{/if}
		{/if}

		{#if hasRemovable || tasks.length > 0}
			<button class="icon-btn danger" onclick={onTrashClick} title={trashTooltip()}>
				<Trash2 size={18} />
			</button>
		{/if}
	</div>
</header>

<style>
	/* 悬浮玻璃 Header */
	.floating-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px 24px;
		margin: 12px 0 24px;
		background: var(--glass-bg);
		backdrop-filter: var(--glass-blur) var(--glass-saturate);
		-webkit-backdrop-filter: var(--glass-blur) var(--glass-saturate);
		border: 1px solid var(--glass-border);
		border-radius: 20px;
		box-shadow: var(--glass-shadow);
		position: sticky;
		top: 12px;
		z-index: 10;
	}

	.header-left {
		display: flex;
		align-items: baseline;
		gap: 12px;
	}

	.header-actions {
		display: flex;
		gap: 8px;
	}

	h1 {
		font-size: 20px;
		font-weight: 600;
		color: var(--text-primary);
		margin: 0;
	}

	.task-count {
		font-size: 13px;
		color: var(--text-muted);
		font-weight: 500;
	}

	.icon-btn {
		width: 32px;
		height: 32px;
		display: flex;
		align-items: center;
		justify-content: center;
		background: transparent;
		border: 1px solid var(--border-color);
		border-radius: 8px;
		color: var(--text-muted);
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.icon-btn:hover {
		background: var(--border-light);
		color: var(--text-primary);
		border-color: var(--text-muted);
	}

	.icon-btn.danger:hover {
		background: rgba(220, 38, 38, 0.1);
		color: #ef4444;
		border-color: #fca5a5;
	}
</style>
