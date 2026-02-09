<script lang="ts">
  import { Monitor, RotateCcw } from '@lucide/svelte';
  import { appSettings, saveAppSettings } from '$lib/stores/settings';
  import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart';
  import { onMount } from 'svelte';

  onMount(async () => {
    try {
      const enabled = await isEnabled();
      if (enabled !== $appSettings.autoStart) {
        $appSettings.autoStart = enabled;
        await saveAppSettings($appSettings);
      }
    } catch (e) {
      console.error('Failed to check autostart status', e);
    }
  });

  async function handleAutoStartChange() {
    try {
      if ($appSettings.autoStart) {
        await enable();
      } else {
        await disable();
      }
      await saveAppSettings($appSettings);
    } catch (e) {
      console.error('Failed to toggle autostart', e);
      // Rollback UI
      $appSettings.autoStart = !$appSettings.autoStart;
    }
  }

  async function toggleSetting(key: keyof typeof $appSettings) {
    // @ts-ignore
    $appSettings[key] = !$appSettings[key];
    try {
      await saveAppSettings($appSettings);
    } catch (e) {
      console.error('Failed to save settings', e);
    }
  }
</script>

<div class="settings-container">
  <section class="settings-section">
    <h4 class="section-title">系统行为</h4>
    
    <div class="setting-list">
      <div class="setting-item">
        <div class="setting-info">
          <div class="setting-name">关闭主面板时</div>
          <div class="setting-desc">
            {$appSettings.closeToTray ? "最小化到托盘，保持后台运行" : "直接退出应用程序"}
          </div>
        </div>
        <label class="switch">
          <input 
            type="checkbox" 
            bind:checked={$appSettings.closeToTray}
            onchange={() => saveAppSettings($appSettings)} 
          />
          <span class="slider"></span>
        </label>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <div class="setting-name">自动恢复下载</div>
          <div class="setting-desc">
            {$appSettings.autoResume ? "应用启动时自动恢复之前的任务" : "启动时保持任务原有状态"}
          </div>
        </div>
        <label class="switch">
          <input 
            type="checkbox" 
            bind:checked={$appSettings.autoResume}
            onchange={() => saveAppSettings($appSettings)} 
          />
          <span class="slider"></span>
        </label>
      </div>
    </div>
  </section>
  <section class="settings-section">
    <h4 class="section-title">启动项</h4>
    <div class="setting-list">
      <div class="setting-item">
        <div class="setting-info">
          <div class="setting-name">开机自启</div>
          <div class="setting-desc">随操作系统自动启动 Mua</div>
        </div>
        <label class="switch">
          <input 
            type="checkbox" 
            bind:checked={$appSettings.autoStart}
            onchange={handleAutoStartChange}
          />
          <span class="slider"></span>
        </label>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <div class="setting-name">启动时最小化</div>
          <div class="setting-desc">
            {$appSettings.startMinimized ? "静默启动到托盘" : "启动时正常显示窗口"}
          </div>
        </div>
        <label class="switch">
          <input 
            type="checkbox" 
            bind:checked={$appSettings.startMinimized}
            onchange={() => saveAppSettings($appSettings)}
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
    padding: 16px;
    border-bottom: 1px solid var(--border-subtle);
    transition: background 0.2s;
  }

  .setting-item:last-child {
    border-bottom: none;
  }

  .setting-item:hover:not(.disabled) {
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

  /* Switch Styles */
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
