<script lang="ts">
  import { Check, Sun, Moon, Monitor } from "@lucide/svelte";
  import { scale } from "svelte/transition";
  import {
    currentTheme,
    themes,
    colorMode,
    colorModes,
    particlesEnabled,
    type ThemeId,
    type ColorMode,
  } from "$lib/services/theme";

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
    dark: Moon,
    light: Sun,
    auto: Monitor,
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
            {$particlesEnabled
              ? "开启动态气泡背景特效"
              : "已停用，降低资源占用"}
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
    grid-template-columns: repeat(
      auto-fill,
      minmax(100px, 1fr)
    ); /* Larger cards */
    gap: 16px; /* Increased gap */
    background: transparent;
    padding: 4px; /* Space for focus rings */
    border: none;
    border-radius: 0;
  }

  .theme-card {
    all: unset;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    cursor: pointer;
    padding: 0;
    border-radius: 14px;
    transition: transform 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    background: transparent;
    border: none;
    box-shadow: none;
  }

  .theme-card:hover {
    background: transparent;
    transform: translateY(-4px);
    box-shadow: none;
    border-color: transparent;
  }

  .theme-card.active {
    border-color: transparent;
    background: transparent;
  }

  .theme-preview {
    width: 100%;
    aspect-ratio: 1.6; /* Mini-window shape */
    border-radius: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    border: 1px solid rgba(0, 0, 0, 0.05); /* Subtle border definition */
    overflow: hidden;
    position: relative;
    background-clip: padding-box;
    box-sizing: border-box;
    box-shadow: var(--shadow-sm);
  }

  .theme-card:hover .theme-preview {
    box-shadow: var(--shadow-md);
  }

  .theme-card.active .theme-preview {
    border: none;
    box-shadow:
      0 0 0 2px var(--bg-base),
      0 0 0 4px var(--accent-primary),
      var(--shadow-md); /* Ring effect + Shadow */
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
    gap: 12px; /* Increased gap */
  }

  .mode-card {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 18px; /* Reduced to compensate for border */
    background: var(--settings-control-bg);
    border: 2px solid transparent; /* Reserve space for active border */
    border-radius: 16px;
    color: var(--settings-list-row-text-muted);
    cursor: pointer;
    box-shadow: var(--shadow-sm);
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .mode-card:hover {
    background: var(--settings-control-bg-hover);
    color: var(--settings-list-row-text);
    transform: translateY(-2px);
    box-shadow: var(--shadow-md);
  }

  .mode-card.active {
    background: color-mix(
      in srgb,
      var(--accent-primary) 5%,
      transparent
    ); /* Subtle tint */
    border-color: var(--accent-primary); /* Accent border */
    color: var(--accent-primary);
    box-shadow: var(--shadow-md);
  }

  .mode-icon-wrapper {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 40px; /* Larger icon wrapper */
    height: 40px;
    background: rgba(0, 0, 0, 0.05); /* Subtle bg */
    border: none;
    border-radius: 12px; /* Softer shape */
    transition: all 0.2s;
  }

  .mode-card.active .mode-icon-wrapper {
    background: transparent; /* Remove bg to let icon pop */
    color: var(--accent-primary);
  }

  .mode-name {
    font-size: 13px; /* Slightly larger */
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
