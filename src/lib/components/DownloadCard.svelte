<!--
  DownloadCard.svelte
  下载任务卡片 - 玻璃效果
-->
<script lang="ts">
	import { Pause, Play, X, MoreVertical, File, CheckCircle, AlertCircle } from '@lucide/svelte';
	import ProgressBar from './ProgressBar.svelte';

	type DownloadState = 'downloading' | 'paused' | 'completed' | 'error' | 'waiting';

	interface Props {
		filename: string;
		progress: number;
		speed?: string;
		downloaded?: string;
		total?: string;
		remaining?: string;
		state?: DownloadState;
		onPause?: () => void;
		onResume?: () => void;
		onCancel?: () => void;
	}

	let {
		filename,
		progress,
		speed = '',
		downloaded = '',
		total = '',
		remaining = '',
		state = 'downloading',
		onPause,
		onResume,
		onCancel
	}: Props = $props();

	const stateColors = {
		downloading: 'text-emerald-400',
		paused: 'text-yellow-400',
		completed: 'text-emerald-400',
		error: 'text-red-400',
		waiting: 'text-zinc-400'
	};
</script>

<article class="download-card">
	<div class="card-header">
		<!-- 文件图标和名称 -->
		<div class="file-info">
			{#if state === 'completed'}
				<CheckCircle size={20} class="text-emerald-400" />
			{:else if state === 'error'}
				<AlertCircle size={20} class="text-red-400" />
			{:else}
				<File size={20} class="text-zinc-400" />
			{/if}
			<span class="filename" title={filename}>{filename}</span>
		</div>

		<!-- 操作按钮 -->
		<div class="actions">
			{#if state === 'downloading'}
				<button class="action-btn" onclick={() => onPause?.()} title="暂停">
					<Pause size={16} />
				</button>
			{:else if state === 'paused' || state === 'waiting'}
				<button class="action-btn" onclick={() => onResume?.()} title="继续">
					<Play size={16} />
				</button>
			{/if}
			{#if state !== 'completed'}
				<button class="action-btn cancel" onclick={() => onCancel?.()} title="取消">
					<X size={16} />
				</button>
			{/if}
			<button class="action-btn" title="更多">
				<MoreVertical size={16} />
			</button>
		</div>
	</div>

	<!-- 进度条 -->
	{#if state !== 'completed'}
		<ProgressBar {progress} {state} />
	{/if}

	<!-- 状态信息 -->
	<div class="card-footer">
		{#if state === 'downloading'}
			<span class="speed">{speed}</span>
			<span class="size">{downloaded} / {total}</span>
			{#if remaining}
				<span class="remaining">剩余 {remaining}</span>
			{/if}
		{:else if state === 'paused'}
			<span class={stateColors[state]}>已暂停</span>
			<span class="size">{downloaded} / {total}</span>
		{:else if state === 'completed'}
			<span class={stateColors[state]}>已完成</span>
			<span class="size">{total}</span>
		{:else if state === 'waiting'}
			<span class={stateColors[state]}>等待中...</span>
		{:else if state === 'error'}
			<span class={stateColors[state]}>下载失败</span>
		{/if}
	</div>
</article>

<style>
	.download-card {
		background: rgba(255, 255, 255, 0.05);
		border: 1px solid rgba(255, 255, 255, 0.08);
		border-radius: 12px;
		padding: 16px;
		transition: all 0.2s ease;
	}

	.download-card:hover {
		background: rgba(255, 255, 255, 0.08);
		border-color: rgba(255, 255, 255, 0.12);
	}

	.card-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 12px;
	}

	.file-info {
		display: flex;
		align-items: center;
		gap: 10px;
		flex: 1;
		min-width: 0;
	}

	.filename {
		font-size: 14px;
		font-weight: 500;
		color: rgba(255, 255, 255, 0.9);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.actions {
		display: flex;
		gap: 4px;
		opacity: 0;
		transition: opacity 0.2s ease;
	}

	.download-card:hover .actions {
		opacity: 1;
	}

	.action-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		background: rgba(255, 255, 255, 0.1);
		border: none;
		border-radius: 6px;
		color: rgba(255, 255, 255, 0.7);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.action-btn:hover {
		background: rgba(255, 255, 255, 0.15);
		color: white;
	}

	.action-btn.cancel:hover {
		background: rgba(239, 68, 68, 0.2);
		color: rgb(248, 113, 113);
	}

	.card-footer {
		display: flex;
		align-items: center;
		gap: 16px;
		margin-top: 10px;
		font-size: 12px;
		color: rgba(255, 255, 255, 0.5);
	}

	.speed {
		color: rgb(52, 211, 153);
		font-weight: 500;
	}

	.remaining {
		margin-left: auto;
	}
</style>
