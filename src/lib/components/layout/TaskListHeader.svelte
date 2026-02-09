<!--
  TaskListHeader.svelte
  任务列表头部组件 - 显示标题、统计和全局操作按钮
-->
<script lang="ts">
	import { Play, Pause, Trash2, X } from '@lucide/svelte';
	import { getCurrentWindow } from '@tauri-apps/api/window';

	interface Props {
		title: string;
		taskCount: number;
		hasDownloading: boolean;
		hasPaused: boolean;
		hasRemovable: boolean;
		isSelectionMode: boolean;
		selectedCount: number;
		onGlobalPause?: () => void;
		onGlobalResume?: () => void;
		onTrashClick?: () => void;
		onExitSelection?: () => void;
	}

	let {
		title,
		taskCount,
		hasDownloading,
		hasPaused,
		hasRemovable,
		isSelectionMode,
		selectedCount,
		onGlobalPause,
		onGlobalResume,
		onTrashClick,
		onExitSelection
	}: Props = $props();

	const trashTooltip = $derived(() => {
		if (isSelectionMode) {
			return selectedCount === 0 ? '全选本次显示的任务' : `删除选中 (${selectedCount})`;
		}
		return '批量管理';
	});

	function startDrag() {
		getCurrentWindow().startDragging();
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<header class="floating-header" onmousedown={startDrag}>
	<div class="header-left">
		<h1>{title}</h1>
		<span class="task-count">{taskCount} 个任务</span>
	</div>
	
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="header-actions" onmousedown={(e) => e.stopPropagation()}>
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

		{#if isSelectionMode}
			<button class="icon-btn" onclick={onExitSelection} title="退出选择">
				<X size={18} />
			</button>
		{/if}

		{#if hasRemovable || taskCount > 0}
			<button class="icon-btn danger" onclick={onTrashClick} title={trashTooltip()}>
				<Trash2 size={18} />
			</button>
		{/if}
	</div>
</header>

<style>
	/* Header 作为面板内嵌头部 */
	.floating-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 18px 24px;
		border-bottom: 1px solid var(--border-subtle);
		flex-shrink: 0;
		-webkit-app-region: drag;
	}

	.header-left {
		display: flex;
		align-items: baseline;
		gap: 12px;
		position: relative;
		pointer-events: none; /* 让鼠标事件穿透到下层 overlay，但保留文本渲染 */
		z-index: 0; 
	}

	.header-actions {
		display: flex;
		gap: 8px;
		-webkit-app-region: no-drag;
		position: relative;
		z-index: 10; /* 确保按钮在遮罩层之上 */
	}

	/* 确保 header 本身相对定位 */
	.floating-header {
		position: relative;
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
