<script lang="ts">
	import Sidebar from '$lib/components/Sidebar.svelte';
	import TaskListHeader from '$lib/components/TaskListHeader.svelte';
	import TaskList from '$lib/components/TaskList.svelte';
	import AddTaskDialog from '$lib/components/AddTaskDialog.svelte';
	import SettingsPanel from '$lib/components/SettingsPanel.svelte';
	import ClearConfirmDialog from '$lib/components/ClearConfirmDialog.svelte';
	import { totalDownloadSpeed } from '$lib/stores/downloadSpeed';
	import type { DownloadConfig } from '$lib/types/download';
	import { 
		activeTasks, 
		completedTasks, 
		allTasks, 
		downloadStats,
		addDownloadTask,
		pauseTask,
		resumeTask,
		cancelTask,
		removeTask,
		removeTasks,
		cancelTasks,
		pauseAll,
		resumeAll,
		hasDownloadingTasks,
		hasPausedTasks
	} from '$lib';
	import { isRemovableTask } from '$lib';

	// ============ 界面状态 ============
	let activeNav: 'active' | 'completed' | 'history' = $state('active');
	let showAddDialog = $state(false);
	let showClearDialog = $state(false);
	let showSettings = $state(false);
	let isSelectionMode = $state(false);
	let selectedIds = $state(new Set<string>());
	let clearDialogProps = $state({
		title: '确认清空',
		description: '确定要清空这些任务吗？此操作无法撤销。',
		confirmText: '清空'
	});

	// ============ Derived States ============

	// 当前显示的任务列表
	const filteredTasks = $derived(() => {
		switch (activeNav) {
			case 'active':
				return $activeTasks;
			case 'completed':
				return $completedTasks;
			case 'history':
				return $allTasks;
			default:
				return $allTasks;
		}
	});

	// 页面标题
	const pageTitle = $derived(() => {
		switch (activeNav) {
			case 'active': return '进行中';
			case 'completed': return '已完成';
			case 'history': return '历史记录';
			default: return '历史记录';
		}
	});

	// 空状态提示文案
	const emptyStateText = $derived(() => {
		switch (activeNav) {
			case 'active': 
				return {
					title: '暂无进行中的任务',
					hint: '点击左侧「添加任务」按钮开始下载'
				};
			case 'completed': 
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
	const hasDownloading = $derived(hasDownloadingTasks(filteredTasks()));
	const hasPaused = $derived(hasPausedTasks(filteredTasks()));
	const hasRemovable = $derived(filteredTasks().some(t => isRemovableTask(t.state)));

	// ============ Effects ============

	// 同步下载速度到全局 store（用于粒子效果）
	$effect(() => {
		totalDownloadSpeed.set($downloadStats.totalSpeedBytes);
	});

	// 自动跳转逻辑：仅当任务"全部完成"时跳转
	let prevActiveIds: string[] = [];
	$effect(() => {
		const currentIds = $activeTasks.map(d => d.id);

		// 触发条件：处于 Active 页面，之前有任务，现在没了
		if (activeNav === 'active' && prevActiveIds.length > 0 && currentIds.length === 0) {
			// 检查消失的任务是否全部完成
			const allCompleted = prevActiveIds.every(id => 
				$completedTasks.some(t => t.id === id)
			);

			if (allCompleted) {
				activeNav = 'completed';
				isSelectionMode = false;
				selectedIds = new Set();
			}
		}
		prevActiveIds = currentIds;
	});

	// ============ Event Handlers ============

	function handleNavChange(nav: typeof activeNav) {
		activeNav = nav;
		isSelectionMode = false;
		selectedIds.clear();
	}

	function toggleSelection(id: string) {
		const next = new Set(selectedIds);
		if (next.has(id)) {
			next.delete(id);
		} else {
			next.add(id);
		}
		selectedIds = next;
	}

	function handleGlobalPause() {
		pauseAll();
	}

	function handleGlobalResume() {
		resumeAll();
	}

	// 垃圾桶按钮点击逻辑：进入模式 -> 全选 -> 确认删除
	function handleTrashClick() {
		// 1. 进入选择模式
		if (!isSelectionMode) {
			isSelectionMode = true;
			selectedIds = new Set();
			return;
		}
		
		// 2. 如果未选中任何项，则全选
		if (selectedIds.size === 0) {
			const next = new Set(selectedIds);
			filteredTasks().forEach(d => next.add(d.id));
			selectedIds = next;
			return;
		}

		// 3. 执行逻辑分流
		// 进行中页面：默认直接移动到历史记录（软删除），不需要弹窗
		if (activeNav === 'active') {
			performClear(false);
			return;
		}

		// 其他页面（已完成/历史）：需要弹窗确认 + 删除文件选项
		const count = selectedIds.size;
		let title = '';
		let description = '';

		if (activeNav === 'history') {
			title = '删除记录';
			description = `确定要永久删除这 ${count} 条任务记录吗？`;
		} else {
			title = '清空记录';
			description = `确定要清空这 ${count} 条已完成的任务记录吗？`;
		}

		clearDialogProps = {
			title,
			description,
			confirmText: '确定'
		};
		showClearDialog = true;
	}

	// 执行批量清理
	function performClear(deleteFile: boolean) {
		if (activeNav === 'active') {
			// 进行中页面：软删除（取消）
			cancelTasks(selectedIds);
		} else {
			// 历史/已完成页面：硬删除
			removeTasks(selectedIds, deleteFile);
		}
		
		showClearDialog = false;
		isSelectionMode = false;
		selectedIds = new Set();
	}

	function handleAddTask(config: DownloadConfig) {
		addDownloadTask(config);
	}

	function handleCancelTask(id: string) {
		if (activeNav === 'active') {
			// 进行中：软删除（取消）
			cancelTask(id);
		} else {
			// 历史记录：物理删除
			removeTask(id);
		}
	}
</script>

<!-- 侧边栏 -->
<Sidebar 
	{activeNav}
	onNavChange={handleNavChange}
	onSettingsClick={() => showSettings = true}
	onAddClick={() => showAddDialog = true}
	stats={$downloadStats}
/>

<!-- 主内容区 -->
<main class="main-content">
	<div class="content-panel">
		<TaskListHeader
			title={pageTitle()}
			taskCount={filteredTasks().length}
			{hasDownloading}
			{hasPaused}
			{hasRemovable}
			{isSelectionMode}
			selectedCount={selectedIds.size}
			onGlobalPause={handleGlobalPause}
			onGlobalResume={handleGlobalResume}
			onTrashClick={handleTrashClick}
		/>

		<TaskList
			tasks={filteredTasks()}
			emptyTitle={emptyStateText().title}
			emptyHint={emptyStateText().hint}
			{isSelectionMode}
			{selectedIds}
			onSelect={toggleSelection}
			onPause={pauseTask}
			onResume={resumeTask}
			onCancel={handleCancelTask}
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
	open={showClearDialog}
	title={clearDialogProps.title}
	description={clearDialogProps.description}
	confirmText={clearDialogProps.confirmText}
	showDeleteFileOption={activeNav !== 'active'}
	onClose={() => showClearDialog = false}
	onConfirm={performClear}
/>

<style>
	/* 主内容区调整 */
	.main-content {
		flex: 1;
		margin-left: 224px;
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
