<script lang="ts">
	import Sidebar from '$lib/components/layout/Sidebar.svelte';
	import TaskListHeader from '$lib/components/layout/TaskListHeader.svelte';
	import TaskList from '$lib/components/layout/TaskList.svelte';
	import AddTaskDialog from '$lib/components/dialogs/AddTaskDialog.svelte';
	import SettingsPanel from '$lib/components/settings/SettingsPanel.svelte';
	import ClearConfirmDialog from '$lib/components/dialogs/ClearConfirmDialog.svelte';
	import TaskDetailsModal from '$lib/components/dialogs/TaskDetailsModal.svelte';

	import type { DownloadConfig, DownloadTask } from '$lib/types/download';
	import { 
		activeTasks, 
		completeTasks, 
		allTasks, 
		downloadStats,
		pauseTask,
		pauseAll,
		resumeAll,
		hasDownloadingTasks,
		hasPausedTasks
	} from '$lib';
	import { isRemovableTask } from '$lib';
    import { TaskController } from '$lib/services/taskController.svelte';

    // 实例化控制器
    const controller = new TaskController();

	// ============ 界面状态 ============
	let showAddDialog = $state(false);
	let showSettings = $state(false);
	
	// 任务详情弹窗状态
	let detailsTaskId = $state<string | null>(null);
	const detailsTask = $derived($allTasks.find(t => t.id === detailsTaskId) || null);
	const showDetailsModal = $derived(detailsTask !== null);

	// ============ Derived States ============

	// 当前显示的任务列表
	const filteredTasks = $derived.by(() => {
		switch (controller.activeNav) {
			case 'active':
				return $activeTasks;
			case 'complete':
				return $completeTasks;
			case 'history':
				return $allTasks;
			default:
				return $allTasks;
		}
	});

	// 页面标题
	const pageTitle = $derived.by(() => {
		switch (controller.activeNav) {
			case 'active': return '进行中';
			case 'complete': return '已完成';
			case 'history': return '历史记录';
			default: return '历史记录';
		}
	});

	// 空状态提示文案
	const emptyStateText = $derived.by(() => {
		switch (controller.activeNav) {
			case 'active': 
				return {
					title: '暂无进行中的任务',
					hint: '点击左侧「添加任务」按钮开始下载'
				};
			case 'complete': 
				return {
					title: '暂无已完成的任务',
					hint: '完成的下载任务会显示在这里'
				};
			case 'history': 
				return {
					title: '暂无历史记录',
					hint: '所有下载任务的历史会显示在这里'
				};
			default: 
				return {
					title: '暂无任务',
					hint: '点击左侧「添加任务」按钮开始下载'
				};
		}
	});

	// 判断当前列表中是否有正在下载/暂停/可删除的任务
	const hasDownloading = $derived(hasDownloadingTasks(filteredTasks));
	const hasPaused = $derived(hasPausedTasks(filteredTasks));
	const hasRemovable = $derived(filteredTasks.some(t => isRemovableTask(t.state)));

	// ============ Effects ============

	// 自动跳转逻辑：仅当任务"全部完成"时跳转
	let prevActiveIds: string[] = [];
	$effect(() => {
		const currentIds = $activeTasks.map(d => d.id);

		// 触发条件：处于 Active 页面，之前有任务，现在没了
		if (controller.activeNav === 'active' && prevActiveIds.length > 0 && currentIds.length === 0) {
			// 检查消失的任务是否全部完成
			const allCompleted = prevActiveIds.every(id => 
				$completeTasks.some(t => t.id === id)
			);

			if (allCompleted) {
                controller.handleNavChange('complete');
			}
		}
		prevActiveIds = currentIds;
	});

	// ============ Event Handlers ============

	function handleAddTask(config: DownloadConfig) {
		controller.handleAddTask(config);
        showAddDialog = false;
	}
	
	function handleShowDetails(task: DownloadTask) {
		detailsTaskId = task.id;
	}
</script>

<!-- 侧边栏 -->
<Sidebar 
	activeNav={controller.activeNav}
	onNavChange={(nav) => controller.handleNavChange(nav)}
	onSettingsClick={() => showSettings = true}
	onAddClick={() => showAddDialog = true}
	stats={$downloadStats}
/>

<!-- 主内容区 -->
<main class="main-content">
	<div class="content-panel">
		<TaskListHeader
			title={pageTitle}
			taskCount={filteredTasks.length}
			hasDownloading={controller.activeNav === 'active' && hasDownloading}
			hasPaused={controller.activeNav === 'active' && hasPaused}
			{hasRemovable}
			isSelectionMode={controller.isSelectionMode}
			selectedCount={controller.selectedIds.size}
			onGlobalPause={pauseAll}
			onGlobalResume={resumeAll}
			onTrashClick={() => controller.handleTrashClick(filteredTasks)}
			onExitSelection={() => controller.exitSelectionMode()}
		/>

		<TaskList
			tasks={filteredTasks}
			emptyTitle={emptyStateText.title}
			emptyHint={emptyStateText.hint}
			isSelectionMode={controller.isSelectionMode}
			selectedIds={controller.selectedIds}
			onSelect={(id) => controller.toggleSelection(id)}
			onPause={pauseTask}
			onResume={(id) => controller.handleResumeTask(id)}
			onCancel={(task) => controller.handleCancelTask(task)}
			onOpenFolder={(id) => controller.handleOpenFolder(id)}
			onShowDetails={handleShowDetails}
			groupByDate={controller.activeNav === 'history'}
		/>
	</div>
</main>

<!-- 添加任务对话框 -->
<AddTaskDialog 
	open={showAddDialog}
	onClose={() => showAddDialog = false}
	onSubmit={handleAddTask}
/>

<!-- 设置面板 -->
<SettingsPanel 
	open={showSettings}
	onClose={() => showSettings = false}
/>

<!-- 清空确认弹窗 -->
<ClearConfirmDialog
	open={controller.showClearDialog}
	title={controller.clearDialogProps.title}
	description={controller.clearDialogProps.description}
	confirmText={controller.clearDialogProps.confirmText}
	showDeleteFileOption={controller.activeNav !== 'active'}
	onClose={() => {
		controller.showClearDialog = false;
		controller.itemToDelete = null;
	}}
	onConfirm={(del) => controller.performClear(del)}
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
		onOpenFolder={() => controller.handleOpenFolder(detailsTask.id)}
		onClose={() => detailsTaskId = null}
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
	}

	/* 统一的玻璃面板容器 - 无模糊，让粒子透过 */
	.content-panel {
		flex: 1;
		min-height: 0;
		background: var(--glass-bg);
		border: 1px solid var(--glass-border);
		border-radius: 16px;
		box-shadow: var(--glass-shadow);
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}
</style>
