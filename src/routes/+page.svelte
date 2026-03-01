<script lang="ts">
	import Sidebar from "$lib/components/layout/Sidebar.svelte";
	import TaskListHeader from "$lib/components/layout/TaskListHeader.svelte";
	import TaskList from "$lib/components/layout/TaskList.svelte";
	import AddTaskDialog from "$lib/components/dialogs/AddTaskDialog.svelte";
	import SettingsPanel from "$lib/components/settings/SettingsPanel.svelte";
	import ClearConfirmDialog from "$lib/components/dialogs/ClearConfirmDialog.svelte";
	import TaskDetailsModal from "$lib/components/dialogs/TaskDetailsModal.svelte";
	import TorrentConfigDialog from "$lib/components/dialogs/TorrentConfigDialog.svelte";
	import type { DownloadTask } from "$lib/types/download";
	import { downloadStats } from "$lib";
	import { TaskController } from "$lib/services/download";
	import { useHomePageOrchestration } from "./useHomePageOrchestration.svelte";
	import { useTaskViewState } from "./useTaskViewState.svelte";
	import { useTaskDetailsState } from "./useTaskDetailsState.svelte";

	// 实例化控制器
	const controller = new TaskController();

	const page = useHomePageOrchestration(controller);
	const taskView = useTaskViewState(controller);
	const detailsState = useTaskDetailsState();
	const detailsTask = $derived(detailsState.detailsTask);
	const showDetailsModal = $derived(detailsTask !== null);

	// 计算是否需要背景虚化 (任一模态框打开时)
	const isBlurred = $derived(page.showSettings || showDetailsModal || page.showTorrentConfig);


	// ============ Event Handlers ============

	function handleShowDetails(task: DownloadTask) {
		detailsState.openDetails(task);
	}
</script>

<svelte:window
	ondragover={(e) => e.preventDefault()}
	ondrop={(e) => e.preventDefault()}
/>

<!-- 侧边栏 -->
	<Sidebar
		activeNav={controller.activeNav}
		onNavChange={(nav) => controller.setNav(nav)}
		onSettingsClick={page.openSettings}
		onAddClick={page.openAddDialog}
		stats={$downloadStats}
		blurred={isBlurred}
	/>

<!-- 主内容区 -->
<main class="main-content" class:content-blurred={isBlurred}>
	<div class="content-panel">
		<TaskListHeader
			title={taskView.pageTitle}
			taskCount={taskView.filteredTasks.length}
			hasDownloading={controller.activeNav === "active" && taskView.hasDownloading}
			hasPaused={controller.activeNav === "active" && taskView.hasPaused}
			hasRemovable={taskView.hasRemovable}
			isSelectionMode={controller.isSelectionMode}
			selectedCount={controller.selectedIds.size}
			onGlobalPause={() => controller.pauseAll()}
			onGlobalResume={() => controller.resumeAll()}
			onTrashClick={() => controller.onTrashClick(taskView.filteredTasks)}
			onExitSelection={() => controller.exitSelectionMode()}
		/>

		<TaskList
			tasks={taskView.filteredTasks}
			emptyTitle={taskView.emptyStateText.title}
			emptyHint={taskView.emptyStateText.hint}
			isSelectionMode={controller.isSelectionMode}
			selectedIds={controller.selectedIds}
			onSelect={(id) => controller.toggleSelection(id)}
			onPause={(id) => controller.pauseTask(id)}
			onResume={(id) => controller.resumeTask(id)}
			onCancel={(task) => controller.cancelOrDeleteTask(task)}
			onOpenFolder={(id) => controller.openTaskFolder(id)}
			onShowDetails={handleShowDetails}
			groupByDate={controller.activeNav === "history"}
		/>
	</div>
</main>

<!-- 添加任务对话框 -->
<AddTaskDialog
	open={page.showAddDialog}
	onClose={page.closeAddDialog}
	onSubmit={page.handleAddTask}
	onTorrentSelect={page.openTorrentConfig}
/>

<!-- 设置面板 -->
<SettingsPanel open={page.showSettings} onClose={page.closeSettings} />

<!-- 清空确认弹窗 -->
<ClearConfirmDialog
	open={controller.showClearDialog}
	title={controller.clearDialogProps.title}
	description={controller.clearDialogProps.description}
	confirmText={controller.clearDialogProps.confirmText}
	showDeleteFileOption={controller.activeNav !== "active"}
	onClose={() => {
		controller.showClearDialog = false;
		controller.itemToDelete = null;
	}}
	onConfirm={(del) => controller.confirmClear(del)}
/>

<!-- 任务详情弹窗 (页面级别渲染以解决 z-index 问题) -->
{#if detailsTask}
	<TaskDetailsModal
		open={showDetailsModal}
		filename={detailsTask.filename}
		url={detailsTask.url}
		state={detailsTask.state}
		savePath={detailsTask.savePath}
		errorMessage={detailsTask.errorMessage}
		userAgent={detailsTask.userAgent}
		referer={detailsTask.referer}
		proxy={detailsTask.proxy}
		headers={detailsTask.headers}
		addedAt={detailsTask.addedAt}
		completedAt={detailsTask.completedAt}
	onOpenFolder={() => controller.openTaskFolder(detailsTask.id)}
		onClose={detailsState.closeDetails}
	/>
{/if}

<!-- 全局拖拽覆盖层 -->
{#if page.isDragOver}
	<div class="global-drag-overlay">
		<div class="drag-hint">
			<svg
				width="48"
				height="48"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.5"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
				<polyline points="7 10 12 15 17 10" />
				<line x1="12" y1="15" x2="12" y2="3" />
			</svg>
			<span>释放以添加种子文件</span>
		</div>
	</div>
{/if}

<!-- Torrent 配置弹窗（全局级别） -->
{#if page.showTorrentConfig}
	<TorrentConfigDialog
		open={page.showTorrentConfig}
		torrentInfo={page.pendingTorrentInfo}
		torrentPath={page.pendingTorrentPath}
		parseError={page.pendingParseError}
		onConfirm={page.handleTorrentConfirm}
		onCancel={page.handleTorrentCancel}
	/>
{/if}

<style>
	/* 主内容区调整 */
	.main-content {
		flex: 1;
		margin-left: var(--sidebar-total-width);
		padding: 12px 12px 12px 0;
		height: 100vh;
		box-sizing: border-box;
		display: flex;
		flex-direction: column;
		position: relative;
		z-index: 1;
		transition: filter 0.3s ease;
	}

	/* 统一的玻璃面板容器 - 无模糊，让粒子透过 */
	.content-panel {
		flex: 1;
		min-height: 0;
		background: var(--panel-glass-bg, var(--glass-bg));
		border: 1px solid var(--panel-glass-border, var(--glass-border));
		border-radius: 16px;
		box-shadow: var(--panel-glass-shadow, var(--glass-shadow));
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	/* 全局拖拽覆盖层 */
	.global-drag-overlay {
		position: fixed;
		inset: 0;
		z-index: 9999;
		background: rgba(0, 0, 0, 0.6);
		backdrop-filter: blur(4px);
		-webkit-backdrop-filter: blur(4px);
		display: flex;
		align-items: center;
		justify-content: center;
		pointer-events: none;
	}

	.drag-hint {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 16px;
		color: var(--accent-primary);
		font-size: 18px;
		font-weight: 600;
		text-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
		animation: drag-pulse 1.5s ease-in-out infinite;
	}

	@keyframes drag-pulse {
		0%,
		100% {
			transform: scale(1);
			opacity: 0.9;
		}
		50% {
			transform: scale(1.05);
			opacity: 1;
		}
	}

	:global(html.dark) .content-panel {
		background:
			linear-gradient(
				164deg,
				color-mix(in srgb, var(--glass-elevated-bg) 86%, var(--accent-primary) 10%),
				color-mix(in srgb, var(--glass-bg) 96%, transparent)
			),
			var(--panel-glass-bg, var(--glass-bg));
	}

	:global(html.dark) .global-drag-overlay {
		background: color-mix(in srgb, var(--dialog-overlay-bg) 92%, rgba(2, 8, 20, 0.72));
		backdrop-filter: blur(12px) saturate(130%);
		-webkit-backdrop-filter: blur(12px) saturate(130%);
	}

	:global(html.dark) .drag-hint {
		color: var(--accent-on-glass, var(--accent-text));
		text-shadow: 0 6px 18px rgba(1, 7, 18, 0.72);
	}
</style>
