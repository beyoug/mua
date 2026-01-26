<script lang="ts">
    import { Check, FileCode, FileUp, Key, RefreshCw, Copy, Eye, EyeOff } from '@lucide/svelte';
    import { aria2Config, configPath, isImporting, loadAria2Config, importAria2Config } from '$lib/stores/aria2Config';
    import { appSettings, loadAppSettings, saveAppSettings } from '$lib/stores/settings';

    let isDirty = $state(false);
    
    function onPortChange() {
        isDirty = true;
    }

    async function saveSettings() {
        try {
            await saveAppSettings($appSettings);
            isDirty = false;
            // Optional: Toast success
        } catch (e) {
            alert('保存失败');
        }
    }

    let showSecret = $state(false);

    function generateSecret() {
        if (confirm('重新生成密钥将导致当前连接断开，且需要重启应用生效。确定要继续吗？')) {
             $appSettings.rpcSecret = crypto.randomUUID();
             saveSettings();
             isDirty = false; // Saved immediately
        }
    }

    async function copySecret() {
        if ($appSettings.rpcSecret) {
            try {
                await navigator.clipboard.writeText($appSettings.rpcSecret);
                // Optional: Toast
            } catch (e) {
                console.error("Copy failed", e);
            }
        }
    }
</script>

<section class="settings-section">
    <div class="settings-group">
        <!-- Helper Text -->
            <div class="input-helper">
            <label for="rpc-port">RPC 监听端口</label>
            <span class="helper-text">修改后将在下次启动时生效</span>
            </div>

            <div class="input-row">
                <input 
                id="rpc-port"
                type="number" 
                min="1024" 
                max="65535"
                class="port-input"
                bind:value={$appSettings.rpcPort}
                onchange={onPortChange}
                />
                <button class="save-btn" onclick={saveSettings} disabled={!isDirty}>
                    <Check size={14} />
                    保存
                </button>
            </div>
    </div>
    


    <div class="divider"></div>

        <div class="settings-group">
            <div class="input-helper">
            <label for="rpc-secret">RPC 密钥 (Secret)</label>
            <span class="helper-text error-text">修改后需要重启应用生效</span>
            </div>

            <div class="secret-row">
                <div class="secret-input-wrapper">
                    <input 
                    id="rpc-secret"
                    type={showSecret ? "text" : "password"}
                    class="secret-input"
                    bind:value={$appSettings.rpcSecret}
                    onchange={onPortChange}
                    placeholder="未设置"
                    />
                    <button class="icon-btn" onclick={() => showSecret = !showSecret} title={showSecret ? "隐藏" : "显示"}>
                        {#if showSecret}
                            <EyeOff size={14} />
                        {:else}
                            <Eye size={14} />
                        {/if}
                    </button>
                </div>
                
                <button class="action-btn" onclick={copySecret} title="复制">
                    <Copy size={14} />
                </button>
                
                <button class="action-btn" onclick={generateSecret} title="重新生成随机密钥">
                    <RefreshCw size={14} />
                </button>
            </div>
            
            <div class="save-row">
                <button class="save-btn" onclick={saveSettings} disabled={!isDirty}>
                    <Check size={14} />
                    保存
                </button>
            </div>
    </div>

    <div class="divider"></div>

    <div class="section-header">
        <FileCode size={16} />
        <span>Aria2 配置文件</span>
    </div>
    
    <div class="import-panel">
        <div class="config-status">
            {#if $aria2Config}
                <div class="status-indicator active">
                    <Check size={14} />
                    <span>已加载自定义配置</span>
                </div>
                <div class="config-preview">
                    {$aria2Config}
                </div>
            {:else}
                <div class="status-indicator">
                    <span>未检测到自定义配置文件</span>
                </div>
            {/if}
        </div>

        <div class="action-row">
            <span class="config-path" title={$configPath}>
                {$configPath ? $configPath : '初始化路径中...'}
            </span>
            <button class="import-btn" onclick={importAria2Config} disabled={$isImporting}>
                <FileUp size={14} />
                {$isImporting ? '导入中...' : '导入配置文件'}
            </button>
        </div>
    </div>
    <p class="section-hint">
        选择本地的 <code>aria2.conf</code> 文件导入。
        <br/>
        注意：导入将覆盖现有配置，且需要重启应用生效。
    </p>
</section>

<style>
	.settings-section {
		margin-bottom: 20px;
	}

    .settings-group {
        display: flex;
        flex-direction: column;
        gap: 10px;
        background: var(--input-bg);
        border: 1px solid var(--border-normal);
        padding: 12px;
        border-radius: 12px;
        margin-bottom: 20px;
    }

	.section-header {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 12px;
		font-weight: 500;
		color: var(--text-secondary);
		margin-bottom: 10px;
		text-transform: uppercase;
		letter-spacing: 0.4px;
	}

    .input-helper {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .input-helper label {
        font-size: 13px;
        color: var(--text-primary);
        font-weight: 500;
    }
    
    .helper-text {
        font-size: 11px;
        color: var(--text-muted);
    }
    
    .error-text {
        color: var(--warning, #eab308);
    }

    .input-row {
        display: flex;
        gap: 8px;
    }

    .port-input {
        flex: 1;
        background: var(--bg-hover);
        border: 1px solid var(--border-subtle);
        border-radius: 8px;
        padding: 8px 12px;
        color: var(--text-primary);
        font-size: 13px;
    }

    .port-input:focus {
        outline: none;
        border-color: var(--accent-primary);
        box-shadow: 0 0 0 2px var(--accent-subtle);
    }

    .save-btn {
        display: flex;
        align-items: center;
        gap: 6px;
        padding: 0 16px;
        background: var(--accent-primary);
        color: white;
        border: none;
        border-radius: 8px;
        font-size: 13px;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.2s;
    }

    .save-btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
        background: var(--border-strong);
    }
    
    .divider {
        height: 1px;
        background: var(--border-subtle);
        margin: 20px 0;
    }

    /* Secret Row */
    .save-row {
        display: flex;
        justify-content: flex-end;
        padding-top: 8px;
    }

    .secret-row {
        display: flex;
        gap: 8px;
        align-items: center;
    }

    .secret-input-wrapper {
        flex: 1;
        position: relative;
        display: flex;
        align-items: center;
    }

    .secret-input {
        width: 100%;
        background: var(--bg-hover);
        border: 1px solid var(--border-subtle);
        border-radius: 8px;
        padding: 8px 36px 8px 12px; /* Space for eye icon */
        color: var(--text-primary);
        font-size: 13px;
        font-family: 'JetBrains Mono', monospace;
    }
    
    .secret-input:focus {
        outline: none;
        border-color: var(--accent-primary);
        box-shadow: 0 0 0 2px var(--accent-subtle);
    }

    .icon-btn {
        position: absolute;
        right: 8px;
        background: transparent;
        border: none;
        color: var(--text-muted);
        cursor: pointer;
        padding: 4px;
        display: flex;
        align-items: center;
        transition: color 0.2s;
    }

    .icon-btn:hover {
        color: var(--text-primary);
    }

    .action-btn {
        padding: 8px;
        background: var(--input-bg);
        border: 1px solid var(--border-normal);
        border-radius: 8px;
        color: var(--text-secondary);
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: all 0.2s;
    }

    .action-btn:hover {
        background: var(--bg-hover);
        color: var(--text-primary);
        border-color: var(--border-strong);
    }
    
    /* Import Panel */
    .import-panel {
        display: flex;
        flex-direction: column;
        gap: 12px;
        padding: 12px;
        background: var(--input-bg);
        border: 1px solid var(--border-color);
        border-radius: 12px;
    }

    .config-status {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .status-indicator {
        display: flex;
        align-items: center;
        gap: 6px;
        font-size: 13px;
        color: var(--text-secondary);
    }

    .status-indicator.active {
        color: var(--success, #10b981);
    }

    .config-preview {
        max-height: 80px;
        overflow-y: auto;
        padding: 8px;
        background: var(--bg-hover);
        border-radius: 6px;
        font-family: 'JetBrains Mono', monospace;
        font-size: 11px;
        color: var(--text-muted);
        white-space: pre-wrap;
        border: 1px solid var(--border-subtle);
    }

    .action-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 12px;
        padding-top: 8px;
        border-top: 1px solid var(--border-subtle);
    }

    .config-path {
        font-size: 11px;
        color: var(--text-muted);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        flex: 1;
        min-width: 0;
    }

    .import-btn {
        display: flex;
        align-items: center;
        gap: 6px;
        padding: 6px 12px;
        font-size: 12px;
        color: white;
        background: var(--accent-primary);
        border: none;
        border-radius: 6px;
        cursor: pointer;
        transition: opacity 0.2s;
        white-space: nowrap;
    }

    .import-btn:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }

    .section-hint {
        margin-top: 8px;
        font-size: 12px;
        color: var(--text-muted);
        line-height: 1.4;
    }
    
    .section-hint code {
        background: var(--border-light);
        padding: 2px 4px;
        border-radius: 4px;
        font-family: inherit;
        font-size: 11px;
    }
</style>
