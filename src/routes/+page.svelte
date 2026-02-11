<script lang="ts">
	import Sidebar from '$lib/components/layout/Sidebar.svelte';
	import TaskListHeader from '$lib/components/layout/TaskListHeader.svelte';
	import TaskList from '$lib/components/layout/TaskList.svelte';
	import AddTaskDialog from '$lib/components/dialogs/AddTaskDialog.svelte';
	import SettingsPanel from '$lib/components/settings/SettingsPanel.svelte';
	import ClearConfirmDialog from '$lib/components/dialogs/ClearConfirmDialog.svelte';
	import TaskDetailsModal from '$lib/components/dialogs/TaskDetailsModal.svelte';
	import TorrentConfigDialog from '$lib/components/dialogs/TorrentConfigDialog.svelte';
	import type { TorrentDialogResult } from '$lib/components/dialogs/TorrentConfigDialog.svelte';
	import { onMount } from 'svelte';
	import { parseTorrent, type TorrentInfo } from '$lib/api/cmd';
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

	// ============ 全局拖拽状态 ============
	let isDragOver = $state(false);
	let showTorrentConfig = $state(false);
	let pendingTorrentInfo = $state<TorrentInfo | null>(null);
	let pendingTorrentPath = $state('');
	let pendingParseError = $state('');

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

	// 全局 Tauri drag-drop 事件监听（onMount + 动态导入，避免 SSR 问题）
	onMount(() => {
		const unlisteners: (() => void)[] = [];

		(async () => {
			try {
				const { listen } = await import('@tauri-apps/api/event');

				const u1 = await listen<{ paths: string[]; position: { x: number; y: number } }>('tauri://drag-enter', () => {
					isDragOver = true;
					lastDragTime = Date.now();
				});
				unlisteners.push(u1);

				const u2 = await listen<{ position: { x: number; y: number } }>('tauri://drag-over', () => {
					// 持续触发，更新时间戳
					lastDragTime = Date.now();
				});
				unlisteners.push(u2);

				const u3 = await listen<{ paths: string[]; position: { x: number; y: number } }>('tauri://drag-drop', (event) => {
					isDragOver = false;
					const paths = event.payload.paths;
					if (paths && paths.length > 0) {
						handleGlobalFileDrop(paths);
					}
				});
				unlisteners.push(u3);

				const u4 = await listen('tauri://drag-leave', () => {
					isDragOver = false;
				});
				unlisteners.push(u4);
			} catch (e) {
				console.error('[DragDrop] Failed to register handlers:', e);
			}
		})();

		return () => {
			unlisteners.forEach(fn => fn());
		};
	});

	// 打开种子配置弹窗：统一入口（拖拽 / AddTaskDialog 选择文件 共用）
	function openTorrentConfig(path: string) {
		pendingTorrentPath = path;
		pendingTorrentInfo = null;
		pendingParseError = '';
		showTorrentConfig = true;
		showAddDialog = false; // 关闭 AddTaskDialog

		// 后台异步解析，不阻塞 UI
		parseTorrent(path).then(info => {
			if (info.files.length > 1000) {
				console.warn('[Torrent] Large file count:', info.files.length);
			}
			pendingTorrentInfo = info;
		}).catch(e => {
			console.error('Failed to parse torrent:', e);
			pendingParseError = typeof e === 'string' ? e : '种子解析失败，但仍可提交任务';
		});
	}

	// 全局拖拽处理：.torrent 文件直接打开配置弹窗
	function handleGlobalFileDrop(paths: string[]) {
		const torrentFile = paths.find(p => p.toLowerCase().endsWith('.torrent'));
		if (torrentFile) {
			openTorrentConfig(torrentFile);
		}
	}

	function handleTorrentConfirm(result: TorrentDialogResult) {
		const config: DownloadConfig = {
			urls: [],
			savePath: result.savePath,
			filename: '',
			userAgent: '',
			referer: '',
			headers: '',
			proxy: '',
			maxDownloadLimit: '',
			torrentConfig: {
				path: result.torrentPath,
				selectFile: result.selectedFiles,
			}
		};
		controller.handleAddTask(config)
			.catch(e => console.error('[TorrentConfirm] addTask failed:', e));
		showTorrentConfig = false;
		pendingTorrentInfo = null;
		pendingTorrentPath = '';
	}

	function handleTorrentCancel() {
		showTorrentConfig = false;
		pendingTorrentInfo = null;
		pendingTorrentPath = '';
		pendingParseError = '';
	}


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

	function handleAddTask(config: DownloadConfig | DownloadConfig[]) {
		controller.handleAddTask(config);
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
	onTorrentSelect={(path) => openTorrentConfig(path)}
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

<!-- 全局拖拽覆盖层 -->
{#if isDragOver}
	<div class="global-drag-overlay">
		<div class="drag-hint">
			<svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
				<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
				<polyline points="7 10 12 15 17 10"/>
				<line x1="12" y1="15" x2="12" y2="3"/>
			</svg>
			<span>释放以添加种子文件</span>
		</div>
	</div>
{/if}

<!-- Torrent 配置弹窗（全局级别） -->
{#if showTorrentConfig}
	<TorrentConfigDialog
		open={showTorrentConfig}
		torrentInfo={pendingTorrentInfo}
		torrentPath={pendingTorrentPath}
		parseError={pendingParseError}
		onConfirm={handleTorrentConfirm}
		onCancel={handleTorrentCancel}
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
		0%, 100% { transform: scale(1); opacity: 0.9; }
		50% { transform: scale(1.05); opacity: 1; }
	}
</style>
