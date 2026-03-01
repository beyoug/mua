<!--
  BTSettings.svelte
  BT/磁力链接相关设置与高级配置
-->
<script lang="ts">
  import { appSettings, updateAppSettings } from '$lib/services/settings';
	import { ChevronUp, ChevronDown, RefreshCw, Plus, Info } from '@lucide/svelte';
  import { fade } from 'svelte/transition';
  import { relaunch } from '@tauri-apps/plugin-process';
  import { createLogger } from '$lib/utils/logger';
  import { fetchTrackers as fetchTrackersService } from '$lib/services/aria2';

  const logger = createLogger('BTSettings');

  // 本地状态同步
  let btTrackers = $state($appSettings.btTrackers || '');
  let enableDht = $state($appSettings.enableDht);
  let enablePeerExchange = $state($appSettings.enablePeerExchange);
  let enableSeeding = $state($appSettings.enableSeeding ?? true);
  let globalMaxUploadLimit = $state($appSettings.globalMaxUploadLimit || '');
  let seedRatio = $state($appSettings.seedRatio);
  let dhtListenPort = $state($appSettings.dhtListenPort || '6881');
  let listenPort = $state($appSettings.listenPort || '6881');

  let isFetchingTrackers = $state(false);
  let publicTrackers: string[] = $state([]);
  let showTrackerPreview = $state(false);

  // 跟踪原始端口以检测变更
  const initialDhtPort = $appSettings.dhtListenPort;
  const initialListenPort = $appSettings.listenPort;

  let isDhtPortChanged = $derived(dhtListenPort !== initialDhtPort);
  let isListenPortChanged = $derived(listenPort !== initialListenPort);

  // 监听 store 变化，防止未保存状态被覆盖
  $effect(() => {
    // 简单单向同步初始化
    if (!btTrackers && $appSettings.btTrackers) btTrackers = $appSettings.btTrackers;
  });

  async function handleSave() {
    try {
      await updateAppSettings({
        btTrackers,
        enableDht,
        enablePeerExchange,
        enableSeeding,
        globalMaxUploadLimit,
        seedRatio,
        dhtListenPort,
        listenPort
      });
    } catch (e) {
      logger.error('Failed to save BT settings', { error: e });
    }
  }

  async function fetchTrackers() {
      if (isFetchingTrackers) return;
      isFetchingTrackers = true;
      try {
          publicTrackers = await fetchTrackersService();
          showTrackerPreview = true;
      } catch (e) {
          logger.error('Failed to fetch public trackers', { error: e });
      } finally {
          isFetchingTrackers = false;
      }
  }

  function appendTrackers() {
      if (publicTrackers.length === 0) return;
      const newTrackers = publicTrackers.join('\n');
      if (btTrackers.trim()) {
          btTrackers = btTrackers.trim() + '\n' + newTrackers;
      } else {
          btTrackers = newTrackers;
      }
      showTrackerPreview = false;
      handleSave();
  }

  function isPortTaken(port: string, currentType: 'dht' | 'bt' | 'rpc') {
      const p = port.trim();
      const rpc = $appSettings.rpcPort.toString();
      
      if (currentType !== 'rpc' && p === rpc) return true;
      if (currentType !== 'dht' && p === dhtListenPort) return true;
      if (currentType !== 'bt' && p === listenPort) return true;
      
      return false;
  }

  function handleBlur() {
      // 最终校验：如果手动输入导致冲突，自动回滚或修正
      if (isPortTaken(dhtListenPort, 'dht')) {
          logger.warn('DHT port conflict detected, reverting...');
          dhtListenPort = initialDhtPort;
      }
      if (isPortTaken(listenPort, 'bt')) {
          logger.warn('BT port conflict detected, reverting...');
          listenPort = initialListenPort;
      }
      handleSave();
  }


  function adjustPort(type: 'dht' | 'bt', delta: number) {
      if (type === 'dht') {
          let current = parseInt(dhtListenPort) || 6881;
          let nextPort = (current + delta).toString();
          
          // 自动跳过冲突端口
          while (isPortTaken(nextPort, 'dht')) {
              current += delta;
              nextPort = (current + delta).toString();
          }
          dhtListenPort = nextPort;
      } else {
          let current = parseInt(listenPort) || 6881;
          let nextPort = (current + delta).toString();
          
          // 自动跳过冲突端口
          while (isPortTaken(nextPort, 'bt')) {
              current += delta;
              nextPort = (current + delta).toString();
          }
          listenPort = nextPort;
      }
      handleSave();
  }
</script>

<div class="settings-container bt-settings">
  <section class="settings-section bt-section">
    <h4 class="section-title">基础网络</h4>

    <div class="setting-list">
      
      <div class="setting-item">
        <div class="setting-info">
          <div class="setting-header-row">
            <div class="setting-title-stack">
              <div class="setting-title-row">
                <div class="setting-name">启用 DHT 网络</div>
                <button type="button" class="tooltip-trigger" aria-label="DHT 说明">
                  <Info size={14} />
                  <span class="tooltip-text">允许从 DHT 网络获取节点，有助于无 Tracker 下载。</span>
                </button>
              </div>

            </div>
            <div class="port-input-inline" class:disabled={!enableDht}>
                <span class="port-label">DHT 端口</span>
                <input 
                    type="text" 
                    bind:value={dhtListenPort} 
                    onblur={handleBlur} 
                    placeholder="6881" 
                    title="DHT 监听端口"
                    disabled={!enableDht}
                >
                <div class="port-controls">
                    <button class="port-btn" onclick={() => adjustPort('dht', 1)} title="增加" disabled={!enableDht}>
                        <ChevronUp size={10} />
                    </button>
                    <button class="port-btn" onclick={() => adjustPort('dht', -1)} title="减少" disabled={!enableDht}>
                        <ChevronDown size={10} />
                    </button>
                </div>
            </div>
          </div>

          {#if isDhtPortChanged}
            <div class="inline-restart-hint ui-status-card warning" transition:fade={{ duration: 150 }}>
                <RefreshCw size={12} />
                <span>端口已改，需重启生效</span>
                <button class="text-relaunch-btn" onclick={() => relaunch()}>立即重启</button>
            </div>
          {/if}
        </div>
        <div class="setting-control">
            <label class="switch">
            <input type="checkbox" bind:checked={enableDht} onchange={handleSave}>
            <span class="slider"></span>
            </label>
        </div>
      </div>

      <!-- 用户交换 (PEX) -->
      <div class="setting-item">
        <div class="setting-info">
          <div class="setting-header-row">
            <div class="setting-title-stack">
              <div class="setting-title-row">
                <div class="setting-name">启用用户交换 (PEX)</div>
                <button type="button" class="tooltip-trigger" aria-label="PEX 说明">
                  <Info size={14} />
                  <span class="tooltip-text">允许与连接的节点交换信息。监听端口用于接收连接 (TCP)。</span>
                </button>
              </div>

            </div>
             <div class="port-input-inline" class:disabled={!enablePeerExchange}>
                <span class="port-label">监听端口</span>
                <input 
                    type="text" 
                    bind:value={listenPort} 
                    onblur={handleBlur} 
                    placeholder="6881" 
                    title="BT 监听端口"
                    disabled={!enablePeerExchange}
                >
                <div class="port-controls">
                    <button class="port-btn" onclick={() => adjustPort('bt', 1)} title="增加" disabled={!enablePeerExchange}>
                        <ChevronUp size={10} />
                    </button>
                    <button class="port-btn" onclick={() => adjustPort('bt', -1)} title="减少" disabled={!enablePeerExchange}>
                        <ChevronDown size={10} />
                    </button>
                </div>
            </div>
          </div>

          {#if isListenPortChanged}
            <div class="inline-restart-hint ui-status-card warning" transition:fade={{ duration: 150 }}>
                <RefreshCw size={12} />
                <span>端口已改，需重启生效</span>
                <button class="text-relaunch-btn" onclick={() => relaunch()}>立即重启</button>
            </div>
          {/if}
        </div>
        <div class="setting-control">
            <label class="switch">
            <input type="checkbox" bind:checked={enablePeerExchange} onchange={handleSave}>
            <span class="slider"></span>
            </label>
        </div>
      </div>
    </div>
  </section>

  <section class="settings-section bt-section">
    <h4 class="section-title">做种策略</h4>

    <div class="setting-list">
        <!-- 启用做种 -->
        <div class="setting-item">
            <div class="setting-info">
              <div class="setting-title-row">
                <div class="setting-name">允许做种 (Seeding)</div>
                <button type="button" class="tooltip-trigger" aria-label="做种说明">
                  <Info size={14} />
                  <span class="tooltip-text">关闭后，任务下载完成后将立即停止，不再上传。</span>
                </button>
              </div>

            </div>
            <div class="setting-control">
                <label class="switch">
                <input type="checkbox" bind:checked={enableSeeding} onchange={handleSave}>
                <span class="slider"></span>
                </label>
            </div>
        </div>

        {#if enableSeeding}
        <div class="setting-item" transition:fade={{ duration: 150 }}>
            <div class="setting-info">
              <div class="setting-title-row">
                <div class="setting-name">分享率限制 (Seed Ratio)</div>
                <button type="button" class="tooltip-trigger" aria-label="分享率说明">
                  <Info size={14} />
                  <span class="tooltip-text">当分享率达到此值时停止做种 (0 为不限制)。建议设置为 1.0。</span>
                </button>
              </div>

            </div>
            <div class="input-wrapper ratio-input">
                <input type="number" step="0.1" min="0" bind:value={seedRatio} onchange={handleBlur}>
            </div>
        </div>
        {/if}

        <div class="setting-item">
            <div class="setting-info">
              <div class="setting-title-row">
                <div class="setting-name">全局最大上传速度</div>
                <button type="button" class="tooltip-trigger" aria-label="上传限速说明">
                  <Info size={14} />
                  <span class="tooltip-text">限制全局上传速率（单位 MB/s），防止占用过多带宽。0 表示不限制。</span>
                </button>
              </div>

            </div>
            <div class="input-wrapper group">
              <input 
                type="text" 
                placeholder="0" 
                bind:value={globalMaxUploadLimit}
                onblur={handleBlur}
              />
              <span class="unit">MB/s</span>
            </div>
        </div>
    </div>
  </section>

  <section class="settings-section bt-section">
    <h4 class="section-title">Trackers 管理</h4>

    <div class="setting-list">
      
      <!-- BT Trackers -->
      <div class="setting-item vertical tracker-item">
        <div class="tracker-layout">
          <div class="tracker-zone tracker-zone-header">
            <div class="tracker-heading">
              <div class="setting-title-row">
                <div class="setting-name">BT Trackers</div>
                <button type="button" class="tooltip-trigger" aria-label="Tracker 说明">
                  <Info size={14} />
                  <span class="tooltip-text">
                    自定义 Tracker 服务器列表，每行一个。更改将在下载新磁力链接或种子时生效。
                  </span>
                </button>
              </div>
              <p class="tracker-description">维护连接源列表，提高新建 BT 任务的可用连接率。</p>
            </div>
            <div class="tracker-actions">
              <button class="action-btn" onclick={fetchTrackers} disabled={isFetchingTrackers}>
                {#if isFetchingTrackers}
                  <RefreshCw size={14} style="animation: spin 1s linear infinite;" />
                {:else}
                  <RefreshCw size={14} />
                {/if}
                获取公共列表
              </button>
            </div>
          </div>

          {#if showTrackerPreview}
            <div class="tracker-zone tracker-zone-preview" transition:fade={{ duration: 150 }}>
              <div class="preview-header">
                <span class="preview-summary">发现 {publicTrackers.length} 个新服务器</span>
                <div class="preview-btns">
                  <button class="mini-btn ui-btn-mini ui-btn-primary ui-btn-focus ui-disabled append" onclick={appendTrackers}>
                    <Plus size={12} />
                    追加
                  </button>
                  <button class="mini-btn ui-btn-mini ui-btn-secondary ui-btn-focus ui-disabled cancel" onclick={() => showTrackerPreview = false}>
                    取消
                  </button>
                </div>
              </div>
              <div class="preview-content">
                {publicTrackers.slice(0, 5).join('\n')}
                {#if publicTrackers.length > 5}
                  {"\n"}... 还有 {publicTrackers.length - 5} 个
                {/if}
              </div>
            </div>
          {/if}

          <div class="tracker-zone tracker-zone-editor">
            <div class="editor-caption">Tracker 列表（每行一个 URL）</div>
            <div class="textarea-wrapper">
              <textarea
                class="tracker-editor-input"
                bind:value={btTrackers}
                onblur={handleBlur}
                placeholder="udp://tracker.opentrackr.org:1337/announce..."
                spellcheck="false"
              ></textarea>
            </div>
          </div>
        </div>
      </div>

    </div>
  </section>



</div>

<style>
  .bt-settings {
    gap: 14px;
  }

  .bt-section {
    position: relative;
  }

  .setting-item {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: center;
    column-gap: 14px;
  }

  .setting-item.vertical {
    display: flex;
    flex-direction: column;
    align-items: stretch;
    gap: 10px;
  }

  .setting-info {
    flex: 1;
    min-width: 0;
  }

  .setting-title-stack {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
  }

  .setting-title-row {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
    min-height: 32px;
    line-height: 1.3;
  }

  .setting-name {
    display: inline-flex;
    align-items: center;
    min-height: 32px;
  }

  .setting-header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    flex-wrap: nowrap;
    width: 100%;
  }

  .setting-control {
    display: flex;
    align-items: center;
    align-self: center;
    padding-top: 0;
  }

  .port-input-inline {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    min-height: 32px;
    padding: 4px 6px 4px 10px;
    border-radius: 10px;
    background: color-mix(in srgb, var(--control-bg) 92%, transparent);
    box-shadow: var(--control-shadow-rest);
    transition: background-color 0.2s ease, box-shadow 0.2s ease;
    margin-left: auto;
  }

  .port-input-inline:hover {
    background: var(--control-bg-hover);
  }

  .port-input-inline:focus-within {
    background: var(--control-bg-hover);
    box-shadow: var(--focus-ring);
  }

  .port-input-inline.disabled {
    opacity: 0.5;
    pointer-events: none;
    filter: saturate(0.2);
  }

  .port-label {
    font-size: 10px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--text-muted);
    font-weight: 600;
    white-space: nowrap;
  }

  .port-input-inline input {
    width: 58px;
    height: 22px;
    padding: 0;
    border: none;
    outline: none;
    background: transparent;
    color: var(--text-primary);
    font-size: 12px;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    text-align: center;
  }

  .port-controls {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .port-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 10px;
    padding: 0;
    border: none;
    border-radius: 4px;
    background: color-mix(in srgb, var(--surface-active) 88%, transparent);
    color: var(--text-muted);
    cursor: pointer;
    transition: background-color 0.15s ease, color 0.15s ease;
  }

  .port-btn:hover {
    background: color-mix(in srgb, var(--accent-primary) 16%, transparent);
    color: var(--text-primary);
  }

  .port-btn:active {
    transform: translateY(1px);
  }

  .inline-restart-hint {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    margin-top: 8px;
    width: fit-content;
  }

  .text-relaunch-btn {
    border: none;
    border-radius: 999px;
    padding: 2px 9px;
    font-size: 11px;
    font-weight: 600;
    color: #111827;
    background: color-mix(in srgb, var(--semantic-warning) 86%, #ffffff);
    cursor: pointer;
    transition: filter 0.2s ease, transform 0.2s ease;
  }

  .text-relaunch-btn:hover {
    filter: brightness(1.03);
    transform: translateY(-1px);
  }

  .input-wrapper {
    display: inline-flex;
    align-items: center;
    align-self: center;
  }

  .input-wrapper input {
    width: 88px;
    min-height: 32px;
    padding: 6px 10px;
    border: none;
    border-radius: 10px;
    outline: none;
    background: var(--control-bg);
    color: var(--text-primary);
    font-size: 12px;
    text-align: right;
    box-shadow: var(--control-shadow-rest);
    transition: background-color 0.2s ease, box-shadow 0.2s ease;
  }

  .input-wrapper input:hover {
    background: var(--control-bg-hover);
  }

  .input-wrapper input:focus,
  .input-wrapper input:focus-visible {
    box-shadow: var(--focus-ring);
  }

  .ratio-input input {
    width: 96px;
  }

  .input-wrapper.group {
    overflow: hidden;
    border-radius: 10px;
    box-shadow: var(--control-shadow-rest);
    background: var(--control-bg);
  }

  .input-wrapper.group input {
    border-radius: 0;
    box-shadow: none;
    width: 72px;
  }

  .unit {
    display: inline-flex;
    align-items: center;
    min-height: 32px;
    padding: 0 10px;
    font-size: 11px;
    color: var(--text-muted);
    background: color-mix(in srgb, var(--surface-active) 86%, transparent);
  }

  .tracker-item {
    padding-top: 8px;
    padding-bottom: 10px;
  }

  .tracker-layout {
    display: grid;
    gap: 12px;
    width: 100%;
  }

  .tracker-zone {
    width: 100%;
    border-radius: 12px;
  }

  .tracker-zone-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 14px;
    padding: 10px 12px;
    background:
      linear-gradient(125deg, color-mix(in srgb, var(--surface-active) 88%, transparent), color-mix(in srgb, var(--control-bg) 92%, transparent));
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--border-subtle) 72%, transparent);
  }

  .tracker-heading {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .tracker-description {
    margin: 0;
    font-size: 11px;
    line-height: 1.45;
    color: var(--text-muted);
  }

  .tracker-actions {
    display: inline-flex;
    flex-shrink: 0;
    align-self: center;
  }

  .action-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    min-height: 32px;
    padding: 6px 12px;
    border: none;
    border-radius: 10px;
    font-size: 12px;
    color: var(--text-secondary);
    background: var(--control-bg);
    box-shadow: var(--control-shadow-rest);
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .action-btn:hover:not(:disabled) {
    background: var(--control-bg-hover);
    color: var(--text-primary);
  }

  .action-btn:disabled {
    opacity: 0.55;
    cursor: not-allowed;
    box-shadow: none;
  }

  .tracker-zone-preview {
    width: 100%;
    padding: 10px;
    background: color-mix(in srgb, var(--surface-active) 88%, transparent);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--accent-primary) 20%, transparent);
  }

  .preview-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    margin-bottom: 8px;
  }

  .preview-summary {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .preview-btns {
    display: flex;
    gap: 6px;
  }

  .mini-btn {
    gap: 4px;
  }

  .mini-btn.append {
    color: #fff;
  }

  .preview-content {
    max-height: 108px;
    overflow-y: auto;
    padding: 9px;
    border-radius: 8px;
    background: color-mix(in srgb, var(--input-bg) 90%, transparent);
    font-size: 11px;
    line-height: 1.45;
    color: var(--text-muted);
    white-space: pre-wrap;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  }

  .tracker-zone-editor {
    display: grid;
    gap: 8px;
  }

  .editor-caption {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
    letter-spacing: 0.01em;
  }

  .textarea-wrapper {
    width: 100%;
  }

  .tracker-editor-input {
    width: 100%;
    min-height: 160px;
    border: none;
    border-radius: 12px;
    padding: 12px 14px;
    box-sizing: border-box;
    resize: vertical;
    outline: none;
    background: var(--control-bg);
    color: var(--text-primary);
    font-size: 12px;
    line-height: 1.5;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    box-shadow: var(--control-shadow-rest);
    transition: background-color 0.2s ease, box-shadow 0.2s ease;
  }

  .tracker-editor-input:hover {
    background: var(--control-bg-hover);
  }

  .tracker-editor-input:focus,
  .tracker-editor-input:focus-visible {
    box-shadow: var(--focus-ring);
  }

  .tooltip-trigger {
    position: relative;
    display: inline-flex;
    align-items: center;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--text-muted);
    cursor: help;
    transition: color 0.2s ease;
  }

  .tooltip-trigger:hover,
  .tooltip-trigger:focus-visible {
    color: var(--accent-primary);
  }

  .tooltip-text {
    position: absolute;
    z-index: 20;
    left: 50%;
    bottom: calc(100% + 8px);
    transform: translateX(-50%);
    width: 216px;
    padding: 8px 10px;
    border-radius: 10px;
    background: color-mix(in srgb, var(--glass-menu-bg) 92%, transparent);
    color: var(--text-primary);
    font-size: 11px;
    font-weight: 400;
    line-height: 1.45;
    box-shadow: var(--glass-shadow), 0 8px 18px rgba(4, 18, 42, 0.2);
    opacity: 0;
    visibility: hidden;
    transition: opacity 0.2s ease;
    pointer-events: none;
  }

  .tooltip-trigger:hover .tooltip-text,
  .tooltip-trigger:focus-visible .tooltip-text {
    opacity: 1;
    visibility: visible;
  }

  .port-btn:focus-visible,
  .action-btn:focus-visible,
  .mini-btn:focus-visible,
  .text-relaunch-btn:focus-visible,
  .tooltip-trigger:focus-visible {
    outline: none;
    box-shadow: var(--focus-ring);
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  @media (max-width: 760px) {
    .setting-item {
      grid-template-columns: 1fr;
      row-gap: 10px;
    }

    .setting-control {
      justify-content: flex-end;
    }

    .setting-header-row {
      align-items: flex-start;
      flex-wrap: wrap;
    }

    .port-input-inline {
      margin-left: 0;
    }

    .tracker-zone-header {
      flex-direction: column;
      align-items: stretch;
      gap: 10px;
    }

    .tracker-actions {
      width: 100%;
    }

    .tracker-actions .action-btn {
      width: 100%;
      justify-content: center;
    }

    .preview-header {
      flex-direction: column;
      align-items: flex-start;
      gap: 8px;
    }

    .preview-btns {
      width: 100%;
      flex-wrap: wrap;
    }

    .tracker-editor-input {
      min-height: 140px;
    }
  }
</style>
