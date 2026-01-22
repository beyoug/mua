<!--
  ProgressBar.svelte
  进度条组件 - 使用主题 CSS 变量 + 光晕效果
-->
<script lang="ts">
	import type { DownloadState } from '$lib/types/download';

	type ProgressState = DownloadState;

	interface Props {
		progress: number;
		state?: ProgressState;
		showPercent?: boolean;
	}

	let {
		progress,
		state = 'downloading',
		showPercent = true
	}: Props = $props();

	const clampedProgress = $derived(Math.min(100, Math.max(0, progress)));
</script>

<div class="progress-container">
	<div class="progress-track">
		<div 
			class="progress-fill"
			class:downloading={state === 'downloading'}
			class:paused={state === 'paused'}
			class:error={state === 'error'}
			style="width: {clampedProgress}%"
		>
			{#if state === 'downloading'}
				<div class="stripes"></div>
				<div class="glow"></div>
			{/if}
		</div>
	</div>
	{#if showPercent}
		<span class="progress-percent">{Math.round(clampedProgress)}%</span>
	{/if}
</div>

<style>
	.progress-container {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.progress-track {
		flex: 1;
		height: 6px;
		background: var(--border-light);
		border-radius: 3px;
		overflow: hidden;
		box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.2);
	}

	.progress-fill {
		height: 100%;
		border-radius: 3px;
		transition: width 0.3s ease;
		position: relative;
		overflow: hidden;
	}

	.progress-fill.downloading {
		background: linear-gradient(90deg, var(--accent-primary), var(--accent-secondary));
	}

	.progress-fill.paused {
		background: var(--warning-color);
	}

	.progress-fill.error {
		background: var(--danger-color);
	}

	.stripes {
		position: absolute;
		inset: 0;
		background: repeating-linear-gradient(
			-45deg,
			transparent,
			transparent 8px,
			rgba(255, 255, 255, 0.12) 8px,
			rgba(255, 255, 255, 0.12) 16px
		);
		animation: stripe-move 0.8s linear infinite;
	}

	.glow {
		position: absolute;
		right: 0;
		top: -4px;
		bottom: -4px;
		width: 20px;
		background: linear-gradient(90deg, transparent, var(--accent-glow));
		filter: blur(4px);
	}

	@keyframes stripe-move {
		from {
			transform: translateX(0);
		}
		to {
			transform: translateX(22.627px);
		}
	}

	.progress-percent {
		font-size: 12px;
		font-weight: 500;
		color: var(--accent-text);
		min-width: 36px;
		text-align: right;
	}
</style>
