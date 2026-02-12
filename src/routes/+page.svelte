<script lang="ts">
	import Sidebar from '$lib/components/layout/Sidebar.svelte';
	import TaskListHeader from '$lib/components/layout/TaskListHeader.svelte';
	import TaskList from '$lib/components/layout/TaskList.svelte';
	import AddTaskDialog from '$lib/components/dialogs/AddTaskDialog.svelte';
	import SettingsPanel from '$lib/components/settings/SettingsPanel.svelte';
	import ClearConfirmDialog from '$lib/components/dialogs/ClearConfirmDialog.svelte';
	import TaskDetailsModal from '$lib/components/dialogs/TaskDetailsModal.svelte';
	import TorrentConfigDialog from '$lib/components/dialogs/TorrentConfigDialog.svelte';
	import { onMount } from 'svelte';
	import type { DownloadConfig, DownloadTask } from '$lib/types/download';
	import { 
		activeTasks, 
		completeTasks, 
		allTasks, 
		downloadStats,
		hasDownloadingTasks,
		hasPausedTasks
	} from '$lib';
	import { isRemovableTask } from '$lib';
    import { TaskController } from '$lib/services/taskController.svelte';
    import { TorrentDialogController } from '$lib/services/torrentDialogController.svelte';
	import { setupGlobalDragDrop, type DragMeta } from '$lib/services/globalDragDropController';
	import {
		getEmptyStateTextByNav,
		getFilteredTasksByNav,
		getPageTitleByNav
	} from '$lib/services/taskPageViewModel';

    // 实例化控制器
    const controller = new TaskController();
    const torrentController = new TorrentDialogController({
        onAddTask: async (config) => {
            await controller.handleAddTask(config);
            showAddDialog = false;
        }
    });

	// ============ 界面状态 ============
	let showAddDialog = $state(false);
	let showSettings = $state(false);
	
	// 任务详情弹窗状态
	let detailsTaskId = $state<string | null>(null);
	const detailsTask = $derived($allTasks.find(t => t.id === detailsTaskId) || null);
	const showDetailsModal = $derived(detailsTask !== null);

	// ============ 全局拖拽状态 ============
	let isDragOver = $state(false);
	let dragMeta = $state<DragMeta | null>(null);
	let dropFeedback = $state('');
	let contentPanelEl = $state<HTMLElement | null>(null);
	let addDialogDropZoneEl = $state<HTMLElement | null>(null);

	// 拖拽看门狗：防止 drag-leave 丢失导致遮罩卡死
	let lastDragTime = 0;
	$effect(() => {
		if (isDragOver) {
			lastDragTime = Date.now();
			const interval = setInterval(() => {
				// 如果超过 300ms 没有收到 drag-over 事件，认为拖拽已结束
				if (Date.now() - lastDragTime > 300) {
					isDragOver = false;
				}
			}, 100);
			return () => clearInterval(interval);
		}
	});

	$effect(() => {
		if (!dropFeedback) {
			return;
		}

		const timer = setTimeout(() => {
			dropFeedback = '';
		}, 2600);

		return () => clearTimeout(timer);
	});

	const showMainDragHint = $derived(isDragOver && !showAddDialog);

	function isPointInsideElement(
		element: HTMLElement | null,
		position?: { x: number; y: number }
	): boolean {
		if (!element || !position) {
			return false;
		}

		const rect = element.getBoundingClientRect();
		return (
			position.x >= rect.left &&
			position.x <= rect.right &&
			position.y >= rect.top &&
			position.y <= rect.bottom
		);
	}

	function canAcceptDrop(position?: { x: number; y: number }): boolean {
		if (showAddDialog) {
			if (!position) {
				return true;
			}
			return isPointInsideElement(addDialogDropZoneEl, position);
		}

		if (!position) {
			return true;
		}

		return isPointInsideElement(contentPanelEl, position);
	}

	function resetDragVisualState() {
		isDragOver = false;
		dragMeta = null;
	}

	const mainDragTitle = $derived.by(() => {
		if (!dragMeta) return '释放以导入 .torrent';
		if (!dragMeta.hasSupportedFiles) return '当前拖拽不包含 .torrent 文件';
		if (dragMeta.torrentFiles === 1) return '释放以打开种子配置';
		return `检测到 ${dragMeta.torrentFiles} 个种子文件`;
	});

	const mainDragHint = $derived.by(() => {
		if (!dragMeta) return '支持拖入种子文件，快速创建下载任务';
		if (!dragMeta.hasSupportedFiles) return '请拖入 .torrent 文件';

		const ignored = dragMeta.totalFiles - dragMeta.torrentFiles;
		if (ignored > 0) {
			return `共 ${dragMeta.totalFiles} 个文件，将仅处理 ${dragMeta.torrentFiles} 个 .torrent`;
		}

		return '释放后将进入种子配置';
	});

	// 全局 Tauri drag-drop 事件监听（onMount + 动态导入，避免 SSR 问题）
	onMount(() => {
		let unlistenAll: (() => void) | null = null;

		void setupGlobalDragDrop({
			onDragStateChange: (next) => {
				isDragOver = next;
			},
            onDragMetaChange: (meta) => {
                dragMeta = meta;
            },
			onDragPulse: () => {
				lastDragTime = Date.now();
			},
			onDropPaths: (paths, position) => {
				if (!canAcceptDrop(position)) {
					dropFeedback = '';
					return;
				}

				resetDragVisualState();

				const accepted = torrentController.openFromDrop(paths);
				if (!accepted) {
					dropFeedback = '仅支持 .torrent 文件，请重新拖入';
				}
			}
		}).then((cleanup) => {
			unlistenAll = cleanup;
		});

		return () => {
			unlistenAll?.();
		};
	});

	function openTorrentConfig(path: string) {
		resetDragVisualState();
		showAddDialog = false;
		torrentController.open(path);
	}

	function openAddDialog() {
		resetDragVisualState();
		showAddDialog = true;
	}

	function closeAddDialog() {
		resetDragVisualState();
		showAddDialog = false;
	}


	// 当前显示的任务列表
	const filteredTasks = $derived.by(() => {
		return getFilteredTasksByNav(controller.activeNav, $activeTasks, $completeTasks, $allTasks);
	});

	// 页面标题
	const pageTitle = $derived(getPageTitleByNav(controller.activeNav));

	// 空状态提示文案
	const emptyStateText = $derived(getEmptyStateTextByNav(controller.activeNav));

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

	async function handleAddTask(config: DownloadConfig | DownloadConfig[]) {
		await controller.handleAddTask(config);
		showAddDialog = false;
	}
	
	function handleShowDetails(task: DownloadTask) {
		detailsTaskId = task.id;
	}
</script>

<svelte:window 
	ondragover={(e) => e.preventDefault()}
	ondrop={(e) => e.preventDefault()}
/>

<!-- 侧边栏 -->
<Sidebar 
	activeNav={controller.activeNav}
	onNavChange={(nav) => controller.handleNavChange(nav)}
	onSettingsClick={() => showSettings = true}
	onAddClick={openAddDialog}
	stats={$downloadStats}
/>

<!-- 主内容区 -->
<main class="main-content">
	{#if dropFeedback}
		<div class="drop-feedback">{dropFeedback}</div>
	{/if}

	<div class="content-panel" class:drag-active={showMainDragHint} bind:this={contentPanelEl}>
		{#if showMainDragHint}
			<div class="panel-drag-overlay" aria-hidden="true">
				<div class="panel-drag-card">
					<svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
						<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
						<polyline points="7 10 12 15 17 10"/>
						<line x1="12" y1="15" x2="12" y2="3"/>
					</svg>
					<strong>{mainDragTitle}</strong>
					<span>{mainDragHint}</span>
				</div>
			</div>
		{/if}

		<TaskListHeader
			title={pageTitle}
			taskCount={filteredTasks.length}
			hasDownloading={controller.activeNav === 'active' && hasDownloading}
			hasPaused={controller.activeNav === 'active' && hasPaused}
			{hasRemovable}
			isSelectionMode={controller.isSelectionMode}
			selectedCount={controller.selectedIds.size}
			onGlobalPause={() => void controller.handlePauseAll()}
			onGlobalResume={() => void controller.handleResumeAll()}
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
			onPause={(id) => void controller.handlePauseTask(id)}
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
	onClose={closeAddDialog}
	onSubmit={handleAddTask}
	onTorrentSelect={(path) => openTorrentConfig(path)}
	dialogDragActive={showAddDialog && isDragOver && !torrentController.showConfig && Boolean(dragMeta)}
	dialogDragMeta={showAddDialog ? dragMeta : null}
	onUrlDropZoneChange={(el) => {
		addDialogDropZoneEl = el;
	}}
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

<!-- Torrent 配置弹窗（全局级别） -->
{#if torrentController.showConfig}
	<TorrentConfigDialog
		open={torrentController.showConfig}
		torrentInfo={torrentController.pendingInfo}
		torrentPath={torrentController.pendingPath}
		parseError={torrentController.pendingParseError}
		onConfirm={(result) => {
			resetDragVisualState();
			torrentController.confirm(result);
		}}
		onCancel={() => {
			resetDragVisualState();
			torrentController.cancel();
		}}
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

	.drop-feedback {
		position: absolute;
		top: 18px;
		right: 22px;
		z-index: 12;
		padding: 9px 14px;
		border-radius: 10px;
		background: var(--drop-feedback-bg);
		border: 1px solid var(--drop-feedback-border);
		color: var(--drop-feedback-text);
		font-size: 12px;
		font-weight: 600;
		box-shadow: 0 8px 20px rgba(0, 0, 0, 0.16);
		backdrop-filter: blur(10px);
		-webkit-backdrop-filter: blur(10px);
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
		position: relative;
	}

	.content-panel.drag-active {
		border-color: color-mix(in srgb, var(--accent-primary) 38%, var(--glass-border));
	}

	.panel-drag-overlay {
		position: absolute;
		inset: 8px;
		z-index: 8;
		border: 2px dashed var(--drag-zone-border);
		border-radius: 14px;
		background: var(--drag-zone-bg);
		display: flex;
		align-items: center;
		justify-content: center;
		pointer-events: none;
	}

	.panel-drag-card {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 6px;
		padding: 16px 20px;
		border-radius: 12px;
		background: var(--drag-card-bg);
		border: 1px solid var(--drag-card-border);
		box-shadow: var(--drag-card-shadow);
		color: var(--drag-title-text);
		text-align: center;
		max-width: min(560px, 92%);
	}

	.panel-drag-card svg {
		color: var(--drag-icon-color);
	}

	.panel-drag-card strong {
		font-size: 15px;
		font-weight: 700;
		line-height: 1.35;
	}

	.panel-drag-card span {
		font-size: 12px;
		font-weight: 500;
		color: var(--drag-hint-text);
	}

	@media (max-width: 768px) {
		.main-content {
			padding: 8px 8px 8px 0;
		}

	}
</style>
