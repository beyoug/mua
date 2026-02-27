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
		state = 'active',
		showPercent = true
	}: Props = $props();

	const clampedProgress = $derived(Math.min(100, Math.max(0, progress)));
</script>

<div class="progress-container">
	<div class="progress-track">
		<div 
			class="progress-fill"
			class:downloading={state === 'active' || state === 'complete'}
			class:paused={state === 'paused'}
			class:error={state === 'error'}
			style="width: {clampedProgress}%"
		>
			{#if state === 'active' || state === 'complete'}
				<div class="stripes" class:moving={state === 'active'}></div>
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
		height: 4px;
		background: var(--progress-track-bg, var(--border-subtle));
		border-radius: 2px;
		overflow: hidden;
	}

	.progress-fill {
		height: 100%;
		border-radius: 2px;
		transition: width 0.1s linear;
		position: relative;
		overflow: hidden;
	}

	.progress-fill.downloading {
		background: linear-gradient(90deg, var(--accent-primary), var(--accent-secondary));
	}

	.progress-fill.paused {
		background: var(--semantic-warning);
	}

	.progress-fill.error {
		background: var(--semantic-danger);
	}

	.stripes {
		position: absolute;
		inset: 0;
		background: repeating-linear-gradient(
			-45deg,
			transparent,
			transparent 8px,
			transparent 8px,
			var(--progress-stripe-c, rgba(255, 255, 255, 0.12)) 8px,
			var(--progress-stripe-c, rgba(255, 255, 255, 0.12)) 16px
		);
	}

	.stripes.moving {
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
		font-size: 11px;
		font-weight: 500;
		color: var(--accent-text);
		min-width: 32px;
		text-align: right;
		opacity: 0.8;
		font-variant-numeric: tabular-nums;
	}
</style>
