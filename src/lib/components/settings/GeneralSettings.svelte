<script lang="ts">
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
