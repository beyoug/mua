<!--
  ProgressBar.svelte
  进度条组件 - 渐变 + 条纹动画
-->
<script lang="ts">
	type ProgressState = 'downloading' | 'paused' | 'completed' | 'error' | 'waiting';

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
		background: rgba(255, 255, 255, 0.1);
		border-radius: 3px;
		overflow: hidden;
	}

	.progress-fill {
		height: 100%;
		border-radius: 3px;
		transition: width 0.3s ease;
		position: relative;
		overflow: hidden;
	}

	.progress-fill.downloading {
		background: linear-gradient(90deg, rgb(16, 185, 129), rgb(20, 184, 166));
	}

	.progress-fill.paused {
		background: rgb(234, 179, 8);
	}

	.progress-fill.error {
		background: rgb(239, 68, 68);
	}

	.stripes {
		position: absolute;
		inset: 0;
		background: repeating-linear-gradient(
			-45deg,
			transparent,
			transparent 8px,
			rgba(255, 255, 255, 0.15) 8px,
			rgba(255, 255, 255, 0.15) 16px
		);
		animation: stripe-move 0.8s linear infinite;
	}

	@keyframes stripe-move {
		from {
			transform: translateX(0);
		}
		to {
			transform: translateX(22.627px); /* 16px * sqrt(2) */
		}
	}

	.progress-percent {
		font-size: 12px;
		font-weight: 500;
		color: rgba(255, 255, 255, 0.7);
		min-width: 36px;
		text-align: right;
	}
</style>
