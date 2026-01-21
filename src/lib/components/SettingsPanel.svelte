<!--
  SettingsPanel.svelte
  滑出式设置面板 - 主题色 + 颜色模式 + 粒子设置
-->
<script lang="ts">
	import { X, Palette, Check, Sun, Moon, Monitor, Zap, Sparkles } from '@lucide/svelte';
	import { currentTheme, themes, colorMode, colorModes, type ThemeId, type ColorMode, particlesEnabled } from '$lib/stores/theme';
	import { totalDownloadSpeed } from '$lib/stores/downloadSpeed';

	// 计算调试显示信息
	function getEmitRate(speedMbps: number): number {
		if (speedMbps <= 0) return 0;
		return Math.min(6 + speedMbps * 0.5, 56);
	}

	function getEstimatedParticles(speedMbps: number): number {
		return Math.round(getEmitRate(speedMbps) * 11); // 平均生命周期约11秒
	}

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
	$effect(() => {
		if (open) {
			document.body.classList.add('no-scroll');
			document.documentElement.classList.add('no-scroll');
		} else {
			document.body.classList.remove('no-scroll');
			document.documentElement.classList.remove('no-scroll');
		}
		return () => {
			document.body.classList.remove('no-scroll');
			document.documentElement.classList.remove('no-scroll');
		};
	});
</script>

{#if open}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="panel-overlay" onclick={onClose} onkeydown={handleKeydown}>
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div class="panel" onclick={(e) => e.stopPropagation()}>
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
		background: rgba(0, 0, 0, 0.5);
		backdrop-filter: blur(4px);
		display: flex;
		justify-content: flex-end;
		z-index: 2000;
	}

	.panel {
		width: 320px;
		height: 100%;
		background: var(--bg-sidebar);
		backdrop-filter: blur(24px) saturate(180%);
		-webkit-backdrop-filter: blur(24px) saturate(180%);
		border-left: 1px solid var(--border-color);
		box-shadow: 
			-4px 0 24px rgba(0, 0, 0, 0.15),
			0 1px 2px rgba(255, 255, 255, 0.1) inset;
		display: flex;
		flex-direction: column;
		animation: slide-in 0.25s cubic-bezier(0.25, 0.46, 0.45, 0.94);
	}

	@keyframes slide-in {
		from {
			transform: translateX(100%);
		}
		to {
			transform: translateX(0);
		}
	}

	.panel-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 20px 24px;
		border-bottom: 1px solid var(--border-color);
	}

	.panel-header h2 {
		font-size: 18px;
		font-weight: 600;
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
		background: var(--border-light);
		color: var(--text-primary);
	}

	.panel-body {
		flex: 1;
		padding: 24px;
		overflow-y: auto;
	}

	.settings-section {
		margin-bottom: 32px;
	}

	.section-header {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 13px;
		font-weight: 500;
		color: var(--text-secondary);
		margin-bottom: 16px;
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.theme-grid {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 12px;
	}

	.theme-card {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		padding: 12px 8px;
		background: var(--border-light);
		border: 1px solid var(--border-color);
		border-radius: 12px;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.theme-card:hover {
		background: var(--border-color);
	}

	.theme-card.active {
		border-color: var(--accent-primary);
		background: var(--accent-active-bg);
	}

	.theme-preview {
		width: 48px;
		height: 48px;
		border-radius: 50%;
		display: flex;
		align-items: center;
		justify-content: center;
		color: white;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
	}

	.theme-name {
		font-size: 12px;
		color: var(--text-secondary);
	}

	/* 颜色模式选择器 */
	.mode-grid {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 10px;
	}

	.mode-card {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		padding: 14px 10px;
		background: var(--border-light);
		border: 1px solid var(--border-color);
		border-radius: 12px;
		color: var(--text-muted);
		font-size: 12px;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.mode-card:hover {
		background: var(--border-color);
		color: var(--text-secondary);
	}

	.mode-card.active {
		border-color: var(--accent-primary);
		background: var(--accent-active-bg);
		color: var(--accent-text);
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
		background: var(--border-light);
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
		background: white;
		border-radius: 50%;
		box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
		transition: transform 0.2s ease;
	}

	.toggle-switch.active .toggle-knob {
		transform: translateX(20px);
	}
</style>
