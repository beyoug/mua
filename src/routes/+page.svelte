<script lang="ts">
	import DownloadCard from '$lib/components/DownloadCard.svelte';
	import AddTaskDialog from '$lib/components/AddTaskDialog.svelte';
	import SettingsPanel from '$lib/components/SettingsPanel.svelte';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import ClearConfirmDialog from '$lib/components/ClearConfirmDialog.svelte';
	import { Download, Play, Pause, Trash2 } from '@lucide/svelte';
	import { flip } from 'svelte/animate';
	import { totalDownloadSpeed } from '$lib/stores/downloadSpeed';
	import type { DownloadTask } from '$lib/types/download';

	let activeNav: 'active' | 'completed' | 'history' = $state('active');
	let showAddDialog = $state(false);
	let showClearDialog = $state(false);
	let showSettings = $state(false);
	let isSelectionMode = $state(false);
	let selectedIds = $state(new Set<string>());
	let dialogShowOption = $state(false);
	let clearDialogProps = $state({
		title: '确认清空',
		description: '确定要清空这些任务吗？此操作无法撤销。',
		confirmText: '清空'
	});
	const downloads = $state<DownloadTask[]>([
		{
			id: '1',
			filename: 'macOS-Tahoe-26.0.dmg',
			progress: 75,
			speed: '12.5 MB/s',
			downloaded: '3.2 GB',
			total: '4.3 GB',
			remaining: '1:28',
			state: 'downloading',
			addedAt: '2024-05-20 14:30'
		},
		{
			id: '2',
			filename: 'Xcode_16.2.xip',
			progress: 45,
			speed: '8.3 MB/s',
			downloaded: '2.1 GB',
			total: '4.7 GB',
			remaining: '5:12',
			state: 'downloading',
			addedAt: '2024-05-20 15:10'
		},
		{
			id: '3',
			filename: 'SF-Pro-Fonts.pkg',
			progress: 100,
			downloaded: '156 MB',
			total: '156 MB',
			state: 'completed',
			addedAt: '2024-05-19 09:20'
		},
		{
			id: '4',
			filename: 'node-v22.0.0.pkg',
			progress: 30,
			downloaded: '24 MB',
			total: '80 MB',
			state: 'paused',
			addedAt: '2024-05-18 18:45'
		},
		{
			id: '5',
			filename: 'docker-desktop.dmg',
			progress: 0,
			state: 'waiting',
			addedAt: '2024-05-21 10:00'
		}
	]);

	// 计算统计数据
	const stats = $derived(() => {
		const activeDownloads = downloads.filter(d => ['downloading', 'waiting'].includes(d.state));
		const completedDownloads = downloads.filter(d => d.state === 'completed');
		
		// 计算总速度（简单模拟）
		let totalSpeed = '0 B/s';
		let totalSpeedBytes = 0;
		if (activeDownloads.length > 0) {
			const speeds = activeDownloads
				.map(d => d.speed || '0')
				.map(s => parseFloat(s.replace(' MB/s', '')) || 0);
			const total = speeds.reduce((a, b) => a + b, 0);
			totalSpeed = `${total.toFixed(1)} MB/s`;
			totalSpeedBytes = total * 1024 * 1024;
		}

		return {
			totalSpeed,
			totalSpeedBytes,
			activeCount: activeDownloads.length,
			completedCount: completedDownloads.length
		};
	});

	// 同步速度到全局 store（用于粒子效果）
	$effect(() => {
		totalDownloadSpeed.set(stats().totalSpeedBytes);
	});

	let prevActiveIds: string[] = []; // 追踪上一时刻的活跃任务ID

	// 自动跳转逻辑：仅当任务"全部完成"时跳转，手动清空/取消不跳转
	$effect(() => {
		const activeList = downloads.filter(d => ['downloading', 'waiting', 'paused'].includes(d.state));
		const currentIds = activeList.map(d => d.id);

		// 触发条件：处于 Active 页面，之前有任务，现在没了
		if (activeNav === 'active' && prevActiveIds.length > 0 && currentIds.length === 0) {
			// 检查消失的任务的状态
			let allCompleted = true;
			for (const id of prevActiveIds) {
				const task = downloads.find(d => d.id === id);
				// 如果任务被删除(找不到)或者状态不是 completed，说明不是自然完成
				if (!task || task.state !== 'completed') {
					allCompleted = false;
					break;
				}
			}

			if (allCompleted) {
				activeNav = 'completed';
				isSelectionMode = false;
				selectedIds = new Set();
			}
		}
		prevActiveIds = currentIds;
	});

	// 根据导航筛选下载列表
	const filteredDownloads = $derived(() => {
		let list: typeof downloads = [];
		switch (activeNav) {
			case 'active':
				list = downloads.filter(d => ['downloading', 'waiting', 'paused'].includes(d.state));
				break;
			case 'completed':
				list = downloads.filter(d => d.state === 'completed');
				break;
			case 'history':
				list = [...downloads];
				break;
			default:
				list = [...downloads];
		}
		// 排序逻辑：
		// 1. 状态优先级：进行中(2) > 已暂停(1) > 其他(0)
		// 2. 添加时间：倒序（最新的在前）
		return list.sort((a, b) => {
			const getScore = (s: string) => {
				if (['downloading', 'waiting'].includes(s)) return 2;
				if (s === 'paused') return 1;
				return 0;
			};
			const scoreA = getScore(a.state);
			const scoreB = getScore(b.state);
			if (scoreA !== scoreB) return scoreB - scoreA; // 分数高在前

			const timeA = a.addedAt || '';
			const timeB = b.addedAt || '';
			return timeB.localeCompare(timeA);
		});
	});

	// 获取当前页面标题
	const pageTitle = $derived(() => {
		switch (activeNav) {
			case 'active': return '进行中';
			case 'completed': return '已完成';
			case 'history': return '历史记录';
			default: return '历史记录';
		}
	});

	const trashTooltip = $derived(() => {
		if (isSelectionMode) {
			return selectedIds.size === 0 ? '全选本次显示的任务' : `删除选中 (${selectedIds.size})`;
		}
		return '批量管理';
	});
	// 全局控制：暂停/开始
	function handleGlobalAction() {
		// 针对不同 Tab 的特定行为
		if (activeNav === 'active') {
			// 智能判断：如果有下载中 -> 全部暂停；否则 -> 全部开始
			const hasActive = downloads.some(d => d.state === 'downloading');
			
			if (hasActive) {
				downloads.forEach(d => {
					if (d.state === 'downloading') d.state = 'paused';
				});
			} else {
				downloads.forEach(d => {
					if (d.state === 'paused') d.state = 'downloading';
				});
			}
			return;
		}

		// "历史"页面的智能判断
		if (activeNav === 'history') {
			const hasActive = downloads.some(d => d.state === 'downloading');
			if (hasActive) {
				// 有下载中 -> 全部暂停
				downloads.forEach(d => {
					if (d.state === 'downloading') d.state = 'paused';
				});
			} else {
				// 无下载中 -> 恢复暂停
				downloads.forEach(d => {
					if (d.state === 'paused') d.state = 'downloading';
				});
			}
		}
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
			filteredDownloads().forEach(d => next.add(d.id));
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
		
		dialogShowOption = true;

		clearDialogProps = {
			title,
			description,
			confirmText: '确定'
		};
		showClearDialog = true;
	}

	// 执行批量清理
	function performClear(deleteFile: boolean) {
		for (let i = downloads.length - 1; i >= 0; i--) {
			const d = downloads[i];
			
			// 仅处理选中的项
			if (!selectedIds.has(d.id)) continue;

			if (activeNav === 'active') {
				// 进行中页面：软删除（取消）
				d.state = 'cancelled';
			} else {
				// 历史/已完成页面：硬删除（仅删除非活跃任务，双重保险）
				if (['completed', 'cancelled', 'error'].includes(d.state)) {
					downloads.splice(i, 1);
					if (deleteFile) {
						// Delete logic here
					}
				}
			}
		}
		
		
		showClearDialog = false;
		isSelectionMode = false;
		selectedIds = new Set();
	}

	interface DownloadConfig {
		urls: string[];
		savePath: string;
		filename: string;
		userAgent: string;
		referer: string;
		headers: string;
		proxy: string;
		maxDownloadLimit: string;
	}

	function handleAddTask(config: DownloadConfig) {
		const now = new Date();
		const timeString = new Intl.DateTimeFormat('zh-CN', {
			year: 'numeric',
			month: '2-digit',
			day: '2-digit',
			hour: '2-digit',
			minute: '2-digit',
			hour12: false
		}).format(now).replace(/\//g, '-');
		
		for (const url of config.urls) {
			const newTask: DownloadTask = {
				id: crypto.randomUUID(),
				// 优先使用用户指定的文件名，否则从 URL 提取
				filename: config.filename || url.split('/').pop() || 'new-download',
				progress: 0,
				state: 'waiting',
				addedAt: timeString
			};
			downloads.push(newTask);
		}
		// TODO: 将 config 中的高级设置传递给 aria2
		console.log('Download config:', config);
	}

	// 单个任务操作处理
	function handlePause(id: string) {
		const task = downloads.find(d => d.id === id);
		if (task) task.state = 'paused';
	}

	function handleResume(id: string) {
		const task = downloads.find(d => d.id === id);
		if (task) task.state = 'downloading';
	}

	function handleCancelTask(id: string) {
		const index = downloads.findIndex(d => d.id === id);
		if (index === -1) return;
		const task = downloads[index];

		if (activeNav === 'active') {
			// 进行中（含暂停）：软删除（取消），不弹窗
			task.state = 'cancelled';
		} else {
			// 历史记录：点击 X 代表物理删除一条记录
			downloads.splice(index, 1);
		}
	}
</script>

<!-- 侧边栏 -->
	<Sidebar 
		{activeNav}
		onNavChange={(nav) => {
			activeNav = nav;
			isSelectionMode = false;
			selectedIds.clear();
		}}
		onSettingsClick={() => showSettings = true}
		onAddClick={() => showAddDialog = true}
		stats={stats()}
	/>

<!-- 主内容区 -->
<main class="main-content">
	<header class="floating-header">
		<div class="header-left">
			<h1>{pageTitle()}</h1>
			<span class="task-count">{filteredDownloads().length} 个任务</span>
		</div>
		
		<div class="header-actions">
			{#if activeNav === 'history'}
				<!-- 历史标签：智能显示 -->
				{#if !isSelectionMode}
					{#if downloads.some(d => d.state === 'downloading')}
						<button class="icon-btn" onclick={handleGlobalAction} title="全部暂停">
							<Pause size={18} fill="currentColor" />
						</button>
					{:else if downloads.some(d => d.state === 'paused')}
						<button class="icon-btn" onclick={handleGlobalAction} title="全部开始">
							<Play size={18} fill="currentColor" />
						</button>
					{/if}
				{/if}
				{#if downloads.some(d => ['completed', 'cancelled', 'error'].includes(d.state))}
					<button class="icon-btn danger" onclick={handleTrashClick} title={trashTooltip()}>
						<Trash2 size={18} />
					</button>
				{/if}

			{:else if activeNav === 'active'}
				<!-- 进行中标签：智能显示 暂停/开始 + 取消 -->
				{#if !isSelectionMode}
					{#if downloads.some(d => d.state === 'downloading')}
						<button class="icon-btn" onclick={handleGlobalAction} title="全部暂停">
							<Pause size={18} fill="currentColor" />
						</button>
					{:else if filteredDownloads().some(d => d.state === 'paused')}
						<button class="icon-btn" onclick={handleGlobalAction} title="全部开始">
							<Play size={18} fill="currentColor" />
						</button>
					{/if}
				{/if}

				{#if filteredDownloads().length > 0}
					<button class="icon-btn danger" onclick={handleTrashClick} title={trashTooltip()}>
						<Trash2 size={18} />
					</button>
				{/if}

			{:else if activeNav === 'completed'}
				<!-- 已完成标签：清空 -->
				{#if filteredDownloads().length > 0}
					<button class="icon-btn danger" onclick={handleTrashClick} title={trashTooltip()}>
						<Trash2 size={18} />
					</button>
				{/if}
			{/if}
		</div>
	</header>

	<section class="downloads-list">
		{#each filteredDownloads() as download (download.id)}
			<div animate:flip={{ duration: 400 }}>
				<DownloadCard
					filename={download.filename}
					progress={download.progress}
					speed={download.speed}
					downloaded={download.downloaded}
					total={download.total}
					remaining={download.remaining}
					state={download.state}
					selectionMode={isSelectionMode}
					selected={selectedIds.has(download.id)}
					onSelect={() => toggleSelection(download.id)}
					onPause={() => handlePause(download.id)}
					onResume={() => handleResume(download.id)}
					onCancel={() => handleCancelTask(download.id)}
					addedAt={download.addedAt}
				/>
			</div>
		{/each}
	</section>

	{#if filteredDownloads().length === 0}
		<div class="empty-state">
			<div class="empty-icon">
				<div class="ripple"></div>
				<div class="ripple delay"></div>
				<Download size={48} strokeWidth={1.5} />
			</div>
			<p class="empty-title">暂无下载任务</p>
			<p class="empty-hint">点击左侧「添加任务」按钮开始下载</p>
		</div>
	{/if}
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
		showDeleteFileOption={dialogShowOption}
		onClose={() => showClearDialog = false}
		onConfirm={performClear}
	/>

<style>
	/* 主内容区调整 */
	.main-content {
		flex: 1;
		margin-left: 224px;
		padding: 0 20px 20px; /* 移除顶部 padding，由 header margin 控制 */
		min-height: 100vh;
		box-sizing: border-box;
		display: flex;
		flex-direction: column;
	}

	/* 悬浮玻璃 Header */
	.floating-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px 24px;
		margin: 12px 0 24px; /* Top 12px 对齐侧边栏 */
		background: var(--bg-sidebar);
		backdrop-filter: blur(24px) saturate(180%);
		-webkit-backdrop-filter: blur(24px) saturate(180%);
		border: 1px solid var(--border-color);
		border-radius: 20px;
		box-shadow: 
			0 8px 32px rgba(0, 0, 0, 0.12),
			0 1px 2px rgba(255, 255, 255, 0.1) inset;
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
		background: rgba(220, 38, 38, 0.1); /* Red-500 with opacity */
		color: #ef4444; /* Red-500 */
		border-color: #fca5a5; /* Red-300 */
	}

	.downloads-list {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.empty-state {
		flex: 1;
		width: 100%;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		color: var(--text-muted);
	}

	.empty-icon {
		width: 96px;
		height: 96px;
		background: var(--bg-sidebar);
		border: 1px solid var(--border-color);
		box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
		border-radius: 32px;
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--text-muted);
		margin-bottom: 24px;
		position: relative;
		animation: float 6s ease-in-out infinite;
	}

	.ripple {
		position: absolute;
		width: 100%;
		height: 100%;
		border-radius: 32px;
		border: 1px solid var(--accent-primary);
		opacity: 0;
		z-index: -1;
		animation: ripple 3s linear infinite;
	}

	.ripple.delay {
		animation-delay: 1.5s;
	}

	@keyframes ripple {
		0% { transform: scale(1); opacity: 0.4; }
		100% { transform: scale(1.6); opacity: 0; }
	}

	@keyframes float {
		0%, 100% { transform: translateY(0); }
		50% { transform: translateY(-10px); }
	}

	.empty-title {
		font-size: 18px;
		font-weight: 500;
		color: var(--text-secondary);
		margin: 0 0 8px;
	}

	.empty-hint {
		font-size: 14px;
		color: var(--text-muted);
		margin: 0 0 24px;
	}
</style>
