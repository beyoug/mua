<!--
  TaskList.svelte
  任务列表组件 - 渲染下载任务卡片列表和空状态
-->
<script lang="ts">
	import { Download } from '@lucide/svelte';
	import { flip } from 'svelte/animate';
	import DownloadCard from './DownloadCard.svelte';
	import type { DownloadTask } from '$lib/types/download';

	interface Props {
		tasks: DownloadTask[];
		emptyTitle: string;
		emptyHint: string;
		isSelectionMode: boolean;
		selectedIds: Set<string>;
		onSelect?: (id: string) => void;
		onPause?: (id: string) => void;
		onResume?: (id: string) => void;
		onCancel?: (id: string) => void;
	}

	let {
		tasks,
		emptyTitle,
		emptyHint,
		isSelectionMode,
		selectedIds,
		onSelect,
		onPause,
		onResume,
		onCancel
	}: Props = $props();
</script>

<div class="scroll-container">
{#if tasks.length > 0}
	<section class="downloads-list">
		{#each tasks as download (download.id)}
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
					onSelect={() => onSelect?.(download.id)}
					onPause={() => onPause?.(download.id)}
					onResume={() => onResume?.(download.id)}
					onCancel={() => onCancel?.(download.id)}
					addedAt={download.addedAt}
				/>
			</div>
		{/each}
	</section>
{:else}
	<div class="empty-state">
		<div class="empty-icon">
			<div class="ripple"></div>
			<div class="ripple delay"></div>
			<Download size={48} strokeWidth={1.5} />
		</div>
		<p class="empty-title">{emptyTitle}</p>
		<p class="empty-hint">{emptyHint}</p>
	</div>
{/if}
</div>

<style>
	.scroll-container {
		flex: 1;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
	}

	.downloads-list {
		display: flex;
		flex-direction: column;
		gap: 12px;
		padding: 20px 24px;
	}

	.empty-state {
		flex: 1;
		width: 100%;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		color: var(--text-muted);
		padding: 40px 24px;
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
