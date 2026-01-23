<!--
  DownloadCard.svelte
  下载任务卡片 - 使用主题 CSS 变量 + 精致效果
-->
<script lang="ts">
	import { Pause, Play, X, MoreVertical, File, CheckCircle, AlertCircle, Check } from '@lucide/svelte';
	import { fade } from 'svelte/transition';
	import ProgressBar from './ProgressBar.svelte';
	import type { DownloadState } from '$lib/types/download';

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
		selectionMode?: boolean;
		selected?: boolean;
		onSelect?: () => void;
		addedAt?: string;
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
		onCancel,
		selectionMode = false,
		selected = false,
		onSelect,
		addedAt = ''
	}: Props = $props();
</script>

<article class="download-card" class:completed={state === 'completed'}>
	<div class="card-header">
		{#if selectionMode}
			<div class="checkbox-wrapper" transition:fade={{ duration: 150 }}>
				<button 
					class="checkbox" 
					class:checked={selected} 
					onclick={(e) => { e.stopPropagation(); onSelect?.(); }}
					aria-label="Select task"
				>
					{#if selected}
						<Check size={12} strokeWidth={3} color="white" />
					{/if}
				</button>
			</div>
		{/if}
		<!-- 文件图标和名称 -->
		<div class="file-info">
			{#if state === 'completed'}
				<span class="icon-wrapper completed">
					<CheckCircle size={18} />
				</span>
			{:else if state === 'error'}
				<span class="icon-wrapper error">
					<AlertCircle size={18} />
				</span>
			{:else if state === 'cancelled'}
				<span class="icon-wrapper cancelled">
					<X size={18} />
				</span>
			{:else if state === 'downloading'}
				<span class="icon-wrapper active">
					<File size={18} />
				</span>
			{:else}
				<span class="icon-wrapper">
					<File size={18} />
				</span>
			{/if}
			<span class="filename" title={filename}>{filename}</span>
		</div>

		<!-- 操作按钮 -->
		<div class="actions">
			{#if state === 'downloading'}
				<button class="action-btn" onclick={() => onPause?.()} title="暂停">
					<Pause size={15} />
				</button>
			{:else if state === 'paused' || state === 'waiting' || state === 'cancelled'}
				<button class="action-btn resume" onclick={() => onResume?.()} title="继续">
					<Play size={15} />
				</button>
			{/if}
			{#if state !== 'completed'}
				<button class="action-btn cancel" onclick={() => onCancel?.()} title="取消">
					<X size={15} />
				</button>
			{/if}
			<button class="action-btn" title="更多">
				<MoreVertical size={15} />
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
			<span class="status paused">已暂停</span>
			<span class="size">{downloaded} / {total}</span>
		{:else if state === 'completed'}
			<span class="status completed">已完成</span>
			<span class="size">{total}</span>
		{:else if state === 'waiting'}
			<span class="status waiting">等待中...</span>
		{:else if state === 'cancelled'}
			<span class="status cancelled">已取消</span>
		{:else if state === 'error'}
			<span class="status error">下载失败</span>
		{/if}
		
		{#if addedAt}
			<span class="added-at">{addedAt}</span>
		{/if}
	</div>
</article>

<style>
	.download-card {
		background: var(--glass-bg);
		backdrop-filter: var(--glass-blur) var(--glass-saturate);
		-webkit-backdrop-filter: var(--glass-blur) var(--glass-saturate);
		border: 1px solid var(--glass-border);
		border-radius: 14px;
		padding: 14px 16px;
		transition: 
			transform 0.25s cubic-bezier(0.4, 0, 0.2, 1),
			box-shadow 0.25s cubic-bezier(0.4, 0, 0.2, 1),
			border-color 0.15s ease;
		box-shadow: var(--glass-shadow);
	}

	.download-card:hover {
		border-color: var(--accent-primary);
		transform: translateY(-1px);
		box-shadow: 
			0 4px 12px rgba(0, 0, 0, 0.1),
			0 12px 32px rgba(0, 0, 0, 0.15),
			0 0 0 1px var(--accent-primary);
	}

	.download-card.completed {
		border-left: 3px solid var(--accent-primary);
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
		gap: 12px;
		flex: 1;
		min-width: 0;
	}

	.icon-wrapper {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 36px;
		height: 36px;
		background: var(--border-light);
		border-radius: 10px;
		color: var(--text-muted);
	}

	.icon-wrapper.active {
		background: var(--accent-active-bg);
		color: var(--accent-text);
	}

	.icon-wrapper.completed {
		background: var(--accent-active-bg);
		color: var(--accent-text);
	}

	.icon-wrapper.error {
		background: rgba(239, 68, 68, 0.15);
		color: #f87171;
	}

	.icon-wrapper.cancelled {
		background: var(--bg-hover);
		color: var(--text-muted);
	}

	.filename {
		font-size: 13px;
		font-weight: 500;
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		letter-spacing: -0.01em;
	}

	.actions {
		display: flex;
		gap: 6px;
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
		width: 30px;
		height: 30px;
		background: var(--surface-hover);
		border: 1px solid var(--border-subtle);
		border-radius: 8px;
		color: var(--text-secondary);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.action-btn:hover {
		background: var(--surface-active);
		border-color: var(--border-normal);
		color: var(--text-primary);
		transform: scale(1.05);
	}

	.action-btn.resume:hover {
		background: var(--accent-active-bg);
		color: var(--accent-text);
		border-color: var(--accent-primary);
	}

	.action-btn.cancel:hover {
		background: rgba(239, 68, 68, 0.15);
		color: #f87171;
		border-color: rgba(239, 68, 68, 0.3);
	}

	.card-footer {
		display: flex;
		align-items: center;
		gap: 14px;
		margin-top: 10px;
		font-size: 11px;
		font-weight: 400;
		color: var(--text-muted);
	}

	.speed {
		color: var(--text-primary);
		font-weight: 500;
	}

	.status.completed {
		color: var(--accent-text);
	}

	.status.paused {
		color: var(--warning-color);
	}

	.status.waiting {
		color: var(--text-muted);
	}

	.status.error {
		color: #f87171;
	}

	.status.cancelled {
		color: var(--text-muted);
	}

	.remaining {
		color: var(--text-muted);
	}
	
	.added-at {
		margin-left: auto;
		font-size: 12px;
		color: var(--text-muted);
		opacity: 0.7;
		font-family: var(--font-mono);
	}

	.checkbox-wrapper {
		display: flex;
		align-items: center;
		margin-right: 12px;
	}

	.checkbox {
		width: 20px;
		height: 20px;
		border-radius: 6px;
		border: 1px solid var(--border-color);
		background: var(--bg-hover);
		display: flex;
		align-items: center;
		justify-content: center;
		cursor: pointer;
		padding: 0;
		transition: all 0.2s ease;
	}

	.checkbox:hover {
		border-color: var(--text-muted);
	}

	.checkbox.checked {
		background: var(--accent-primary);
		border-color: var(--accent-primary);
	}
</style>
