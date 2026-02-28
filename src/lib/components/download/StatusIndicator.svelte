<script lang="ts">
	import type { DownloadState, SpeedInfo } from '$lib/types/download';

	interface Props {
		state: DownloadState;
		speed?: SpeedInfo;
		remaining?: string;
		errorMessage?: string;
	}

	let { state, speed = { value: '0', unit: 'B/s' }, remaining = '', errorMessage = '' }: Props = $props();
</script>

{#if state === 'active'}
	<span class="status-indicator active">
		<span class="status-icon">↓</span>
		<span class="speed-num">{speed.value}</span>
		<span class="speed-unit-text">{speed.unit}</span>
	</span>
	{#if remaining}
		<span class="separator">·</span>
		<span class="time-remaining">{remaining}</span>
	{/if}
{:else if state === 'paused'}
	<span class="status-indicator paused">
		<span class="status-icon">⏸</span>
		<span class="status-text">已暂停</span>
	</span>
{:else if state === 'complete'}
	<span class="status-indicator complete">
		<span class="status-icon">✓</span>
		<span class="status-text">已完成</span>
	</span>
{:else if state === 'waiting'}
	<span class="status-indicator waiting">
		<span class="status-icon">◦</span>
		<span class="status-text">等待中</span>
	</span>
{:else if state === 'removed'}
	<span class="status-indicator removed">
		<span class="status-icon">✕</span>
		<span class="status-text">已取消</span>
	</span>
{:else if state === 'error'}
	<span class="status-indicator error">
		<span class="status-icon">⚠</span>
		<span class="status-text">下载失败</span>
	</span>
	{#if errorMessage}
		<span class="separator">·</span>
		<span class="error-inline" title={errorMessage}>
			{errorMessage}
		</span>
	{/if}
{:else if state === 'missing'}
	<span class="status-indicator missing">
		<span class="status-icon">⚠</span>
		<span class="status-text">本地文件不存在</span>
	</span>
{/if}

<style>
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

	.status-indicator.active {
		color: var(--accent-text);
	}

	.status-indicator.paused {
		color: var(--semantic-warning);
	}

	.status-indicator.complete {
		color: var(--semantic-success);
	}

	.status-indicator.waiting {
		color: var(--text-muted);
	}

	.status-indicator.removed {
		color: var(--text-muted);
	}

	.status-indicator.error {
		color: var(--semantic-danger);
	}

	.status-indicator.missing {
		color: var(--semantic-warning);
		opacity: 0.85;
	}

	.speed-num {
		font-weight: 600;
		display: inline-block;
		text-align: right;
		min-width: 3.2em;
		font-variant-numeric: tabular-nums;
	}

	.speed-unit-text {
		font-weight: 600;
		margin-left: 2px;
		display: inline-block;
		color: var(--text-secondary);
		opacity: 0.9;
	}

	.separator {
		color: var(--text-muted);
		opacity: 0.5;
		margin: 0 6px;
		flex-shrink: 0;
	}

	.time-remaining {
		justify-content: flex-start;
		font-variant-numeric: tabular-nums;
	}

	.status-text {
		color: inherit;
	}

	.error-inline {
		color: var(--semantic-danger);
		font-size: 11px;
		opacity: 0.9;
		font-family: var(--font-base);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		flex: 1;
		min-width: 0;
	}
</style>
