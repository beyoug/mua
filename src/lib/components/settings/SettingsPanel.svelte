<!--
  SettingsPanel.svelte
  浮动设置面板 - 侧边栏导航布局
-->
<script lang="ts">
  import { X, Palette, Settings2, Download, Cpu, Info, Terminal, Network } from '@lucide/svelte';
	import { fade, scale } from 'svelte/transition';
	import { loadAria2Config } from '$lib/services/aria2Config';
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
    backdrop-filter: blur(10px) saturate(115%);
    z-index: 2000;
  }

  .panel {
    position: fixed;
    left: 224px;
    top: 12px;
    right: 12px;
    bottom: 12px;
    background: color-mix(in srgb, var(--dialog-bg) 97%, transparent);
    backdrop-filter: var(--glass-blur) var(--glass-saturate);
    -webkit-backdrop-filter: var(--glass-blur) var(--glass-saturate);
    border: 1px solid color-mix(in srgb, var(--panel-glass-border, var(--glass-border)) 68%, transparent);
    border-radius: 18px;
    box-shadow: var(--panel-glass-shadow, var(--glass-shadow));
    display: flex;
    overflow: hidden;
  }

  /* 侧边栏样式 */
  .panel-sidebar {
    width: 100px;
    background:
      linear-gradient(
        168deg,
        color-mix(in srgb, var(--glass-elevated-bg, var(--dialog-bg)) 84%, var(--accent-primary) 8%),
        color-mix(in srgb, var(--glass-elevated-bg, var(--dialog-bg)) 92%, transparent)
      );
    border-left: 1px solid color-mix(in srgb, var(--glass-border) 34%, transparent);
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
    position: relative;
  }

  .sidebar-nav::before {
    content: "";
    position: absolute;
    inset: 8px;
    border-radius: 10px;
    background: color-mix(in srgb, var(--surface-hover) 58%, transparent);
    border: 1px solid color-mix(in srgb, var(--glass-border) 32%, transparent);
    pointer-events: none;
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
    position: relative;
    z-index: 1;
    overflow: hidden;
  }

  .nav-item::before {
    content: "";
    position: absolute;
    left: -1px;
    top: 6px;
    bottom: 6px;
    width: 2px;
    border-radius: 2px;
    background: linear-gradient(
      180deg,
      color-mix(in srgb, var(--accent-secondary) 80%, white),
      color-mix(in srgb, var(--accent-primary) 80%, transparent)
    );
    opacity: 0;
    transform: scaleY(0.5);
    transition: opacity 0.16s ease, transform 0.16s ease;
  }

  .nav-item:hover {
    background: color-mix(in srgb, var(--accent-primary) 12%, transparent);
    color: var(--text-primary);
    box-shadow: var(--control-shadow-rest);
  }

  .nav-item:focus-visible {
    outline: none;
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent-primary) 20%, transparent);
  }

  .nav-item.active {
    background: color-mix(in srgb, var(--accent-primary) 18%, transparent);
    color: var(--accent-on-glass, var(--accent-text));
    box-shadow:
      inset 0 1px 0 color-mix(in srgb, var(--glass-highlight) 72%, transparent),
      0 0 0 1px color-mix(in srgb, var(--accent-primary) 14%, transparent),
      0 10px 18px -16px color-mix(in srgb, var(--accent-glow) 46%, transparent);
  }

  .nav-item.active::before {
    opacity: 1;
    transform: scaleY(1);
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
    border-bottom: 1px solid color-mix(in srgb, var(--glass-border) 32%, transparent);
    background: color-mix(in srgb, var(--glass-elevated-bg, var(--dialog-bg)) 70%, transparent);
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
    background: var(--control-bg);
    border: 1px solid color-mix(in srgb, var(--control-border) 60%, transparent);
    border-radius: 8px;
    color: var(--text-muted);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .close-btn:hover {
    background: var(--control-bg-hover);
    border-color: color-mix(in srgb, var(--control-border-hover) 66%, transparent);
    color: var(--text-primary);
  }

  .close-btn:focus-visible {
    outline: none;
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent-primary) 20%, transparent);
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
    background:
      linear-gradient(
        164deg,
        color-mix(in srgb, var(--glass-elevated-bg, var(--control-bg)) 72%, var(--accent-primary) 4%),
        color-mix(in srgb, var(--glass-elevated-bg, var(--control-bg)) 78%, transparent)
      );
    border-radius: 12px;
    padding: 12px;
    border: 1px solid color-mix(in srgb, var(--glass-border) 34%, transparent);
    box-shadow:
      var(--glass-inner-shadow),
      0 12px 20px -20px rgba(0, 0, 0, 0.78);
  }

  :global(.section-title) {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.055em;
    margin-bottom: 10px;
    padding-left: 1px;
    line-height: 1.2;
  }

  /* ── 共享设置布局 ── */
  :global(.settings-container) {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  :global(.setting-list) {
    background: color-mix(in srgb, var(--glass-elevated-bg, var(--control-bg)) 88%, transparent);
    border: none;
    border-radius: 12px;
    overflow: hidden;
    box-shadow:
      inset 0 0 0 1px color-mix(in srgb, var(--border-subtle) 30%, transparent),
      0 10px 18px -20px rgba(0, 0, 0, 0.72);
  }

  :global(.setting-item) {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 16px;
    border-bottom: none;
    transition: background 0.2s;
  }

	:global(.setting-item:hover:not(.disabled)) {
		background: color-mix(in srgb, var(--accent-primary) 10%, transparent);
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
    font-weight: 550;
    color: var(--text-primary);
  }

  :global(.setting-desc) {
    font-size: 11px;
    color: color-mix(in srgb, var(--text-muted) 92%, transparent);
    line-height: 1.35;
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
    background-color: color-mix(in srgb, var(--control-bg) 96%, rgba(120, 120, 128, 0.42));
    box-shadow:
      inset 0 0 0 1px color-mix(in srgb, var(--control-border) 46%, transparent),
      var(--control-shadow-rest);
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
    background: linear-gradient(
      135deg,
      color-mix(in srgb, var(--accent-primary) 92%, white),
      var(--accent-secondary)
    );
    box-shadow:
      inset 0 0 0 1px color-mix(in srgb, var(--accent-primary) 18%, transparent),
      0 8px 14px -10px color-mix(in srgb, var(--accent-glow) 62%, transparent);
  }

  :global(input:checked + .slider:before) {
    transform: translateX(16px);
  }

  :global(.switch input:focus-visible + .slider) {
    outline: none;
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent-primary) 20%, transparent);
  }

	:global(html.dark) .panel-overlay {
		background: color-mix(in srgb, var(--dialog-overlay-bg) 94%, rgba(1, 5, 12, 0.66));
		backdrop-filter: blur(14px) saturate(132%);
		-webkit-backdrop-filter: blur(14px) saturate(132%);
	}

	:global(html.dark) .panel {
		background:
			linear-gradient(
				162deg,
				color-mix(in srgb, var(--dialog-bg) 90%, var(--accent-primary) 10%),
				color-mix(in srgb, var(--dialog-bg) 97%, transparent)
			),
			color-mix(in srgb, var(--dialog-bg) 98%, transparent);
	}

	:global(html.dark) .close-btn {
		box-shadow: var(--control-shadow-rest);
	}

	:global(html.dark) .close-btn:hover {
		box-shadow: var(--control-shadow-elevated);
	}

	:global(html.light) .panel,
	:global(html.light) .panel-content,
	:global(html.light) :global(.settings-section),
	:global(html.light) :global(.setting-list),
	:global(html.light) :global(.setting-item),
	:global(html.light) :global(.setting-item:hover:not(.disabled)) {
		box-shadow: none;
	}

	:global(html.light) :global(.settings-section),
	:global(html.light) :global(.setting-list) {
		border: none;
	}

	:global(html.light) .content-header,
	:global(html.light) .sidebar-nav::before,
	:global(html.light) .panel-sidebar {
		border: none;
	}

	:global(html.light) .close-btn,
	:global(html.light) :global(.slider),
	:global(html.light) :global(input:checked + .slider) {
		box-shadow: none;
	}
</style>
