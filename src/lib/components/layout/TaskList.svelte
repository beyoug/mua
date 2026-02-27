<!--
  TaskList.svelte
  任务列表组件 - 渲染下载任务卡片列表和空状态
-->
<script lang="ts">
	import { Download } from '@lucide/svelte';
	import { flip } from 'svelte/animate';
	import DownloadCard from '$lib/components/download/DownloadCard.svelte';
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
		onCancel?: (task: DownloadTask) => void;
		onOpenFolder?: (id: string) => void;
		onShowDetails?: (task: DownloadTask) => void;
		groupByDate?: boolean;
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
		onCancel,
		onOpenFolder,
		onShowDetails,
		groupByDate = false
	}: Props = $props();

	const dateLabelCache = new Map<string, string>();
	function getCachedTaskDate(dateStr: string): string {
		if (!dateStr) return '';
		const cached = dateLabelCache.get(dateStr);
		if (cached !== undefined) return cached;
		const computed = computeTaskDate(dateStr);
		if (dateLabelCache.size > 5000) dateLabelCache.clear();
		dateLabelCache.set(dateStr, computed);
		return computed;
	}

	const taskDateLabels = $derived.by(() => {
		if (!groupByDate) return [] as string[];
		return tasks.map(task => getCachedTaskDate(task.addedAt));
	});
	// Helper to format date for grouping
	// Helper to format date for grouping with natural language
	function computeTaskDate(dateStr: string): string {
		if (!dateStr) return '';
		try {
			const date = new Date(dateStr);
			const now = new Date();
			
			// Reset time part for accurate date comparison
			const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
			const target = new Date(date.getFullYear(), date.getMonth(), date.getDate());
			
			const diffTime = today.getTime() - target.getTime();
			const diffDays = Math.round(diffTime / (1000 * 60 * 60 * 24));

			if (diffDays === 0) return '今天';
			if (diffDays === 1) return '昨天';
			if (diffDays === 2) return '前天';
			
			// Format: "M月D日 周X"
			const weekDays = ['周日', '周一', '周二', '周三', '周四', '周五', '周六'];
			const weekDay = weekDays[date.getDay()];
			
			if (today.getFullYear() === target.getFullYear()) {
				return `${date.getMonth() + 1}月${date.getDate()}日 ${weekDay}`;
			}
			
			return `${date.getFullYear()}年${date.getMonth() + 1}月${date.getDate()}日`;
		} catch (e) {
			return '';
		}
	}
</script>

<div class="scroll-container">
{#if tasks.length > 0}
	<section class="downloads-list">
		{#if tasks.length <= 80}
			{#each tasks as download, i (download.id)}
				{@const currentDate = groupByDate ? (taskDateLabels[i] || '') : ''}
				{@const prevDate = groupByDate && i > 0 ? (taskDateLabels[i - 1] || '') : ''}
				
				<div animate:flip={{ duration: 400 }}>
					{#if groupByDate && currentDate && currentDate !== prevDate}
						<div class="date-divider">
							<span>{currentDate}</span>
							<div class="line"></div>
						</div>
					{/if}

					<DownloadCard
						task={download}
						selectionMode={isSelectionMode}
						selected={selectedIds.has(download.id)}
						onSelect={() => onSelect?.(download.id)}
						onPause={() => onPause?.(download.id)}
						onResume={() => onResume?.(download.id)}
						onCancel={() => onCancel?.(download)}
						onOpenFolder={() => onOpenFolder?.(download.id)}
						onShowDetails={() => onShowDetails?.(download)}
					/>
				</div>
			{/each}
		{:else}
			{#each tasks as download, i (download.id)}
				{@const currentDate = groupByDate ? (taskDateLabels[i] || '') : ''}
				{@const prevDate = groupByDate && i > 0 ? (taskDateLabels[i - 1] || '') : ''}
				
				<div>
					{#if groupByDate && currentDate && currentDate !== prevDate}
						<div class="date-divider">
							<span>{currentDate}</span>
							<div class="line"></div>
						</div>
					{/if}

					<DownloadCard
						task={download}
						selectionMode={isSelectionMode}
						selected={selectedIds.has(download.id)}
						onSelect={() => onSelect?.(download.id)}
						onPause={() => onPause?.(download.id)}
						onResume={() => onResume?.(download.id)}
						onCancel={() => onCancel?.(download)}
						onOpenFolder={() => onOpenFolder?.(download.id)}
						onShowDetails={() => onShowDetails?.(download)}
					/>
				</div>
			{/each}
		{/if}
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
		overflow-x: hidden; /* Prevent horizontal scroll, force children to fit */
		display: flex;
		flex-direction: column;
        
        /* 自定义滚动条样式 */
        scrollbar-width: thin;
        scrollbar-color: var(--border-subtle) transparent;
	}

    .scroll-container::-webkit-scrollbar {
        width: 6px;
    }

    .scroll-container::-webkit-scrollbar-track {
        background: transparent;
    }

    .scroll-container::-webkit-scrollbar-thumb {
        background: var(--border-subtle);
        border-radius: 10px;
    }

    .scroll-container::-webkit-scrollbar-thumb:hover {
        background: var(--border-normal);
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
		background: var(--surface-hover);
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

	.date-divider {
		display: flex;
		align-items: center;
		gap: 12px;
		margin-top: 32px; /* Increased spacing for better separation */
		margin-bottom: 12px;
		width: 100%;
		padding-left: 4px; /* Slight indent */
	}
	
	/* Remove margin for the very first divider if it's at the top */
	.date-divider:first-child {
		margin-top: 0;
	}

	.date-divider span {
		font-size: 13px;
		font-weight: 600; /* Slightly bolder */
		color: var(--text-primary); /* Darker text for better readability */
		opacity: 0.8;
	}

	.date-divider .line {
		flex: 1;
		height: 1px;
		background: linear-gradient(to right, var(--border-color) 0%, transparent 100%); /* Fade out line */
		opacity: 0.6;
	}
</style>
