<script lang="ts">
	import { Palette, Check, Sun, Moon, Monitor } from '@lucide/svelte';
	import { currentTheme, themes, colorMode, colorModes, type ThemeId, type ColorMode } from '$lib/stores/theme';
    import { appSettings, saveAppSettings } from '$lib/stores/settings';

	function selectTheme(themeId: ThemeId) {
		currentTheme.set(themeId);
	}

	function selectColorMode(mode: ColorMode) {
		colorMode.set(mode);
	}

	const themeList = Object.values(themes);

	const modeIcons = {
		'dark': Moon,
		'light': Sun,
		'auto': Monitor
	};

    async function saveSettings() {
        try {
            await saveAppSettings($appSettings);
            // Optional: Toast success
        } catch (e) {
            console.error('Failed to save settings', e);
        }
    }
</script>

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

<style>
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
		display: none;
	}

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
