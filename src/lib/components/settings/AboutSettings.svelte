<script lang="ts">
  import { Github, Globe, Heart } from '@lucide/svelte';
  import { open } from '@tauri-apps/plugin-shell';
  import { getVersion } from '@tauri-apps/api/app';
  import { onMount } from 'svelte';
  import { createLogger } from '$lib/utils/logger';

  const logger = createLogger('AboutSettings');

  let version = $state("0.0.0");
 
  onMount(async () => {
    try {
      version = await getVersion();
    } catch (e) {
      logger.error('Failed to get version', { error: e });
    }
  });

  async function openUrl(url: string) {
    try {
      await open(url);
    } catch (e) {
      logger.error('Failed to open URL', { url, error: e });
    }
  }
</script>

<div class="settings-container">
  <div class="about-card">
    <div class="about-header">
      <div class="app-logo">
        <img src="/logo.png" alt="Mua" class="app-logo-img" />
      </div>

      <div class="brand-meta">
        <h2 class="app-name">Mua</h2>
        <p class="app-tagline">简洁、现代的轻量级下载工具</p>
        <div class="app-version">Version {version}</div>
      </div>
    </div>
    
    <div class="action-grid">
      <button class="action-tile" onclick={() => openUrl('https://github.com/beyoug/mua')}>
        <Github size={20} />
        <span>GitHub</span>
      </button>
      <button class="action-tile" onclick={() => openUrl('https://mua.local')}>
        <Globe size={20} />
        <span>官网</span>
      </button>
    </div>

  </div>

  <section class="settings-section">
    <h4 class="section-title">开源致谢</h4>
    <div class="credits-list">
      <div class="credit-item">
        <span class="lib-name">Tauri</span>
        <span class="lib-desc">高性能桌面框架</span>
      </div>
      <div class="credit-item">
        <span class="lib-name">Aria2</span>
        <span class="lib-desc">全能下载引擎</span>
      </div>
      <div class="credit-item">
        <span class="lib-name">Svelte</span>
        <span class="lib-desc">现代极简 UI 框架</span>
      </div>
    </div>
  </section>

  <footer class="about-footer">
    <div class="heart-icon"><Heart size={14} fill="currentColor" /></div>
    <span>Made with love for excellence</span>
  </footer>
</div>

<style>
  .settings-container {
    display: flex;
    flex-direction: column;
    gap: 16px;
    align-items: stretch;
    padding-top: 4px;
  }

  .about-card {
    width: 100%;
    background: color-mix(in srgb, var(--input-bg) 96%, transparent);
    border: none;
    border-radius: 16px;
    padding: 24px;
    display: flex;
    flex-direction: column;
    gap: 18px;
    box-shadow: inset 0 1px 0 color-mix(in srgb, #ffffff 10%, transparent);
  }

  .about-header {
    display: flex;
    align-items: center;
    gap: 14px;
  }

  .app-logo {
    flex-shrink: 0;
  }

  .app-logo-img {
    width: 72px;
    height: 72px;
    filter: drop-shadow(0 6px 14px color-mix(in srgb, var(--accent-glow) 28%, transparent));
    object-fit: contain;
  }

  .brand-meta {
    display: flex;
    flex-direction: column;
    gap: 5px;
    min-width: 0;
  }

  .app-name {
    font-size: 22px;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0;
    letter-spacing: -0.02em;
  }

  .app-tagline {
    font-size: 13px;
    color: var(--text-secondary);
    margin: 0;
    line-height: 1.35;
  }

  .app-version {
    font-size: 11px;
    color: var(--text-tertiary);
    font-family: var(--font-mono);
    margin-top: 2px;
    background: var(--surface-hover);
    padding: 2px 9px;
    border-radius: 10px;
    width: fit-content;
  }

  .action-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 10px;
    width: 100%;
  }

  .action-tile {
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: center;
    gap: 10px;
    min-height: 42px;
    padding: 10px 12px;
    background: var(--surface-hover);
    border: none;
    border-radius: 12px;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--border-subtle) 44%, transparent);
  }

  .action-tile span {
    font-size: 12px;
    font-weight: 500;
  }

  .action-tile:hover {
    background: color-mix(in srgb, var(--accent-primary) 10%, transparent);
    color: var(--text-primary);
    transform: translateY(-1px);
    box-shadow:
      inset 0 0 0 1px color-mix(in srgb, var(--accent-primary) 24%, transparent),
      0 8px 16px -14px color-mix(in srgb, var(--accent-glow) 24%, transparent);
  }

  .action-tile:focus-visible {
    outline: none;
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent-primary) 20%, transparent);
  }

  .settings-section {
    width: 100%;
  }

  .credits-list {
    background: color-mix(in srgb, var(--input-bg) 94%, transparent);
    border: none;
    border-radius: 12px;
    width: 100%;
    overflow: hidden;
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--border-subtle) 44%, transparent);
  }

  .credit-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    border-bottom: none;
    transition: background 0.2s ease;
  }

  .credit-item:hover {
    background: var(--surface-hover);
  }

  .credit-item:last-child {
    border-bottom: none;
  }

  .lib-name {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .lib-desc {
    font-size: 11px;
    color: var(--text-muted);
  }

  .about-footer {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    color: var(--text-muted);
    margin-top: 2px;
    opacity: 0.74;
    padding-left: 2px;
  }

  .heart-icon {
    color: var(--semantic-danger);
  }

  @media (max-width: 620px) {
    .about-header {
      align-items: flex-start;
    }

    .action-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
