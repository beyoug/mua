<script lang="ts">
  import { Check, Sun, Moon, Monitor } from '@lucide/svelte';
  import { scale } from 'svelte/transition';
  import { currentTheme, themes, colorMode, colorModes, particlesEnabled, type ThemeId, type ColorMode } from '$lib/stores/theme';

  function selectTheme(themeId: ThemeId) {
    currentTheme.set(themeId);
  }

  function selectColorMode(mode: ColorMode) {
    colorMode.set(mode);
  }

  function toggleParticles() {
    particlesEnabled.set(!$particlesEnabled);
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
            checked={$particlesEnabled}
            onchange={toggleParticles}
          />
          <span class="slider"></span>
        </label>
      </div>
    </div>
  </section>
</div>

<style>
  /* 组件特有样式 — 主题网格 */
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

  /* 组件特有样式 — 模式网格 */
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
</style>
