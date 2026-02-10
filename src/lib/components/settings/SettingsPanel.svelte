<!--
  SettingsPanel.svelte
  浮动设置面板 - 侧边栏导航布局
-->
<script lang="ts">
  import { X, Palette, Settings2, Download, Cpu, Info, Terminal, Network } from '@lucide/svelte';
  import { fade, scale } from 'svelte/transition';
  import { loadAria2Config } from '$lib/stores/aria2Config';
  import { loadAppSettings } from '$lib/stores/settings';
  import { createScrollLockEffect } from '$lib';
  
  import AppearanceSettings from './AppearanceSettings.svelte';
  import GeneralSettings from './GeneralSettings.svelte';
  import DownloadSettings from './DownloadSettings.svelte';
  import CoreSettings from './CoreSettings.svelte';
  import AboutSettings from './AboutSettings.svelte';
  import LogSettings from './LogSettings.svelte';
  import BTSettings from './BTSettings.svelte';

  interface Props {
    open: boolean;
    onClose: () => void;
  }

  let { open, onClose }: Props = $props();
  
  type TabId = 'appearance' | 'general' | 'download' | 'bt' | 'core' | 'about' | 'logs';
  let activeTab: TabId = $state('appearance');

  const navItems = [
    { id: 'appearance' as TabId, label: '外观', icon: Palette },
    { id: 'general' as TabId, label: '通用', icon: Settings2 },
    { id: 'download' as TabId, label: '下载', icon: Download },
    { id: 'bt' as TabId, label: 'BT', icon: Network },
    { id: 'core' as TabId, label: '核心', icon: Cpu },
    { id: 'logs' as TabId, label: '日志', icon: Terminal },
    { id: 'about' as TabId, label: '关于', icon: Info },
  ];

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onClose();
    }
  }

  // Load config when panel opens
  $effect(() => {
    if (open) {
      loadAria2Config();
    }
  });

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
      
      <!-- 左侧内容区 (原右侧) -->
      <main class="panel-content">
        <header class="content-header">
          <h3>{navItems.find(i => i.id === activeTab)?.label}</h3>
          <button class="close-btn" onclick={onClose}>
            <X size={18} />
          </button>
        </header>

        <div class="content-body">
          {#if activeTab === 'appearance'}
            <div in:fade={{ duration: 150 }}>
              <AppearanceSettings />
            </div>
          {:else if activeTab === 'general'}
            <div in:fade={{ duration: 150 }}>
              <GeneralSettings />
            </div>
          {:else if activeTab === 'download'}
            <div in:fade={{ duration: 150 }}>
              <DownloadSettings />
            </div>
          {:else if activeTab === 'bt'}
            <div in:fade={{ duration: 150 }}>
              <BTSettings />
            </div>
          {:else if activeTab === 'core'}
            <div in:fade={{ duration: 150 }}>
              <CoreSettings />
            </div>
          {:else if activeTab === 'logs'}
            <div in:fade={{ duration: 150 }} style="height: 100%;">
              <LogSettings />
            </div>
          {:else if activeTab === 'about'}
            <div in:fade={{ duration: 150 }}>
              <AboutSettings />
            </div>
          {/if}
        </div>
      </main>

      <!-- 右侧边栏 (原左侧) -->
      <aside class="panel-sidebar">
        <header class="sidebar-header">
          <h2>设置</h2>
        </header>
        <nav class="sidebar-nav">
          {#each navItems as item}
            <button 
              class="nav-item" 
              class:active={activeTab === item.id}
              onclick={() => activeTab = item.id}
            >
              <item.icon size={16} />
              <span>{item.label}</span>
            </button>
          {/each}
        </nav>
      </aside>
    </div>
  </div>
{/if}

<style>
  .panel-overlay {
    position: fixed;
    inset: 0;
    background: var(--dialog-overlay-bg, rgba(0, 0, 0, 0.2));
    backdrop-filter: blur(4px);
    z-index: 2000;
  }

  .panel {
    position: fixed;
    left: 224px;
    top: 12px;
    right: 12px;
    bottom: 12px;
    background: var(--dialog-bg);
    backdrop-filter: var(--glass-blur) var(--glass-saturate);
    -webkit-backdrop-filter: var(--glass-blur) var(--glass-saturate);
    border: 1px solid var(--glass-border);
    border-radius: 18px;
    box-shadow: var(--glass-shadow);
    display: flex;
    overflow: hidden;
  }

  /* 侧边栏样式 */
  .panel-sidebar {
    width: 100px;
    background: rgba(255, 255, 255, 0.05);
    border-left: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
  }

  .sidebar-header {
    padding: 24px 20px 16px;
  }

  .sidebar-header h2 {
    font-size: 18px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
  }

  .sidebar-nav {
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 12px;
    border: none;
    background: transparent;
    color: var(--text-secondary);
    border-radius: 8px;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
    text-align: left;
  }

  .nav-item:hover {
    background: var(--surface-hover);
    color: var(--text-primary);
  }

  .nav-item.active {
    background: var(--accent-active-bg);
    color: var(--accent-primary);
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
  }

  /* 内容区样式 */
  .panel-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .content-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 24px;
    /* border-bottom: 1px solid var(--border-color); */
  }

  .content-header h3 {
    font-size: 16px;
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
    background: var(--input-bg);
    color: var(--text-primary);
  }

  .content-body {
    flex: 1;
    padding: 8px 24px 24px;
    overflow-y: auto;
    /* 优化滚动行为 */
    scrollbar-gutter: stable;
  }

  :global(.settings-section) {
    margin-bottom: 32px;
  }

  :global(.section-title) {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 12px;
    padding-left: 4px;
  }

  /* ── 共享设置布局 ── */
  :global(.settings-container) {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  :global(.setting-list) {
    background: var(--input-bg);
    border: 1px solid var(--border-normal);
    border-radius: 12px;
    overflow: hidden;
  }

  :global(.setting-item) {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px;
    border-bottom: 1px solid var(--border-subtle);
    transition: background 0.2s;
  }

  :global(.setting-item:last-child) {
    border-bottom: none;
  }

  :global(.setting-item:hover:not(.disabled)) {
    background: var(--surface-hover);
  }

  :global(.setting-item.vertical) {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

  :global(.setting-info) {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  :global(.setting-name) {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-primary);
  }

  :global(.setting-desc) {
    font-size: 11px;
    color: var(--text-muted);
  }

  /* ── 统一 Switch 开关 ── */
  :global(.switch) {
    position: relative;
    display: inline-block;
    width: 34px;
    height: 18px;
    flex-shrink: 0;
  }

  :global(.switch input) {
    opacity: 0;
    width: 0;
    height: 0;
  }

  :global(.slider) {
    position: absolute;
    cursor: pointer;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(120, 120, 128, 0.36);
    transition: .3s cubic-bezier(0.4, 0, 0.2, 1);
    border-radius: 20px;
  }

  :global(.slider:before) {
    position: absolute;
    content: "";
    height: 14px;
    width: 14px;
    left: 2px;
    bottom: 2px;
    background-color: var(--toggle-knob-c, white);
    transition: .3s cubic-bezier(0.4, 0, 0.2, 1);
    border-radius: 50%;
    box-shadow: 0 1px 3px rgba(0,0,0,0.2);
  }

  :global(input:checked + .slider) {
    background-color: var(--accent-primary);
  }

  :global(input:checked + .slider:before) {
    transform: translateX(16px);
  }
</style>
