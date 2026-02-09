<script lang="ts">
  import { Palette, Check, Sun, Moon, Monitor } from '@lucide/svelte';
  import { scale } from 'svelte/transition';
  import { currentTheme, themes, colorMode, colorModes, particlesEnabled, type ThemeId, type ColorMode } from '$lib/stores/theme';

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
</script>

<div class="settings-container">
  <section class="settings-section">
    <h4 class="section-title">主题颜色</h4>
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
          <span class="theme-label">{theme.name}</span>
        </button>
      {/each}
    </div>
  </section>

  <section class="settings-section">
    <h4 class="section-title">外观模式</h4>
    <div class="mode-grid">
      {#each colorModes as mode}
        {@const Icon = modeIcons[mode.id]}
        <button
          class="mode-card"
          class:active={$colorMode === mode.id}
          onclick={() => selectColorMode(mode.id)}
        >
          <div class="mode-icon-wrapper">
            <Icon size={18} />
          </div>
          <span class="mode-name">{mode.name}</span>
          {#if $colorMode === mode.id}
            <div class="active-dot" in:scale></div>
          {/if}
        </button>
      {/each}
    </div>
  </section>
  
  <section class="settings-section">
    <h4 class="section-title">特效</h4>
    <div class="setting-list">
      <div class="setting-item">
        <div class="setting-info">
          <div class="setting-name">背景气泡动画</div>
          <div class="setting-desc">
            {$particlesEnabled ? "开启动态气泡背景特效" : "已停用，降低资源占用"}
          </div>
        </div>
        <label class="switch">
          <input 
            type="checkbox" 
            bind:checked={$particlesEnabled}
          />
          <span class="slider"></span>
        </label>
      </div>
    </div>
  </section>
</div>

<style>
  .settings-container {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  /* ... (Existing grid styles) ... */
  .theme-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(60px, 1fr));
    gap: 12px;
    background: var(--input-bg);
    padding: 16px;
    border-radius: 12px;
    border: 1px solid var(--border-normal);
  }

  .theme-card {
    all: unset;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    padding: 8px;
    border-radius: 12px;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    background: transparent;
  }

  .theme-card:hover {
    background: var(--surface-hover);
  }

  .theme-preview {
    width: 38px;
    height: 38px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    border: none;
    overflow: hidden;
    position: relative;
    background-clip: padding-box;
    box-sizing: border-box;
    box-shadow: none;
  }

  .theme-card.active .theme-preview {
    border: 2px solid var(--text-primary);
  }

  .theme-label {
    font-size: 11px;
    color: var(--text-secondary);
    font-weight: 500;
  }

  .mode-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 10px;
  }

  .mode-card {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    padding: 16px;
    background: var(--input-bg);
    border: 1px solid var(--border-normal);
    border-radius: 12px;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .mode-card:hover {
    background: var(--surface-active);
    border-color: var(--border-strong);
    color: var(--text-primary);
    transform: translateY(-2px);
  }

  .mode-card.active {
    background: var(--accent-active-bg);
    border-color: var(--accent-primary);
    color: var(--accent-primary);
  }

  .mode-icon-wrapper {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 8px;
  }

  .mode-name {
    font-size: 12px;
    font-weight: 500;
  }

  .active-dot {
    position: absolute;
    top: 8px;
    right: 8px;
    width: 6px;
    height: 6px;
    background: var(--accent-primary);
    border-radius: 50%;
  }

  /* List & Switch Styles */
  .setting-list {
    background: var(--input-bg);
    border: 1px solid var(--border-normal);
    border-radius: 12px;
    overflow: hidden;
  }

  .setting-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    transition: background 0.2s;
  }

  .setting-item:hover {
    background: var(--surface-hover);
  }

  .setting-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .setting-name {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-primary);
  }

  .setting-desc {
    font-size: 11px;
    color: var(--text-muted);
  }

  /* Switch Styles (Sync with GeneralSettings) */
  .switch {
    position: relative;
    display: inline-block;
    width: 34px;
    height: 18px;
    flex-shrink: 0;
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
    background-color: var(--border-strong);
    transition: .3s cubic-bezier(0.4, 0, 0.2, 1);
    border-radius: 20px;
  }

  .slider:before {
    position: absolute;
    content: "";
    height: 14px;
    width: 14px;
    left: 2px;
    bottom: 2px;
    background-color: white;
    transition: .3s cubic-bezier(0.4, 0, 0.2, 1);
    border-radius: 50%;
    box-shadow: 0 1px 3px rgba(0,0,0,0.2);
  }

  input:checked + .slider {
    background-color: var(--accent-primary);
  }

  input:checked + .slider:before {
    transform: translateX(16px);
  }
</style>
