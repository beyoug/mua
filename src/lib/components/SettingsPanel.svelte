<!--
  SettingsPanel.svelte
  浮动设置面板 - 主题色 + 颜色模式 + 粒子设置
-->
<script lang="ts">
	import { X, Palette, Check, Sun, Moon, Monitor, FileCode, FileUp } from '@lucide/svelte';
    import { fade, scale } from 'svelte/transition';
	import { currentTheme, themes, colorMode, colorModes, type ThemeId, type ColorMode } from '$lib/stores/theme';
    import { aria2Config, configPath, isImporting, loadAria2Config, importAria2Config } from '$lib/stores/aria2Config';
    import { appSettings, loadAppSettings, saveAppSettings } from '$lib/stores/settings';
	import { createScrollLockEffect } from '$lib';

	interface Props {
		open: boolean;
		onClose: () => void;
	}

	let { open, onClose }: Props = $props();
    let activeTab: 'basic' | 'advanced' = $state('basic');

	function selectTheme(themeId: ThemeId) {
		currentTheme.set(themeId);
	}

	function selectColorMode(mode: ColorMode) {
		colorMode.set(mode);
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			onClose();
		}
	}

	const themeList = Object.values(themes);

	const modeIcons = {
		'dark': Moon,
		'light': Sun,
		'auto': Monitor
	};

	// Load config when panel opens
	$effect(() => {
		if (open) {
			loadAria2Config();
            loadAppSettings();
		}
	});

    let isDirty = $state(false);
    
    function onPortChange() {
        isDirty = true;
    }

    async function saveSettings() {
        try {
            await saveAppSettings($appSettings);
            isDirty = false;
            // Optional: Toast success
        } catch (e) {
            alert('保存失败');
        }
    }

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
				<div class="tabs">
					<button 
						class="tab-btn" 
						class:active={activeTab === 'basic'} 
						onclick={() => activeTab = 'basic'}
					>
						基本设置
					</button>
					<button 
						class="tab-btn" 
						class:active={activeTab === 'advanced'} 
						onclick={() => activeTab = 'advanced'}
					>
						高级配置
					</button>
				</div>

				{#if activeTab === 'basic'}
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

                    <!-- 系统设置 -->
					<section class="settings-section">
						<div class="section-header">
							<Monitor size={16} />
							<span>系统</span>
						</div>
                        <div class="settings-group">
                            <div class="input-helper">
                                <label for="close-to-tray">关闭主面板时</label>
                                <span class="helper-text">
                                    {$appSettings.closeToTray ? "最小化到托盘，保持后台运行" : "退出应用"}
                                </span>
                            </div>
                            
                            <div class="switch-row">
                                <label class="switch">
                                    <input 
                                        type="checkbox" 
                                        id="close-to-tray"
                                        bind:checked={$appSettings.closeToTray}
                                        onchange={saveSettings} 
                                    />
                                    <span class="slider round"></span>
                                </label>
                            </div>

                            <div class="divider-small"></div>

                            <div class="input-helper">
                                <label for="auto-resume">自动恢复下载</label>
                                <span class="helper-text">
                                    {$appSettings.autoResume ? "启动时自动恢复未完成的任务" : "启动时暂停所有未完成的任务"}
                                </span>
                            </div>
                            
                            <div class="switch-row">
                                <label class="switch">
                                    <input 
                                        type="checkbox" 
                                        id="auto-resume"
                                        bind:checked={$appSettings.autoResume}
                                        onchange={saveSettings} 
                                    />
                                    <span class="slider round"></span>
                                </label>
                            </div>
                        </div>
                    </section>
				{/if}

				{#if activeTab === 'advanced'}
					<section class="settings-section">
						<div class="settings-group">
                            <!-- Helper Text -->
                             <div class="input-helper">
                                <label for="rpc-port">RPC 监听端口</label>
                                <span class="helper-text">修改后将在下次启动时生效</span>
                             </div>

                             <div class="input-row">
                                 <input 
                                    id="rpc-port"
                                    type="number" 
                                    min="1024" 
                                    max="65535"
                                    class="port-input"
                                    bind:value={$appSettings.rpcPort}
                                    onchange={onPortChange}
                                 />
                                 <button class="save-btn" onclick={saveSettings} disabled={!isDirty}>
                                     <Check size={14} />
                                     保存
                                 </button>
                             </div>
                        </div>
                        


                        <div class="divider"></div>

                        <div class="section-header">
							<FileCode size={16} />
							<span>Aria2 配置文件</span>
						</div>
						
						<div class="import-panel">
							<div class="config-status">
								{#if $aria2Config}
									<div class="status-indicator active">
										<Check size={14} />
										<span>已加载自定义配置</span>
									</div>
									<div class="config-preview">
										{$aria2Config}
									</div>
								{:else}
									<div class="status-indicator">
										<span>未检测到自定义配置文件</span>
									</div>
								{/if}
							</div>

							<div class="action-row">
								<span class="config-path" title={$configPath}>
									{$configPath ? $configPath : '初始化路径中...'}
								</span>
								<button class="import-btn" onclick={importAria2Config} disabled={$isImporting}>
									<FileUp size={14} />
									{$isImporting ? '导入中...' : '导入配置文件'}
								</button>
							</div>
						</div>
						<p class="section-hint">
							选择本地的 <code>aria2.conf</code> 文件导入。
							<br/>
							注意：导入将覆盖现有配置，且需要重启应用生效。
						</p>
					</section>
				{/if}
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

    .tabs {
        display: flex;
        gap: 8px;
        margin-bottom: 24px;
        background: var(--input-bg);
        padding: 4px;
        border-radius: 10px;
        border: 1px solid var(--border-normal);
    }

    .tab-btn {
        flex: 1;
        padding: 6px;
        font-size: 13px;
        color: var(--text-secondary);
        background: transparent;
        border: none;
        border-radius: 8px;
        cursor: pointer;
        transition: all 0.2s ease;
        font-weight: 500;
    }

    .tab-btn:hover {
        color: var(--text-primary);
    }

    .tab-btn.active {
        background: var(--dialog-bg);
        color: var(--accent-primary);
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
        font-weight: 600;
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

	.mode-card.active {
		border-color: var(--accent-primary);
		background: var(--accent-subtle);
		color: var(--accent-primary);
	}

    /* Import Panel */
    .import-panel {
        display: flex;
        flex-direction: column;
        gap: 12px;
        padding: 12px;
        background: var(--input-bg);
        border: 1px solid var(--border-color);
        border-radius: 12px;
    }

    .config-status {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .status-indicator {
        display: flex;
        align-items: center;
        gap: 6px;
        font-size: 13px;
        color: var(--text-secondary);
    }

    .status-indicator.active {
        color: var(--success, #10b981);
    }

    .config-preview {
        max-height: 80px;
        overflow-y: auto;
        padding: 8px;
        background: var(--bg-hover);
        border-radius: 6px;
        font-family: 'JetBrains Mono', monospace;
        font-size: 11px;
        color: var(--text-muted);
        white-space: pre-wrap;
        border: 1px solid var(--border-subtle);
    }

    .action-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 12px;
        padding-top: 8px;
        border-top: 1px solid var(--border-subtle);
    }

    .config-path {
        font-size: 11px;
        color: var(--text-muted);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        flex: 1;
        min-width: 0;
    }

    .import-btn {
        display: flex;
        align-items: center;
        gap: 6px;
        padding: 6px 12px;
        font-size: 12px;
        color: white;
        background: var(--accent-primary);
        border: none;
        border-radius: 6px;
        cursor: pointer;
        transition: opacity 0.2s;
        white-space: nowrap;
    }

    .import-btn:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }

    .section-hint {
        margin-top: 8px;
        font-size: 12px;
        color: var(--text-muted);
        line-height: 1.4;
    }
    
    .section-hint code {
        background: var(--border-light);
        padding: 2px 4px;
        border-radius: 4px;
        font-family: inherit;
        font-size: 11px;
    }

    .settings-group {
        display: flex;
        flex-direction: column;
        gap: 10px;
        background: var(--input-bg);
        border: 1px solid var(--border-normal);
        padding: 12px;
        border-radius: 12px;
        margin-bottom: 20px;
    }

    .input-helper {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .input-helper label {
        font-size: 13px;
        color: var(--text-primary);
        font-weight: 500;
    }
    
    .helper-text {
        font-size: 11px;
        color: var(--text-muted);
    }

    .input-row {
        display: flex;
        gap: 8px;
    }

    .port-input {
        flex: 1;
        background: var(--bg-hover);
        border: 1px solid var(--border-subtle);
        border-radius: 8px;
        padding: 8px 12px;
        color: var(--text-primary);
        font-size: 13px;
    }

    .port-input:focus {
        outline: none;
        border-color: var(--accent-primary);
        box-shadow: 0 0 0 2px var(--accent-subtle);
    }

    .save-btn {
        display: flex;
        align-items: center;
        gap: 6px;
        padding: 0 16px;
        background: var(--accent-primary);
        color: white;
        border: none;
        border-radius: 8px;
        font-size: 13px;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.2s;
    }

    .save-btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
        background: var(--border-strong);
    }
    
    .divider {
        height: 1px;
        background: var(--border-subtle);
        margin: 20px 0;
    }

    .divider-small {
        height: 1px;
        background: var(--border-subtle);
        margin: 12px 0;
        opacity: 0.5;
    }

    .switch-row {
        margin-top: 8px;
        display: flex;
        justify-content: flex-end;
    }

    /* Toggle Switch Styles */
    .switch {
        position: relative;
        display: inline-block;
        width: 36px;
        height: 20px;
    }

    .switch input {
        opacity: 0;
        width: 0;
        height: 0;
    }

    .slider {
        position: absolute;
        cursor: pointer;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background-color: var(--bg-hover);
        transition: .3s;
        border: 1px solid var(--border-subtle);
    }

    .slider:before {
        position: absolute;
        content: "";
        height: 14px;
        width: 14px;
        left: 2px;
        bottom: 2px;
        background-color: var(--text-muted);
        transition: .3s;
        border-radius: 50%;
    }

    input:checked + .slider {
        background-color: var(--accent-subtle);
        border-color: var(--accent-primary);
    }

    input:checked + .slider:before {
        transform: translateX(16px);
        background-color: var(--accent-primary);
    }

    .slider.round {
        border-radius: 20px;
    }

</style>
