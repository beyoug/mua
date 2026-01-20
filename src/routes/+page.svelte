<script lang="ts">
	import DownloadCard from '$lib/components/DownloadCard.svelte';
	import HeaderToolbar from '$lib/components/HeaderToolbar.svelte';
	import AddTaskDialog from '$lib/components/AddTaskDialog.svelte';

	let activeTab: 'all' | 'active' | 'completed' | 'paused' = $state('all');
	let showAddDialog = $state(false);

	// 模拟下载数据
	const downloads = $state([
		{
			id: '1',
			filename: 'macOS-Tahoe-26.0.dmg',
			progress: 75,
			speed: '12.5 MB/s',
			downloaded: '3.2 GB',
			total: '4.3 GB',
			remaining: '1:28',
			state: 'downloading' as const
		},
		{
			id: '2',
			filename: 'Xcode_16.2.xip',
			progress: 45,
			speed: '8.3 MB/s',
			downloaded: '2.1 GB',
			total: '4.7 GB',
			remaining: '5:12',
			state: 'downloading' as const
		},
		{
			id: '3',
			filename: 'SF-Pro-Fonts.pkg',
			progress: 100,
			downloaded: '156 MB',
			total: '156 MB',
			state: 'completed' as const
		},
		{
			id: '4',
			filename: 'node-v22.0.0.pkg',
			progress: 30,
			downloaded: '24 MB',
			total: '80 MB',
			state: 'paused' as const
		},
		{
			id: '5',
			filename: 'docker-desktop.dmg',
			progress: 0,
			state: 'waiting' as const
		}
	]);

	// 根据 Tab 筛选下载列表
	const filteredDownloads = $derived(() => {
		switch (activeTab) {
			case 'active':
				return downloads.filter(d => d.state === 'downloading' || d.state === 'waiting');
			case 'paused':
				return downloads.filter(d => d.state === 'paused');
			case 'completed':
				return downloads.filter(d => d.state === 'completed');
			default:
				return downloads;
		}
	});

	function handleAddTask(urls: string[], savePath: string) {
		// 模拟添加新任务
		for (const url of urls) {
			const newTask = {
				id: String(downloads.length + 1),
				filename: url.split('/').pop() || 'new-download',
				progress: 0,
				state: 'waiting' as const
			};
			downloads.push(newTask);
		}
	}
</script>

<div class="downloads-page">
	<header class="page-header">
		<div class="title-group">
			<h1>Mua</h1>
			<p class="subtitle">管理您的下载任务</p>
		</div>
		<HeaderToolbar 
			{activeTab}
			onTabChange={(tab) => activeTab = tab}
			onAddClick={() => showAddDialog = true}
		/>
	</header>

	<section class="downloads-list">
		{#each filteredDownloads() as download (download.id)}
			<DownloadCard
				filename={download.filename}
				progress={download.progress}
				speed={download.speed}
				downloaded={download.downloaded}
				total={download.total}
				remaining={download.remaining}
				state={download.state}
			/>
		{/each}
	</section>

	{#if filteredDownloads().length === 0}
		<div class="empty-state">
			<p>暂无下载任务</p>
			<p class="hint">点击 + 按钮添加新下载</p>
		</div>
	{/if}
</div>

<!-- 添加任务对话框 -->
<AddTaskDialog 
	open={showAddDialog}
	onClose={() => showAddDialog = false}
	onSubmit={handleAddTask}
/>

<style>
	.downloads-page {
		width: 100%;
	}

	.page-header {
		display: flex;
		align-items: flex-end;
		justify-content: space-between;
		margin-bottom: 24px;
		gap: 16px;
	}

	.title-group {
		display: flex;
		flex-direction: column;
	}

	h1 {
		font-size: 28px;
		font-weight: 600;
		color: rgba(255, 255, 255, 0.95);
		margin: 0;
		line-height: 1.2;
	}

	.subtitle {
		font-size: 14px;
		color: rgba(255, 255, 255, 0.5);
		margin-top: 4px;
	}

	.downloads-list {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: 80px 20px;
		text-align: center;
		color: rgba(255, 255, 255, 0.4);
	}

	.empty-state .hint {
		font-size: 13px;
		margin-top: 8px;
		color: rgba(255, 255, 255, 0.3);
	}
</style>

