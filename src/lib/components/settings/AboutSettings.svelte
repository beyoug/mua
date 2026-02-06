<script lang="ts">
  import { Github, Globe, Heart } from '@lucide/svelte';

  const version = "0.0.1";
</script>

<div class="settings-container">
  <div class="about-card">
    <div class="app-logo">
      <div class="logo-circle">M</div>
    </div>
    <h2 class="app-name">Mua</h2>
    <p class="app-tagline">简洁、现代的轻量级下载工具</p>
    <div class="app-version">Version {version}</div>
    
    <div class="action-grid">
      <button class="action-tile" onclick={() => window.open('https://github.com', '_blank')}>
        <Github size={20} />
        <span>GitHub</span>
      </button>
      <button class="action-tile" onclick={() => window.open('https://mua.local', '_blank')}>
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
    gap: 20px;
    align-items: center;
    padding-top: 10px;
  }

  .about-card {
    width: 100%;
    background: var(--input-bg);
    border: 1px solid var(--border-normal);
    border-radius: 16px;
    padding: 32px 24px;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
  }

  .app-logo {
    margin-bottom: 16px;
  }

  .logo-circle {
    width: 64px;
    height: 64px;
    background: linear-gradient(135deg, var(--accent-primary), var(--accent-secondary));
    border-radius: 18px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 32px;
    font-weight: 800;
    color: white;
    box-shadow: 0 8px 24px var(--accent-glow);
  }

  .app-name {
    font-size: 24px;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0;
  }

  .app-tagline {
    font-size: 13px;
    color: var(--text-secondary);
    margin: 8px 0 0;
  }

  .app-version {
    font-size: 11px;
    color: var(--text-muted);
    font-family: 'JetBrains Mono', monospace;
    margin-top: 12px;
    background: var(--surface-hover);
    padding: 2px 8px;
    border-radius: 10px;
  }

  .action-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
    width: 100%;
    margin-top: 32px;
  }

  .action-tile {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 16px;
    background: var(--surface-hover);
    border: 1px solid var(--border-subtle);
    border-radius: 12px;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .action-tile:hover {
    background: var(--surface-active);
    color: var(--text-primary);
    border-color: var(--border-strong);
    transform: translateY(-2px);
  }

  .settings-section {
    width: 100%;
    align-self: flex-start;
  }

  .credits-list {
    background: var(--input-bg);
    border: 1px solid var(--border-normal);
    border-radius: 12px;
    width: 100%;
  }

  .credit-item {
    display: flex;
    justify-content: space-between;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border-subtle);
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
    margin-top: 10px;
    opacity: 0.6;
  }

  .heart-icon {
    color: #ef4444;
  }
</style>
