<script lang="ts">
  import { FolderOpen } from '@lucide/svelte';
  import { appSettings, updateAppSettings } from '$lib/services/settings';
  import { createLogger } from '$lib/utils/logger';
  import { pickSingleDirectory } from '$lib/utils/dialog';

  const logger = createLogger('DownloadSettings');

  async function selectFolder() {
    try {
      const path = await pickSingleDirectory('选择默认下载目录');
      if (path) {
        await updateAppSettings({ defaultSavePath: path });
      }
    } catch (e) {
      logger.error('Failed to select default download folder', { error: e });
    }
  }
</script>

<div class="settings-container">
  <section class="settings-section">
    <h4 class="section-title">下载偏好</h4>
    
    <div class="setting-list">
      <div class="setting-item">
        <div class="setting-info">
          <div class="setting-name">默认保存位置</div>
          <div class="setting-desc">新任务默认使用的路径</div>
        </div>
        <button class="path-btn" onclick={selectFolder}>
          <span class="path-val">{$appSettings.defaultSavePath || '~/Downloads'}</span>
          <FolderOpen size={14} />
        </button>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <div class="setting-name">同时下载任务数</div>
          <div class="setting-desc">建议设置在 1-5 之间</div>
        </div>
        <div class="inner-input">
          <input 
            type="number" 
            bind:value={$appSettings.maxConcurrentDownloads} 
            onchange={() => updateAppSettings({ maxConcurrentDownloads: $appSettings.maxConcurrentDownloads })}
            min="1" 
            max="16" 
          />
        </div>
      </div>
    </div>
  </section>

  <section class="settings-section">
    <h4 class="section-title">限速策略</h4>
    <div class="setting-list">
      <div class="setting-item">
        <div class="setting-info">
          <div class="setting-name">全局最大下载速度</div>
          <div class="setting-desc">0 表示不限制</div>
        </div>
        <div class="inner-input group">
          <input 
            type="text" 
            placeholder="0" 
            bind:value={$appSettings.globalMaxDownloadLimit}
            onchange={() => updateAppSettings({ globalMaxDownloadLimit: $appSettings.globalMaxDownloadLimit })}
          />
          <span class="unit">MB/s</span>
        </div>
      </div>
    </div>
  </section>
</div>

<style>
  /* 组件特有样式 */
  .path-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    background: var(--surface-hover);
    border: 1px solid var(--border-subtle);
    border-radius: 6px;
    color: var(--text-secondary);
    font-size: 12px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .path-btn:hover {
    background: var(--surface-active);
    color: var(--text-primary);
  }

  .path-btn:focus-visible {
    outline: none;
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent-primary) 20%, transparent);
  }

  .inner-input {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .inner-input input {
    width: 60px;
    background: var(--surface-hover);
    border: 1px solid var(--border-subtle);
    border-radius: 6px;
    padding: 4px 8px;
    color: var(--text-primary);
    font-size: 12px;
    text-align: center;
  }

  .inner-input input:focus-visible {
    outline: none;
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent-primary) 20%, transparent);
  }

  .inner-input.group input {
    width: 80px;
    border-top-right-radius: 0;
    border-bottom-right-radius: 0;
  }

  .unit {
    background: var(--surface-active);
    border: 1px solid var(--border-subtle);
    border-left: none;
    padding: 4px 8px;
    font-size: 11px;
    color: var(--text-muted);
    border-top-right-radius: 6px;
    border-bottom-right-radius: 6px;
  }
</style>
