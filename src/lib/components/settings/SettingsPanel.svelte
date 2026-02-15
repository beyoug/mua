<!--
  SettingsPanel.svelte
  浮动设置面板 - 侧边栏导航布局
-->
<script lang="ts">
  import {
    X,
    Palette,
    Settings2,
    Download,
    Cpu,
    Info,
    Terminal,
    Network,
  } from "@lucide/svelte";
  import { fade, scale } from "svelte/transition";
  import { loadAria2Config } from "$lib/services/aria2Config";
  import { loadAppSettings } from "$lib/services/settings";
  import { createScrollLockEffect } from "$lib";

  import AppearanceSettings from "./AppearanceSettings.svelte";
  import GeneralSettings from "./GeneralSettings.svelte";
  import DownloadSettings from "./DownloadSettings.svelte";
  import CoreSettings from "./CoreSettings.svelte";
  import AboutSettings from "./AboutSettings.svelte";
  import LogSettings from "./LogSettings.svelte";
  import BTSettings from "./BTSettings.svelte";

  interface Props {
    open: boolean;
    onClose: () => void;
  }

  let { open, onClose }: Props = $props();

  type TabId =
    | "appearance"
    | "general"
    | "download"
    | "bt"
    | "core"
    | "about"
    | "logs";
  let activeTab: TabId = $state("appearance");

  const navItems = [
    { id: "appearance" as TabId, label: "外观", icon: Palette },
    { id: "general" as TabId, label: "通用", icon: Settings2 },
    { id: "download" as TabId, label: "下载", icon: Download },
    { id: "bt" as TabId, label: "BT", icon: Network },
    { id: "core" as TabId, label: "核心", icon: Cpu },
    { id: "logs" as TabId, label: "日志", icon: Terminal },
    { id: "about" as TabId, label: "关于", icon: Info },
  ];

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
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
  <div
    class="panel-overlay"
    in:fade={{ duration: 150 }}
    out:fade={{ duration: 100 }}
    onclick={onClose}
    onkeydown={handleKeydown}
  >
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
      class="panel"
      in:scale={{ duration: 150, start: 0.98, opacity: 0.5 }}
      out:fade={{ duration: 80 }}
      onclick={(e) => e.stopPropagation()}
    >
      <!-- 左侧内容区 (原右侧) -->
      <main class="panel-content">
        <header class="content-header">
          <h3>{navItems.find((i) => i.id === activeTab)?.label}</h3>
          <button class="close-btn" onclick={onClose}>
            <X size={18} />
          </button>
        </header>

        <div class="content-body">
          {#key activeTab}
            <div
              class="tab-pane"
              in:fade={{ duration: 120 }}
              out:fade={{ duration: 90 }}
            >
              <div in:scale={{ duration: 180, start: 0.985 }}>
                {#if activeTab === "appearance"}
                  <AppearanceSettings />
                {:else if activeTab === "general"}
                  <GeneralSettings />
                {:else if activeTab === "download"}
                  <DownloadSettings />
                {:else if activeTab === "bt"}
                  <BTSettings />
                {:else if activeTab === "core"}
                  <CoreSettings />
                {:else if activeTab === "logs"}
                  <div class="tab-pane-fill">
                    <LogSettings />
                  </div>
                {:else if activeTab === "about"}
                  <AboutSettings />
                {/if}
              </div>
            </div>
          {/key}
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
              onclick={() => (activeTab = item.id)}
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
    background: var(--dialog-overlay-bg);
    backdrop-filter: var(--overlay-backdrop-soft-blur);
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
    border: none;
    border-radius: 18px;
    box-shadow: var(--glass-shadow);
    display: flex;
    overflow: hidden;
  }

  /* 侧边栏样式 */
  .panel-sidebar {
    width: 100px;
    background: var(--bg-secondary);
    border-left: none;
    display: flex;
    flex-direction: column;
  }

  .sidebar-header {
    padding: 0 16px; /* Unified padding */
    height: 60px; /* Standardized height */
    display: flex;
    align-items: center;
    box-sizing: border-box;
  }

  .sidebar-header h2 {
    font-size: 20px; /* Up from 18px */
    font-weight: 700; /* Up from 600 */
    color: var(--text-primary);
    margin: 0;
    letter-spacing: -0.01em;
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
    padding: 7px 10px;
    border: none;
    background: transparent;
    border: 1px solid transparent; /* Keep transparent border to prevent layout shift on hover if hover adds border */
    color: var(--text-secondary);
    border-radius: 8px;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
    text-align: left;
    position: relative;
    overflow: hidden;
  }

  .nav-item::after {
    content: "";
    position: absolute;
    right: 8px;
    top: 50%;
    width: 5px;
    height: 5px;
    border-radius: 999px;
    background: var(--accent-primary);
    opacity: 0;
    transform: translateY(-50%) scale(0.7);
    transition:
      opacity 0.2s ease,
      transform 0.22s ease;
  }

  .nav-item:hover {
    background: var(--nav-hover-bg);
    border-color: var(--nav-hover-border);
    color: var(--text-primary);
    transform: translateX(1px);
  }

  .nav-item.active {
    background: var(--nav-active-bg);
    color: var(--nav-active-text);
    border-color: var(--nav-active-border);
    box-shadow: var(--shadow-sm);
    transform: translateX(3px); /* Increased slightly */
    font-weight: 600; /* Bolder text for active state */
  }

  .nav-item.active::after {
    opacity: 1;
    transform: translateY(-50%) scale(1);
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
    padding: 0 24px; /* Unified padding */
    height: 60px; /* Standardized height */
    box-sizing: border-box;
  }

  .content-header h3 {
    font-size: 15px; /* Slightly smaller for refinement */
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
    flex: 1;
    padding: 8px 24px 24px; /* Increased side padding to 24px */
    overflow-y: auto;
    scrollbar-gutter: stable;
    position: relative;
  }

  .tab-pane {
    animation: panel-enter-glow 220ms ease-out;
  }

  .tab-pane-fill {
    height: 100%;
  }

  @keyframes panel-enter-glow {
    0% {
      opacity: 0.88;
      filter: saturate(0.96);
    }
    100% {
      opacity: 1;
      filter: saturate(1);
    }
  }

  :global(.settings-section) {
    margin-bottom: 16px; /* Increased breathing room */
    background: var(--settings-section-bg);
    border: none;
    padding: 12px; /* Reduced from 16px */
    box-shadow: var(--settings-section-shadow);
    border-radius: 12px;
  }

  :global(.section-title) {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
    margin-bottom: 12px;
    padding-left: 2px;
    line-height: 1.2;
    letter-spacing: 0.04em;
    text-transform: uppercase; /* Re-added uppercase */
  }

  /* ── 共享设置布局 ── */
  :global(.settings-container) {
    display: flex;
    flex-direction: column;
    gap: 8px; /* Reduced from 12px */
  }

  :global(.setting-list) {
    background: var(--settings-list-bg);
    border: none;
    border-radius: 10px;
    overflow: hidden;
  }

  :global(.setting-item) {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 16px; /* Reduced to 10px */
    border-bottom: none;
    margin-bottom: 2px;
    border-radius: 8px; /* Added subtle radius */
    transition:
      background 0.2s ease,
      transform 0.2s ease;
  }

  :global(.setting-item:last-child) {
    border-bottom: none;
  }

  :global(.setting-item:hover:not(.disabled)) {
    background: var(--settings-list-row-bg-hover);
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
    font-size: 15px; /* Up from 14px */
    font-weight: 500;
    color: var(--settings-list-row-text);
    letter-spacing: 0.01em;
    margin-bottom: 3px; /* Increased spacing */
  }

  :global(.setting-desc) {
    font-size: 11px; /* Reduced to 11px */
    color: #94a3b8;
    line-height: 1.4;
    margin-top: 2px;
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
    background-color: var(
      --settings-switch-track-off,
      #cbd5e1
    ); /* Fallback to concrete gray */
    border: 1px solid rgba(0, 0, 0, 0.05); /* Slight border for definition */
    transition: 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    border-radius: 20px;
  }

  :global(.slider:before) {
    position: absolute;
    content: "";
    height: 14px;
    width: 14px;
    left: 2px;
    bottom: 2px;
    background-color: white; /* Force white thumb */
    transition: 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    border-radius: 50%;
    box-shadow: var(--settings-switch-thumb-shadow);
  }

  :global(input:checked + .slider) {
    background-color: #2563eb !important; /* Force Blue 600 */
    opacity: 1;
  }

  :global(input:checked + .slider:before) {
    transform: translateX(16px);
  }
</style>
