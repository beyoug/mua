<!--
  SettingsPanel.svelte
  浮动设置面板 - 主题色 + 颜色模式 + 粒子设置
-->
<script lang="ts">
	import { X, Palette, Check, Sun, Moon, Monitor, Zap, Sparkles } from '@lucide/svelte';
	import { fade, scale } from 'svelte/transition';
	import { currentTheme, themes, colorMode, colorModes, type ThemeId, type ColorMode, particlesEnabled } from '$lib/stores/theme';
	import { totalDownloadSpeed } from '$lib/stores/downloadSpeed';
	import { createScrollLockEffect } from '$lib';
	import { getEmitRate, getEstimatedParticles } from '$lib/utils/particles';

	interface Props {
		open: boolean;
		onClose: () => void;
	}

	let { open, onClose }: Props = $props();
	let testSpeed = $state(0); // 测试速度 MB/s

	function selectTheme(themeId: ThemeId) {
		currentTheme.set(themeId);
	}

	function selectColorMode(mode: ColorMode) {
		colorMode.set(mode);
	}

	function toggleParticles() {
		particlesEnabled.toggle();
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			onClose();
		}
	}

	function updateTestSpeed(e: Event) {
		const value = (e.target as HTMLInputElement).valueAsNumber;
		testSpeed = value;
		totalDownloadSpeed.set(value * 1024 * 1024);
	}

	const themeList = Object.values(themes);

	const modeIcons = {
		'dark': Moon,
		'light': Sun,
		'auto': Monitor
	};

	// 使用统一的滚动锁定工具
	$effect(() => {
		return createScrollLockEffect(open);
	});
</script>

{#if open}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="panel-overlay" 
		in:fade={{ duration: 150 }} 
		out:fade={{ duration: 100 }}
		onclick={onClose} 
		onkeydown={handleKeydown}>
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div class="panel" 
			in:scale={{ duration: 150, start: 0.98, opacity: 0.5 }}
			out:fade={{ duration: 80 }}
			onclick={(e) => e.stopPropagation()}>
			<header class="panel-header">
				<h2>设置</h2>
				<button class="close-btn" onclick={onClose}>
					<X size={18} />
				</button>
			</header>

			<div class="panel-body">
				<!-- 主题选择 -->
				<section class="settings-section">
					<div class="section-header">
						<Palette size={16} />
						<span>主题颜色</span>
					</div>
					
					<div class="theme-grid">
						{#each themeList as theme}
							<button
								class="theme-card"
								class:active={$currentTheme === theme.id}
								onclick={() => selectTheme(theme.id)}
								title={theme.name}
							>
								<div 
									class="theme-preview"
									style="background: linear-gradient(135deg, {theme.primary}, {theme.secondary})"
								>
									{#if $currentTheme === theme.id}
										<Check size={16} strokeWidth={3} />
									{/if}
								</div>
								<span class="theme-name">{theme.name}</span>
							</button>
						{/each}
					</div>
				</section>

				<!-- 颜色模式 -->
				<section class="settings-section">
					<div class="section-header">
						<Sun size={16} />
						<span>外观模式</span>
					</div>
					
					<div class="mode-grid">
						{#each colorModes as mode}
							{@const Icon = modeIcons[mode.id]}
							<button
								class="mode-card"
								class:active={$colorMode === mode.id}
								onclick={() => selectColorMode(mode.id)}
							>
								<Icon size={20} />
								<span>{mode.name}</span>
							</button>
						{/each}
					</div>
				</section>

				<!-- 粒子效果 -->
				<section class="settings-section">
					<div class="section-header">
						<Sparkles size={16} />
						<span>粒子效果</span>
					</div>
					
					<div class="toggle-row">
						<span class="toggle-label">背景粒子效果</span>
						<button 
							class="toggle-switch"
							class:active={$particlesEnabled}
							onclick={toggleParticles}
							aria-pressed={$particlesEnabled}
							aria-label="Toggle particle effects"
						>
							<span class="toggle-knob"></span>
						</button>
					</div>
				</section>

					<!-- 粒子调试 -->
				<section class="settings-section">
					<div class="section-header">
						<Zap size={16} />
						<span>粒子调试</span>
					</div>
					
					<div class="speed-slider">
						<label>
							<span>模拟速度: {testSpeed} MB/s</span>
							<input 
								type="range" 
								min="0" 
								max="100" 
								step="1"
								value={testSpeed}
								oninput={updateTestSpeed}
							/>
						</label>
						<div class="speed-info">
							<span>释放: {getEmitRate(testSpeed).toFixed(1)}/秒</span>
							<span>预估: ~{getEstimatedParticles(testSpeed)}个</span>
						</div>
					</div>
				</section>
			</div>
		</div>
	</div>
{/if}

<style>
	.panel-overlay {
		position: fixed;
		inset: 0;
		background: var(--dialog-overlay-bg, rgba(0, 0, 0, 0.2));
		backdrop-filter: blur(2px);
		z-index: 2000;
	}

	.panel {
		position: fixed;
		/* 与任务列表区域对齐：Sidebar (200px) + 左边距 (12px) + 间距 (12px) = 224px */
		left: 224px;
		top: 12px;
		right: 12px;
		bottom: 12px;
		background: var(--dialog-bg);
		backdrop-filter: var(--glass-blur) var(--glass-saturate);
		-webkit-backdrop-filter: var(--glass-blur) var(--glass-saturate);
		border: 1px solid var(--glass-border);
		border-radius: 16px;
		box-shadow: var(--glass-shadow);
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.panel-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 14px 18px;
		border-bottom: 1px solid var(--border-color);
	}

	.panel-header h2 {
		font-size: 15px;
		font-weight: 500;
		color: var(--text-primary);
		margin: 0;
	}

	.close-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		background: transparent;
		border: none;
		border-radius: 8px;
		color: var(--text-muted);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.close-btn:hover {
		background: var(--input-bg);
		color: var(--text-primary);
	}

	.panel-body {
		flex: 1;
		padding: 16px 18px;
		overflow-y: auto;
	}

	.settings-section {
		margin-bottom: 20px;
	}

	.section-header {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 12px;
		font-weight: 500;
		color: var(--text-secondary);
		margin-bottom: 10px;
		text-transform: uppercase;
		letter-spacing: 0.4px;
	}

	.theme-grid {
		display: flex;
		gap: 8px;
	}

	.theme-card {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 6px;
		background: var(--input-bg);
		border: 1px solid var(--border-normal);
		border-radius: 10px;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.theme-card:hover {
		background: var(--surface-active);
		border-color: var(--border-strong);
	}

	.theme-card.active {
		border-color: var(--accent-primary);
		background: var(--accent-active-bg);
	}

	.theme-preview {
		width: 32px;
		height: 32px;
		border-radius: 50%;
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--accent-btn-text, white);
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.25);
	}

	.theme-name {
		/* 隐藏文字，使用 title 属性提供提示 */
		display: none;
	}

	/* 颜色模式选择器 */
	.mode-grid {
		display: flex;
		gap: 8px;
	}

	.mode-card {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		padding: 8px 12px;
		background: var(--input-bg);
		border: 1px solid var(--border-normal);
		border-radius: 10px;
		color: var(--text-secondary);
		font-size: 12px;
		font-weight: 400;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.mode-card:hover {
		background: var(--surface-active);
		border-color: var(--border-strong);
		color: var(--text-primary);
	}

	.mode-card.active {
		border-color: var(--accent-primary);
		background: var(--accent-subtle);
		color: var(--accent-primary);
	}

	/* 速度滑块 */
	.speed-slider {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.speed-slider label {
		display: flex;
		flex-direction: column;
		gap: 8px;
		font-size: 13px;
		color: var(--text-secondary);
	}

	.speed-slider input[type="range"] {
		width: 100%;
		height: 6px;
		appearance: none;
		background: var(--border-color);
		border-radius: 3px;
		cursor: pointer;
	}

	.speed-slider input[type="range"]::-webkit-slider-thumb {
		appearance: none;
		width: 16px;
		height: 16px;
		background: var(--accent-primary);
		border-radius: 50%;
		box-shadow: 0 2px 6px var(--accent-glow);
	}

	.speed-info {
		display: flex;
		justify-content: space-between;
		font-size: 12px;
		color: var(--text-muted);
		padding: 8px 12px;
		background: var(--border-light);
		border-radius: 8px;
	}

	/* Toggle 开关 */
	.toggle-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 14px;
		background: var(--input-bg);
		border: 1px solid var(--border-subtle);
		border-radius: 10px;
	}

	.toggle-label {
		font-size: 13px;
		color: var(--text-secondary);
	}

	.toggle-switch {
		position: relative;
		width: 44px;
		height: 24px;
		background: var(--border-color);
		border: none;
		border-radius: 12px;
		cursor: pointer;
		transition: background 0.2s ease;
	}

	.toggle-switch.active {
		background: var(--accent-primary);
	}

	.toggle-knob {
		position: absolute;
		top: 2px;
		left: 2px;
		width: 20px;
		height: 20px;
		background: var(--toggle-knob-c, white);
		border-radius: 50%;
		box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
		transition: transform 0.2s ease;
	}

	.toggle-switch.active .toggle-knob {
		transform: translateX(20px);
	}
</style>
