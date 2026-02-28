<script lang="ts">
  import { Check, Sun, Moon, Monitor } from '@lucide/svelte';
  import { scale } from 'svelte/transition';
  import { currentTheme, themes, colorMode, colorModes, particlesEnabled, type ThemeId, type ColorMode } from '$lib/services/theme';

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
  const singleTheme = themeList[0] ?? null;
  const isSingleTheme = themeList.length === 1;

  const modeIcons = {
    'dark': Moon,
    'light': Sun,
    'auto': Monitor
  };
</script>

<div class="settings-container">
  <section class="settings-section">
    <h4 class="section-title">主题颜色</h4>
    {#if isSingleTheme && singleTheme}
      <div class="fixed-theme-card" role="status" aria-label="当前固定主题">
        <div
          class="fixed-theme-swatch"
          style="background: linear-gradient(135deg, {singleTheme.primary}, {singleTheme.secondary})"
          aria-hidden="true"
        >
          <Check size={14} strokeWidth={3} />
        </div>
        <div class="fixed-theme-title">{singleTheme.name}</div>
      </div>
    {:else}
      <div class="theme-grid multi-theme-grid">
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
    {/if}
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
    grid-template-columns: repeat(auto-fill, minmax(90px, 1fr));
    gap: 8px;
    background: color-mix(in srgb, var(--input-bg) 96%, transparent);
    padding: 10px;
    border-radius: 12px;
    border: none;
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--border-subtle) 42%, transparent);
  }

  .fixed-theme-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 6px;
    min-height: 66px;
    width: 112px;
    padding: 6px 8px;
    border-radius: 12px;
    background: color-mix(in srgb, var(--input-bg) 96%, transparent);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--border-subtle) 42%, transparent);
  }

  .fixed-theme-swatch {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
    flex-shrink: 0;
    box-shadow: inset 0 0 0 1px color-mix(in srgb, #ffffff 26%, transparent);
  }

  .fixed-theme-title {
    font-size: 11px;
    color: var(--text-primary);
    font-weight: 600;
    line-height: 1.2;
    text-align: center;
  }

  .multi-theme-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: stretch;
  }

  .theme-card {
    all: unset;
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: center;
    gap: 8px;
    cursor: pointer;
    min-height: 42px;
    min-width: 124px;
    padding: 8px 10px;
    border-radius: 12px;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    background: transparent;
  }

  .theme-card:hover {
    background: var(--surface-hover);
  }

  .theme-card:focus-visible,
  .mode-card:focus-visible {
    outline: none;
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent-primary) 20%, transparent);
  }

  .theme-preview {
    width: 34px;
    height: 34px;
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
    box-shadow: inset 0 0 0 1px color-mix(in srgb, #ffffff 24%, transparent);
  }

  .theme-card.active .theme-preview {
    box-shadow:
      0 0 0 2px color-mix(in srgb, var(--accent-primary) 30%, transparent),
      inset 0 0 0 1px color-mix(in srgb, #ffffff 32%, transparent);
  }

  .theme-label {
    font-size: 11px;
    color: var(--text-tertiary);
    font-weight: 500;
    text-align: left;
  }

  .theme-card.active .theme-label {
    color: var(--text-primary);
  }

  /* 组件特有样式 — 模式网格 */
  .mode-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 8px;
    border-radius: 12px;
    padding: 2px;
  }

  .mode-card {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    min-height: 74px;
    padding: 10px 8px;
    background: color-mix(in srgb, var(--input-bg) 96%, transparent);
    border: none;
    border-radius: 12px;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--border-subtle) 42%, transparent);
  }

  .mode-card:hover {
    background: var(--surface-active);
    color: var(--text-primary);
    transform: translateY(-1px);
  }

  .mode-card.active {
    background: var(--accent-active-bg);
    color: var(--accent-primary);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--accent-primary) 30%, transparent);
  }

  .mode-icon-wrapper {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 30px;
    height: 30px;
    background: var(--surface-hover);
    border-radius: 8px;
  }

  .mode-name {
    font-size: 12px;
    font-weight: 500;
    line-height: 1.2;
  }

  .active-dot {
    position: absolute;
    top: 8px;
    right: 8px;
    width: 5px;
    height: 5px;
    background: var(--accent-primary);
    border-radius: 50%;
  }

  @media (max-width: 640px) {
    .multi-theme-grid {
      display: grid;
      grid-template-columns: 1fr;
    }

    .mode-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
