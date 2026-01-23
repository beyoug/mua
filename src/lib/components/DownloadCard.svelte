<!--
  DownloadCard.svelte
  下载任务卡片 - 使用主题 CSS 变量 + 精致效果
-->
<script lang="ts">
	import { Pause, Play, X, MoreVertical, File, CheckCircle, AlertCircle, Check, Copy, Folder, Info, ExternalLink } from '@lucide/svelte';
	import { fade, scale } from 'svelte/transition';
	import { open } from '@tauri-apps/plugin-shell';
	import ProgressBar from './ProgressBar.svelte';
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
		addedAt = ''
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
			// TODO: Show toast
			closeMenu();
		} catch (e) {
			console.error('Failed to copy', e);
		}
	}

	async function openFolder() {
		// Mock opening folder for now, or use shell open if path known
		try {
			// 这里假设默认下载目录，实际上可能需要传入 savePath
			// 由于安全限制和路径获取复杂性，这里暂时只做演示或打开默认 Home
			await open(await  import('@tauri-apps/api/path').then(p => p.downloadDir())); 
			closeMenu();
		} catch (e) {
			console.error(e);
		}
	}

	function showDetails() {
		alert(`文件名: ${filename}\nURL: ${url}\n状态: ${downloadState}`);
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
				
				{#if showMenu}
					<!-- svelte-ignore a11y_click_events_have_key_events -->
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div class="menu-backdrop" onclick={closeMenu}></div>
					
					<div 
						class="dropdown-menu" 
						transition:scale={{ duration: 150, start: 0.95 }}
					>
						<button class="menu-item" onclick={copyUrl} disabled={!url}>
							<Copy size={14} />
							<span>复制链接</span>
						</button>
						<button class="menu-item" onclick={openFolder}>
							<Folder size={14} />
							<span>打开文件夹</span>
						</button>
						<button class="menu-item" onclick={showDetails}>
							<Info size={14} />
							<span>查看详情</span>
						</button>
						<div class="menu-divider"></div>
						<button class="menu-item danger" onclick={() => { onCancel?.(); closeMenu(); }}>
							<X size={14} />
							<span>{['completed', 'cancelled', 'error'].includes(downloadState) ? '删除任务' : '取消下载'}</span>
						</button>
					</div>
				{/if}
			</div>
		</div>
	</div>

	<!-- 进度条 -->
	{#if downloadState !== 'completed'}
		<ProgressBar {progress} state={downloadState} />
	{/if}

	<!-- 状态信息 -->
	<div class="card-footer">
		{#if downloadState === 'downloading'}
			<span class="speed">{speed}</span>
			<span class="size">{downloaded} / {total}</span>
			{#if remaining}
				<span class="remaining">剩余 {remaining}</span>
			{/if}
		{:else if downloadState === 'paused'}
			<span class="status paused">已暂停</span>
			<span class="size">{downloaded} / {total}</span>
		{:else if downloadState === 'completed'}
			<span class="status completed">已完成</span>
			<span class="size">{total}</span>
		{:else if downloadState === 'waiting'}
			<span class="status waiting">等待中...</span>
		{:else if downloadState === 'cancelled'}
			<span class="status cancelled">已取消</span>
		{:else if downloadState === 'error'}
			<span class="status error">下载失败</span>
		{/if}
		
		{#if addedAt}
			<span class="added-at">{addedAt}</span>
		{/if}
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
		border-color: var(--accent-primary);
		transform: translateY(-1px);
		box-shadow: 
			0 4px 12px rgba(0, 0, 0, 0.1),
			0 12px 32px rgba(0, 0, 0, 0.15),
			0 0 0 1px var(--accent-primary);
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

	/* 下拉菜单样式 */
	.menu-container {
		position: relative;
	}

	.menu-backdrop {
		position: fixed;
		inset: 0;
		z-index: 90;
		cursor: default;
	}

	.dropdown-menu {
		position: absolute;
		top: calc(100% + 4px);
		right: 0;
		width: 140px; /* 稍微窄一点 */
		background: var(--glass-menu-bg); /* 65% 透明度 */
		backdrop-filter: blur(20px);
		-webkit-backdrop-filter: blur(20px);
		border: 1px solid var(--border-strong);
		border-radius: 12px;
		box-shadow: 
			0 4px 6px -1px rgba(0, 0, 0, 0.1),
			0 10px 15px -3px rgba(0, 0, 0, 0.1),
			0 0 0 1px rgba(255, 255, 255, 0.05); /* 更加立体的阴影 */
		padding: 4px;
		display: flex;
		flex-direction: column;
		gap: 2px;
		z-index: 100;
		transform-origin: top right;
	}

	.menu-item {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 8px 10px;
		background: transparent;
		border: none;
		border-radius: 8px;
		color: var(--text-primary);
		font-size: 13px;
		cursor: pointer;
		text-align: left;
		transition: all 0.15s ease;
	}

	.menu-item:hover {
		background: var(--surface-active);
		color: var(--accent-primary);
	}

	.menu-item:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.menu-item.danger {
		color: var(--danger-color);
	}

	.menu-item.danger:hover {
		background: rgba(239, 68, 68, 0.1);
	}

	.menu-divider {
		height: 1px;
		background: var(--border-subtle);
		margin: 4px 0;
	}
</style>
