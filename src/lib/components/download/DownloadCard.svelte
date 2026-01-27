<!--
  DownloadCard.svelte
  下载任务卡片 - 使用主题 CSS 变量 + 精致效果
-->
<script lang="ts">
	import { Pause, Play, X, MoreVertical, File, CheckCircle, AlertCircle, Check } from '@lucide/svelte';
	import { fade } from 'svelte/transition';
	import ProgressBar from './ProgressBar.svelte';
    import DownloadCardMenu from './DownloadCardMenu.svelte';
	import type { DownloadState } from '$lib/types/download';

	interface Props {
		filename: string;
		url?: string;
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
		onOpenFolder?: () => void;
		onShowDetails?: () => void;
	}

	let {
		filename,
		url = '',
		progress,
		speed = '',
		downloaded = '',
		total = '',
		remaining = '',
		state: downloadState = 'downloading',
		onPause,
		onResume,
		onCancel,
		selectionMode = false,
		selected = false,
		onSelect,
		addedAt = '',
		onOpenFolder,
		onShowDetails
	}: Props = $props();

	let showMenu = $state(false);

	function toggleMenu(e: MouseEvent) {
		e.stopPropagation();
		showMenu = !showMenu;
	}

	function closeMenu() {
		showMenu = false;
	}

	async function copyUrl() {
		if (!url) return;
		try {
			await navigator.clipboard.writeText(url);
			closeMenu();
		} catch (e) {
			console.error('Failed to copy', e);
		}
	}

	function openFolder() {
		// Delegate to parent
		onOpenFolder?.();
		closeMenu();
	}

	function showDetails() {
		onShowDetails?.();
		closeMenu();
	}
</script>

<article class="download-card" class:completed={downloadState === 'completed'} class:menu-open={showMenu}>
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
			{#if downloadState === 'completed'}
				<span class="icon-wrapper completed">
					<CheckCircle size={18} />
				</span>
			{:else if downloadState === 'error'}
				<span class="icon-wrapper error">
					<AlertCircle size={18} />
				</span>
			{:else if downloadState === 'cancelled'}
				<span class="icon-wrapper cancelled">
					<X size={18} />
				</span>
			{:else if downloadState === 'downloading'}
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
			{#if downloadState === 'downloading' || downloadState === 'waiting'}
				<button class="action-btn" onclick={() => onPause?.()} title="暂停">
					<Pause size={15} />
				</button>
			{:else if downloadState === 'paused' || downloadState === 'cancelled'}
				<button class="action-btn resume" onclick={() => onResume?.()} title="继续">
					<Play size={15} />
				</button>
			{/if}
			{#if downloadState !== 'completed'}
				<button class="action-btn cancel" onclick={() => onCancel?.()} title="取消">
					<X size={15} />
				</button>
			{/if}
			<div class="menu-container" onmouseleave={closeMenu} role="presentation">
				<button class="action-btn" title="更多" onclick={toggleMenu}>
					<MoreVertical size={15} />
				</button>
				
                <DownloadCardMenu 
                    show={showMenu}
                    {downloadState}
                    {url}
                    onClose={closeMenu}
                    onCopy={copyUrl}
                    onOpenFolder={openFolder}
                    onDetails={showDetails}
                    onCancelOrDelete={() => { onCancel?.(); closeMenu(); }}
                />
			</div>
		</div>
	</div>

	<!-- 进度条 -->
	{#if downloadState !== 'completed'}
		<ProgressBar {progress} state={downloadState} />
	{/if}

	<!-- 状态信息栏 - 现代化设计 -->
	<div class="card-footer">
		<!-- 左区域：动态状态信息 -->
		<div class="footer-status">
			{#if downloadState === 'downloading'}
				<span class="status-indicator downloading">
					<span class="status-icon">↓</span>
					<span class="speed-value">{speed || '0 B/s'}</span>
				</span>
				{#if remaining}
					<span class="separator">•</span>
					<span class="time-remaining">{remaining}</span>
				{/if}
			{:else if downloadState === 'paused'}
				<span class="status-indicator paused">
					<span class="status-icon">⏸</span>
					<span class="status-text">已暂停</span>
				</span>
			{:else if downloadState === 'completed'}
				<span class="status-indicator completed">
					<span class="status-icon">✓</span>
					<span class="status-text">已完成</span>
				</span>
			{:else if downloadState === 'waiting'}
				<span class="status-indicator waiting">
					<span class="status-icon">◦</span>
					<span class="status-text">等待中</span>
				</span>
			{:else if downloadState === 'cancelled'}
				<span class="status-indicator cancelled">
					<span class="status-icon">✕</span>
					<span class="status-text">已取消</span>
				</span>
			{:else if downloadState === 'error'}
				<span class="status-indicator error">
					<span class="status-icon">⚠</span>
					<span class="status-text">下载失败</span>
				</span>
			{/if}
		</div>
		
		<!-- 右区域：文件大小和时间 -->
		<div class="footer-meta">
			{#if downloadState === 'downloading' || downloadState === 'paused'}
				<span class="size-info">{downloaded} / {total}</span>
			{:else if downloadState === 'completed'}
				<span class="size-info">{total}</span>
			{/if}
			{#if addedAt}
				<span class="time-added">{addedAt}</span>
			{/if}
		</div>
	</div>
</article>

<style>
	.download-card {
		position: relative;
		background: var(--glass-bg);
		backdrop-filter: var(--glass-blur) var(--glass-saturate);
		-webkit-backdrop-filter: var(--glass-blur) var(--glass-saturate);
		border: 1px solid var(--glass-border);
		border-radius: 14px;
		padding: 14px 16px;
		transition: 
			transform 0.25s cubic-bezier(0.4, 0, 0.2, 1),
			box-shadow 0.25s cubic-bezier(0.4, 0, 0.2, 1),
			border-color 0.15s ease,
			z-index 0s;
		box-shadow: var(--glass-shadow);
		z-index: 1;
	}

	.download-card.menu-open {
		z-index: 100 !important;
		border-color: var(--accent-primary);
	}

	.download-card:hover {
		transform: translateY(-1px);
		box-shadow: 
			0 4px 12px rgba(0, 0, 0, 0.1),
			0 12px 32px rgba(0, 0, 0, 0.15),
			0 0 8px var(--accent-glow);
		z-index: 10;
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
		border-color: var(--accent-primary);
		color: var(--text-primary);
		transform: scale(1.05);
		box-shadow: 0 0 8px var(--accent-glow);
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
		justify-content: space-between;
		margin-top: 10px;
		font-size: 11px;
		font-variant-numeric: tabular-nums;
	}

	/* 左区域 - 状态信息 */
	.footer-status {
		display: flex;
		align-items: center;
		gap: 6px;
		min-width: 0;
	}

	.status-indicator {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		font-weight: 500;
	}

	.status-icon {
		font-size: 12px;
		line-height: 1;
	}

	.status-indicator.downloading {
		color: var(--accent-text);
	}

	.status-indicator.paused {
		color: var(--warning-color, #f59e0b);
	}

	.status-indicator.completed {
		color: var(--semantic-success, #10b981);
	}

	.status-indicator.waiting {
		color: var(--text-muted);
	}

	.status-indicator.cancelled {
		color: var(--text-muted);
	}

	.status-indicator.error {
		color: var(--semantic-danger, #ef4444);
	}

	.speed-value {
		font-weight: 600;
		min-width: 70px;
	}

	.separator {
		color: var(--text-muted);
		opacity: 0.5;
	}

	.time-remaining {
		color: var(--text-secondary);
	}

	.status-text {
		color: inherit;
	}

	/* 右区域 - 文件信息 */
	.footer-meta {
		display: flex;
		align-items: center;
		gap: 12px;
		flex-shrink: 0;
	}

	.size-info {
		color: var(--text-secondary);
		min-width: 110px;
		text-align: right;
	}

	.time-added {
		font-size: 10px;
		color: var(--text-muted);
		opacity: 0.7;
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

	/* 下拉菜单样式 */
	.menu-container {
		position: relative;
	}
</style>
