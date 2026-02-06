<script lang="ts">
    import { Check, FileCode, FileUp, Key, RefreshCw, Copy, Eye, EyeOff, AlertCircle } from '@lucide/svelte';
    import { aria2Config, configPath, isImporting, loadAria2Config, importAria2Config } from '$lib/stores/aria2Config';
    import { appSettings, saveAppSettings } from '$lib/stores/settings';

    let isDirty = $state(false);
    let showSecret = $state(false);

    function onPortChange() {
        isDirty = true;
    }

    async function saveSettings() {
        try {
            await saveAppSettings($appSettings);
            isDirty = false;
        } catch (e) {
            console.error(e);
        }
    }

    function generateSecret() {
        if (confirm('重新生成密钥将导致当前连接断开，且需要重启应用生效。确定要继续吗？')) {
             $appSettings.rpcSecret = crypto.randomUUID();
             saveSettings();
             isDirty = false;
        }
    }

    async function copySecret() {
        if ($appSettings.rpcSecret) {
            try {
                await navigator.clipboard.writeText($appSettings.rpcSecret);
            } catch (e) {
                console.error("Copy failed", e);
            }
        }
    }
</script>

<div class="settings-container">
  <section class="settings-section">
    <h4 class="section-title">RPC 服务</h4>
    
    <div class="setting-list">
      <div class="setting-item">
        <div class="setting-info">
          <div class="setting-name">监听端口</div>
          <div class="setting-desc">Aria2 RPC 服务的端口</div>
        </div>
        <div class="input-actions">
          <input 
            type="number" 
            bind:value={$appSettings.rpcPort}
            oninput={onPortChange}
            class="inner-input-field" 
          />
          {#if isDirty}
            <button class="small-save-btn" onclick={saveSettings}>保存</button>
          {/if}
        </div>
      </div>

      <div class="setting-item vertical">
        <div class="setting-info">
          <div class="setting-name">连接密钥 (Secret)</div>
          <div class="setting-desc">用于 RPC 通信的身份验证</div>
        </div>
        
        <div class="secret-field">
          <div class="secret-box">
            <input 
              type={showSecret ? "text" : "password"} 
              bind:value={$appSettings.rpcSecret}
              readonly
            />
            <button class="icon-toggle" onclick={() => showSecret = !showSecret}>
              {#if showSecret}<EyeOff size={14} />{:else}<Eye size={14} />{/if}
            </button>
          </div>
          <div class="side-actions">
            <button class="mini-btn" onclick={copySecret} title="复制"><Copy size={14} /></button>
            <button class="mini-btn" onclick={generateSecret} title="重新生成"><RefreshCw size={14} /></button>
          </div>
        </div>
      </div>
    </div>
  </section>

  <section class="settings-section">
    <h4 class="section-title">高级配置</h4>
    <div class="import-card">
      <div class="import-header">
        <FileCode size={20} class="core-icon" />
        <div class="import-info">
          <div class="comp-name">自定义 aria2.conf</div>
          <div class="comp-path">{$configPath || '加载中...'}</div>
        </div>
      </div>
      
      <div class="config-actions">
        {#if $aria2Config}
          <div class="badge success">已加载自定义配置</div>
        {:else}
          <div class="badge gray">使用内置默认配置</div>
        {/if}
        <button class="secondary-btn" onclick={importAria2Config} disabled={$isImporting}>
          <FileUp size={14} />
          <span>导入配置</span>
        </button>
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
  }

  .setting-item.vertical {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

  .setting-item:last-child {
    border-bottom: none;
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

  .inner-input-field {
    width: 70px;
    background: var(--surface-hover);
    border: 1px solid var(--border-subtle);
    border-radius: 6px;
    padding: 6px 8px;
    color: var(--text-primary);
    font-size: 12px;
    text-align: center;
  }

  .input-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .small-save-btn {
    padding: 6px 12px;
    background: var(--accent-primary);
    color: white;
    border: none;
    border-radius: 6px;
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
  }

  /* Secret Row */
  .secret-field {
    width: 100%;
    display: flex;
    gap: 8px;
  }

  .secret-box {
    flex: 1;
    position: relative;
    display: flex;
    align-items: center;
  }

  .secret-box input {
    width: 100%;
    background: var(--surface-hover);
    border: 1px solid var(--border-subtle);
    border-radius: 8px;
    padding: 8px 36px 8px 12px;
    color: var(--text-primary);
    font-size: 12px;
    font-family: 'JetBrains Mono', monospace;
  }

  .icon-toggle {
    position: absolute;
    right: 8px;
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 4px;
    display: flex;
  }

  .side-actions {
    display: flex;
    gap: 4px;
  }

  .mini-btn {
    width: 34px;
    height: 34px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--surface-hover);
    border: 1px solid var(--border-subtle);
    border-radius: 8px;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.2s;
  }

  .mini-btn:hover {
    color: var(--text-primary);
    background: var(--surface-active);
  }

  /* Import Card */
  .import-card {
    background: var(--input-bg);
    border: 1px solid var(--border-normal);
    border-radius: 12px;
    padding: 16px;
  }

  .import-header {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    margin-bottom: 16px;
  }

  :global(.core-icon) {
    color: var(--accent-primary);
    opacity: 0.8;
  }

  .comp-name {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .comp-path {
    font-size: 10px;
    color: var(--text-muted);
    word-break: break-all;
    margin-top: 2px;
  }

  .config-actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-top: 12px;
    border-top: 1px solid var(--border-subtle);
  }

  .badge {
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 10px;
    font-weight: 600;
  }

  .badge.success { background: rgba(16, 185, 129, 0.1); color: #10b981; }
  .badge.gray { background: var(--surface-active); color: var(--text-muted); }

  .secondary-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    background: transparent;
    border: 1px solid var(--border-strong);
    border-radius: 6px;
    color: var(--text-primary);
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .secondary-btn:hover {
    background: var(--surface-active);
  }
</style>
