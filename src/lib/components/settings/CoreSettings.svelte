<script lang="ts">
    import { FileCode, FileUp, Key, RefreshCw, Copy, Eye, EyeOff, AlertCircle, RotateCcw } from '@lucide/svelte';
    import { aria2Config, configPath, isImporting, loadAria2Config, importAria2Config } from '$lib/stores/aria2Config';
    import { appSettings, updateAppSettings } from '$lib/stores/settings';
    import { onMount } from 'svelte';
    import { importCustomBinary, getAria2VersionInfo } from '$lib/api/cmd';
    import type { Aria2VersionInfo } from '$lib/types/download';
    import { open as openDialog } from '@tauri-apps/plugin-dialog';
    import { relaunch } from '@tauri-apps/plugin-process';

    let isDirty = $state(false);
    let showSecret = $state(false);
    let aria2Version = $state<Aria2VersionInfo | null>(null);
    let isImportingKernel = $state(false);

    onMount(() => {
        loadVersionInfo();
    });

    async function loadVersionInfo() {
        try {
            aria2Version = await getAria2VersionInfo();
        } catch (e) {
            console.error(e);
        }
    }

    async function importKernel() {
        isImportingKernel = true;
        try {
            const selected = await openDialog({
                filters: [{
                    name: 'Executable',
                    extensions: window.navigator.userAgent.includes('Win') ? ['exe'] : []
                }],
                multiple: false
            });

            if (selected) {
                 const path = typeof selected === 'string' ? selected : (selected as any).path;
                 if (path) {
                     const version = await importCustomBinary(path);
                     alert(`内核导入成功！\n版本: ${version}\n请手动开启"启用自定义内核"开关并重启应用以生效。`);
                     await saveSettings();
                     await loadVersionInfo();
                 }
            }
        } catch (e) {
            console.error(e);
            alert('导入失败: ' + e);
        } finally {
            isImportingKernel = false;
        }
    }

    async function restartApp() {
        if (confirm('确定要重启应用以应用更改吗？')) {
            await relaunch();
        }
    }

    function onPortChange() {
        isDirty = true;
    }

    async function saveSettings() {
        try {
            await updateAppSettings({
                rpcPort: $appSettings.rpcPort,
                rpcSecret: $appSettings.rpcSecret,
                useCustomAria2: $appSettings.useCustomAria2,
            });
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
    <h4 class="section-title">内核管理</h4>
    
    <div class="kernel-card">
      <div class="kernel-header">
         <div class="kernel-info">
             <div class="kernel-label">当前内核版本</div>
             <div class="kernel-value">
                 {#if aria2Version}
                     <span class="version-text">{aria2Version.version}</span>
                     {#if aria2Version.is_custom}
                        <span class="badge warning">自定义</span>
                     {:else}
                        <span class="badge gray">内置</span>
                     {/if}
                 {:else}
                     <span class="loading-text">检测中...</span>
                 {/if}
             </div>
             {#if aria2Version?.path}
                <div class="kernel-path" title={aria2Version.path}>{aria2Version.path}</div>
             {/if}
         </div>
      </div>
      
      <div class="kernel-actions">
           <div class="toggle-row">
               <span class="toggle-label">启用自定义内核</span>
               <label class="switch">
                 <input 
                    type="checkbox" 
                    bind:checked={$appSettings.useCustomAria2} 
                    disabled={!aria2Version?.custom_binary_exists}
                    onchange={async () => {
                        await saveSettings();
                        loadVersionInfo();
                    }}
                 />
                 <span class="slider"></span>
               </label>
               
               <button class="mini-btn" onclick={restartApp} title="重启应用">
                   <RotateCcw size={14} />
               </button>
           </div>
           
           <button class="secondary-btn" onclick={importKernel} disabled={isImportingKernel}>
              {#if isImportingKernel}
                  <RefreshCw size={14} class="spin" />
              {:else}
                  <FileUp size={14} />
              {/if}
              <span>导入新内核...</span>
           </button>
      </div>
      
      <div class="kernel-tip">
          <AlertCircle size={12} />
          <span>支持导入外部 aria2c 可执行文件 (v1.35.0+)。需要重启应用生效。</span>
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
  /* 组件特有样式 */
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
  .badge.warning { background: rgba(245, 158, 11, 0.1); color: #f59e0b; }

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
  
  /* Kernel Card */
  .kernel-card {
    background: var(--input-bg);
    border: 1px solid var(--border-normal);
    border-radius: 12px;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  
  .kernel-header {
      display: flex;
      justify-content: space-between;
      align-items: flex-start;
  }
  
  .kernel-info {
      display: flex;
      flex-direction: column;
      gap: 4px;
      overflow: hidden;
  }
  
  .kernel-label {
      font-size: 11px;
      color: var(--text-muted);
  }
  
  .kernel-value {
      display: flex;
      align-items: center;
      gap: 8px;
  }
  
  .version-text {
      font-size: 14px;
      font-weight: 600;
      color: var(--text-primary);
      font-family: 'JetBrains Mono', monospace;
  }
  
  .kernel-path {
      font-size: 10px;
      color: var(--text-muted);
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      max-width: 300px;
      opacity: 0.7;
  }
  
  .kernel-actions {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding-top: 12px;
      border-top: 1px solid var(--border-subtle);
  }
  
  .toggle-row {
      display: flex;
      align-items: center;
      gap: 8px;
  }
  
  .toggle-label {
      font-size: 12px;
      color: var(--text-secondary);
  }
  
  .kernel-tip {
      display: flex;
      align-items: center;
      gap: 6px;
      font-size: 10px;
      color: var(--text-tertiary);
      padding: 8px 12px;
      background: var(--surface-ground);
      border-radius: 6px;
  }
  
  :global(.spin) {
      animation: spin 1s linear infinite;
  }
  
  @keyframes spin {
      100% { transform: rotate(360deg); }
  }
</style>
