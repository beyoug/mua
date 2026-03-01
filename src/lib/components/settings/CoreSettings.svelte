<script lang="ts">
	import { FileCode, FileUp, RefreshCw, Copy, Eye, EyeOff, AlertCircle, RotateCcw } from '@lucide/svelte';
    import { aria2Config, configPath, isImporting, loadAria2Config, importAria2Config } from '$lib/services/aria2Config';
    import { appSettings, updateAppSettings } from '$lib/services/settings';
    import { onMount } from 'svelte';
    import { importAria2Binary, getAria2KernelVersionInfo, trustImportedAria2Binary } from '$lib/services/aria2';
    import type { Aria2VersionInfo } from '$lib/types/download';
    import { relaunch } from '@tauri-apps/plugin-process';
    import { getPlatformInfo } from '$lib/services/platform';
    import { createLogger } from '$lib/utils/logger';
    import { pickSingleFile } from '$lib/utils/dialog';
    import { confirmAction, showErrorFeedback, showSuccessFeedback } from '$lib/services/feedback';

    const logger = createLogger('CoreSettings');

    let isDirty = $state(false);
    let showSecret = $state(false);
    let aria2Version = $state<Aria2VersionInfo | null>(null);
    let isImportingKernel = $state(false);
    let currentPlatform = $state('');
    let restartRequired = $state(false);
    let lastAppliedCore = $state({
        rpcPort: 0,
        rpcSecret: '',
        useCustomAria2: false
    });

    onMount(() => {
        lastAppliedCore = {
            rpcPort: $appSettings.rpcPort,
            rpcSecret: $appSettings.rpcSecret ?? '',
            useCustomAria2: $appSettings.useCustomAria2
        };
        detectPlatform();
        loadVersionInfo();
    });

    async function detectPlatform() {
        try {
            const info = await getPlatformInfo();
            currentPlatform = info.os;
        } catch (e) {
            logger.warn('Failed to detect platform', { error: e });
            currentPlatform = '';
        }
    }

    async function loadVersionInfo() {
        try {
            aria2Version = await getAria2KernelVersionInfo();
            if ($appSettings.useCustomAria2 && aria2Version && !aria2Version.custom_binary_trusted) {
                $appSettings.useCustomAria2 = false;
            }
        } catch (e) {
            logger.error('Failed to load aria2 version info', { error: e });
        }
    }

    async function importKernel() {
        isImportingKernel = true;
        try {
            const path = await pickSingleFile('选择 Aria2 内核', [{
                name: 'Executable',
                extensions: currentPlatform === 'windows' ? ['exe'] : []
            }]);
            if (path) {
                const version = await importAria2Binary(path);
                await showSuccessFeedback('导入成功', `内核导入成功！\n版本: ${version}\n请手动开启"启用自定义内核"开关并重启应用以生效。`);
                await saveSettings();
                await loadVersionInfo();
            }
        } catch (e) {
            logger.error('Failed to import custom aria2 binary', { error: e });
            await showErrorFeedback('导入失败', e);
        } finally {
            isImportingKernel = false;
        }
    }

    async function restartApp() {
        const confirmed = await confirmAction('确定要重启应用以应用更改吗？', { title: '重启应用' });
        if (confirmed) {
            await relaunch();
        }
    }

    function onPortChange() {
        isDirty = true;
    }

    async function saveSettings() {
        const securityStatus = aria2Version?.custom_binary_security_status;

        if ($appSettings.useCustomAria2 && securityStatus === 'hash_mismatch') {
            await showErrorFeedback('启用失败', '当前自定义内核校验失败（哈希不匹配）。请重新导入后再启用。');
            $appSettings.useCustomAria2 = false;
            return;
        }

        if (
            $appSettings.useCustomAria2 &&
            (aria2Version?.custom_binary_exists ?? false) &&
            securityStatus !== 'trusted'
        ) {
            const trustConfirmed = await confirmAction(
                '你即将启用外部导入的 aria2 可执行文件。请确认该文件来源可信且未被篡改。是否继续信任并启用？',
                { title: '确认信任自定义内核' }
            );

            if (!trustConfirmed) {
                $appSettings.useCustomAria2 = false;
                return;
            }

            await trustImportedAria2Binary();
            $appSettings.customAria2Trusted = true;
            await loadVersionInfo();

            if (aria2Version?.custom_binary_security_status !== 'trusted') {
                await showErrorFeedback('启用失败', '自定义内核信任校验未通过，请重新导入后再试。');
                $appSettings.useCustomAria2 = false;
                return;
            }
        }

        const nextCore = {
            rpcPort: $appSettings.rpcPort,
            rpcSecret: $appSettings.rpcSecret ?? '',
            useCustomAria2: $appSettings.useCustomAria2
        };
        const hasRestartSensitiveChange =
            nextCore.rpcPort !== lastAppliedCore.rpcPort ||
            nextCore.rpcSecret !== lastAppliedCore.rpcSecret ||
            nextCore.useCustomAria2 !== lastAppliedCore.useCustomAria2;

        try {
            await updateAppSettings({
                rpcPort: $appSettings.rpcPort,
                rpcSecret: $appSettings.rpcSecret,
                useCustomAria2: $appSettings.useCustomAria2,
            });
            isDirty = false;
            if (hasRestartSensitiveChange) {
                restartRequired = true;
            }
            lastAppliedCore = nextCore;
        } catch (e) {
            logger.error('Failed to save core settings', { error: e });
        }
    }

    async function generateSecret() {
        const confirmed = await confirmAction(
            '重新生成密钥将导致当前连接断开，且需要重启应用生效。确定要继续吗？',
            { title: '重新生成密钥' }
        );
        if (confirmed) {
             $appSettings.rpcSecret = crypto.randomUUID();
             await saveSettings();
             isDirty = false;
        }
    }

    async function copySecret() {
        if ($appSettings.rpcSecret) {
            try {
                await navigator.clipboard.writeText($appSettings.rpcSecret);
            } catch (e) {
                logger.error('Failed to copy rpc secret', { error: e });
            }
        }
    }

    async function reTrustCustomBinary() {
        try {
            const confirmed = await confirmAction(
                '将重新计算并写入当前自定义内核哈希。仅在你确认文件未被篡改时继续。',
                { title: '重新建立自定义内核信任' }
            );
            if (!confirmed) return;

            await trustImportedAria2Binary();
            $appSettings.customAria2Trusted = true;
            await loadVersionInfo();
            await showSuccessFeedback('操作成功', '已完成自定义内核信任重建。');
        } catch (e) {
            logger.error('Failed to re-trust custom aria2 binary', { error: e });
            await showErrorFeedback('操作失败', e);
        }
    }

    const securityStatusText = $derived.by(() => {
        const status = aria2Version?.custom_binary_security_status;
        switch (status) {
            case 'trusted':
                return '可信';
            case 'untrusted':
                return '待信任';
            case 'hash_mismatch':
                return '哈希异常';
            default:
                return '未导入';
        }
    });

    const securityStatusClass = $derived.by(() => {
        const status = aria2Version?.custom_binary_security_status;
        switch (status) {
            case 'trusted':
                return 'status-safe';
            case 'untrusted':
                return 'status-warn';
            case 'hash_mismatch':
                return 'status-danger';
            default:
                return 'status-neutral';
        }
    });
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
            <button class="mini-btn ui-btn-icon ui-btn-focus ui-disabled" onclick={copySecret} title="复制"><Copy size={14} /></button>
            <button class="mini-btn ui-btn-icon ui-btn-focus ui-disabled" onclick={generateSecret} title="重新生成"><RefreshCw size={14} /></button>
          </div>
        </div>
      </div>

      {#if restartRequired}
        <div class="kernel-tip restart-tip">
          <AlertCircle size={12} />
          <span>RPC 端口 / 密钥 / 自定义内核开关已更新，需重启应用后生效。</span>
        </div>
      {/if}
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
                         <span class="badge warning ui-badge">自定义</span>
                     {:else}
                         <span class="badge gray ui-badge">内置</span>
                     {/if}
                 {:else}
                     <span class="loading-text">检测中...</span>
                 {/if}
              </div>
              <div class="security-state-row">
                <span class="security-label">内核安全状态</span>
                <span class={`security-badge ${securityStatusClass}`}>{securityStatusText}</span>
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
               
                <button class="mini-btn ui-btn-icon ui-btn-focus ui-disabled" onclick={restartApp} title="重启应用">
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

      {#if aria2Version?.custom_binary_exists && !aria2Version.custom_binary_trusted}
        <div class="kernel-tip warning-tip">
          <AlertCircle size={12} />
          <span>已检测到自定义内核，但尚未建立信任。启用时将要求你确认来源可信。</span>
        </div>
      {/if}

      {#if aria2Version?.custom_binary_exists && !aria2Version.custom_binary_hash_match}
        <div class="kernel-tip danger-tip">
          <AlertCircle size={12} />
          <span>检测到自定义内核哈希不匹配，系统将自动回退到内置内核。请重新导入并确认信任。</span>
        </div>

        <div class="kernel-remediation">
          <button class="secondary-btn" onclick={reTrustCustomBinary}>重新建立信任</button>
        </div>
      {/if}
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
          <div class="badge success ui-badge">已加载自定义配置</div>
        {:else}
          <div class="badge gray ui-badge">使用内置默认配置</div>
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
    border: none;
    border-radius: 6px;
    padding: 6px 8px;
    color: var(--text-primary);
    font-size: 12px;
    text-align: center;
  }

  .inner-input-field:focus-visible,
  .secret-box input:focus-visible,
  .small-save-btn:focus-visible,
  .secondary-btn:focus-visible,
  .icon-toggle:focus-visible,
  .mini-btn:focus-visible {
    outline: none;
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent-primary) 20%, transparent);
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
    border: none;
    border-radius: 8px;
    padding: 8px 36px 8px 12px;
    color: var(--text-primary);
    font-size: 12px;
    font-family: var(--font-mono);
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

  .icon-toggle:hover {
    color: var(--text-primary);
    background: var(--surface-hover);
    border-radius: 6px;
  }

  .side-actions {
    display: flex;
    gap: 4px;
  }

  .mini-btn {
    width: 32px;
    height: 32px;
  }

  /* Import Card */
  .import-card {
    background: var(--input-bg);
    border: none;
    border-radius: 12px;
    padding: 16px;
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--border-subtle) 44%, transparent);
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
    font-size: 11px;
    color: var(--text-muted);
    word-break: break-all;
    margin-top: 2px;
    opacity: 0.82;
  }

  .config-actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-top: 12px;
    border-top: none;
  }

  .badge {
    font-size: 11px;
    font-weight: 600;
  }

  .badge.success { background: color-mix(in srgb, var(--semantic-success) 10%, transparent); color: var(--semantic-success); }
  .badge.gray { background: var(--surface-active); color: var(--text-muted); }
  .badge.warning { background: color-mix(in srgb, var(--semantic-warning) 10%, transparent); color: var(--semantic-warning); }

  .secondary-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    min-height: 26px;
    padding: 4px 10px;
    background: var(--control-bg);
    border: none;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    box-shadow: var(--control-shadow-rest);
  }

  .secondary-btn:hover {
    background: var(--control-bg-hover);
    color: var(--text-primary);
  }

  .secondary-btn:disabled,
  .small-save-btn:disabled,
  .mini-btn:disabled {
    opacity: 0.55;
    cursor: not-allowed;
    transform: none;
    filter: none;
    box-shadow: none;
    pointer-events: none;
  }
  
  /* Kernel Card */
  .kernel-card {
    background: var(--input-bg);
    border: none;
    border-radius: 12px;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--border-subtle) 44%, transparent);
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
    font-family: var(--font-mono);
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
      border-top: none;
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
      background: var(--surface-hover);
      border-radius: 6px;
  }

  .restart-tip {
    margin-top: 10px;
    border-color: color-mix(in srgb, var(--semantic-warning) 30%, transparent);
    background: color-mix(in srgb, var(--semantic-warning) 8%, transparent);
  }

  .warning-tip {
    margin-top: 10px;
    border-color: color-mix(in srgb, var(--semantic-warning) 30%, transparent);
    background: color-mix(in srgb, var(--semantic-warning) 10%, transparent);
  }

  .danger-tip {
    margin-top: 10px;
    border-color: color-mix(in srgb, var(--semantic-danger) 34%, transparent);
    background: color-mix(in srgb, var(--semantic-danger) 10%, transparent);
  }

  .kernel-remediation {
    margin-top: 10px;
    display: flex;
    justify-content: flex-start;
  }

  .security-state-row {
    margin-top: 6px;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .security-label {
    font-size: 12px;
    color: var(--text-secondary);
  }

  .security-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 999px;
    padding: 2px 8px;
    font-size: 11px;
    font-weight: 600;
    border: none;
  }

  .status-safe {
    color: var(--semantic-success);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--semantic-success) 30%, transparent);
    background: color-mix(in srgb, var(--semantic-success) 10%, transparent);
  }

  .status-warn {
    color: var(--semantic-warning);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--semantic-warning) 30%, transparent);
    background: color-mix(in srgb, var(--semantic-warning) 10%, transparent);
  }

  .status-danger {
    color: var(--semantic-danger);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--semantic-danger) 30%, transparent);
    background: color-mix(in srgb, var(--semantic-danger) 10%, transparent);
  }

  .status-neutral {
    color: var(--text-secondary);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--border-subtle) 75%, transparent);
    background: var(--surface-hover);
  }
  
  :global(.spin) {
      animation: spin 1s linear infinite;
  }
  
  @keyframes spin {
      100% { transform: rotate(360deg); }
  }
</style>
