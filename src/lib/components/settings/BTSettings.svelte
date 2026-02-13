<!--
  BTSettings.svelte
  BT/磁力链接相关设置与高级配置
-->
<script lang="ts">
  import { appSettings, updateAppSettings } from '$lib/stores/settings';
  import { Network, Share2, Database, HelpCircle, ChevronUp, ChevronDown, RefreshCw, Plus, Info } from '@lucide/svelte';
  import { fade } from 'svelte/transition';
  import { invoke } from '@tauri-apps/api/core';
  import { relaunch } from '@tauri-apps/plugin-process';
  import { createLogger } from '$lib/utils/logger';

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
    
    // 自动迁移旧默认值
    if (dhtListenPort === '6881-6999') dhtListenPort = '6881';
    if (listenPort === '6881-6999') listenPort = '6881';
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
          publicTrackers = await invoke<string[]>('fetch_public_trackers');
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

  function handleToggle() {
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

<div class="settings-container">
  
  <div class="settings-section">
    <div class="section-title">基础选项</div>
    <div class="setting-list">
      
      <div class="setting-item">
        <div class="setting-info">
          <div class="setting-header-row">
            <div class="setting-name">
                启用 DHT 网络
                <div class="tooltip-trigger">
                    <Info size={14} />
                    <span class="tooltip-text">允许从 DHT 网络获取节点，有助于无 Tracker 下载。</span>
                </div>
            </div>
            <div class="port-input-inline" class:disabled={!enableDht}>
                <span class="port-label">端口</span>
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
            <div class="inline-restart-hint" transition:fade={{ duration: 150 }}>
                <RefreshCw size={12} />
                <span>端口已改，需重启生效</span>
                <button class="text-relaunch-btn" onclick={() => relaunch()}>立即重启</button>
            </div>
          {/if}
        </div>
        <div class="setting-control">
            <label class="switch">
            <input type="checkbox" bind:checked={enableDht} onchange={handleToggle}>
            <span class="slider"></span>
            </label>
        </div>
      </div>

      <!-- 用户交换 (PEX) -->
      <div class="setting-item">
        <div class="setting-info">
          <div class="setting-header-row">
            <div class="setting-name">
                启用用户交换 (PEX)
                <div class="tooltip-trigger">
                    <Info size={14} />
                    <span class="tooltip-text">允许与连接的节点交换信息。监听端口用于接收连接 (TCP)。</span>
                </div>
            </div>
             <div class="port-input-inline" class:disabled={!enablePeerExchange}>
                <span class="port-label">端口</span>
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
            <div class="inline-restart-hint" transition:fade={{ duration: 150 }}>
                <RefreshCw size={12} />
                <span>端口已改，需重启生效</span>
                <button class="text-relaunch-btn" onclick={() => relaunch()}>立即重启</button>
            </div>
          {/if}
        </div>
        <div class="setting-control">
            <label class="switch">
            <input type="checkbox" bind:checked={enablePeerExchange} onchange={handleToggle}>
            <span class="slider"></span>
            </label>
        </div>
      </div>
    </div>
  </div>

  <div class="settings-section">
    <div class="section-title">做种设置</div>
    <div class="setting-list">
        <!-- 启用做种 -->
        <div class="setting-item">
            <div class="setting-info">
              <div class="setting-name">
                  允许做种 (Seeding)
                  <div class="tooltip-trigger">
                      <Info size={14} />
                      <span class="tooltip-text">关闭后，任务下载完成后将立即停止，不再上传。</span>
                  </div>
              </div>
            </div>
            <div class="setting-control">
                <label class="switch">
                <input type="checkbox" bind:checked={enableSeeding} onchange={handleToggle}>
                <span class="slider"></span>
                </label>
            </div>
        </div>

        {#if enableSeeding}
        <div class="setting-item" transition:fade={{ duration: 150 }}>
            <div class="setting-info">
              <div class="setting-name">
                  分享率限制 (Seed Ratio)
                  <div class="tooltip-trigger">
                      <Info size={14} />
                      <span class="tooltip-text">当分享率达到此值时停止做种 (0 为不限制)。建议设置为 1.0。</span>
                  </div>
              </div>
            </div>
            <div class="input-wrapper ratio-input">
                <input type="number" step="0.1" min="0" bind:value={seedRatio} onchange={handleBlur}>
            </div>
        </div>
        {/if}

        <div class="setting-item">
            <div class="setting-info">
              <div class="setting-name">
                  全局最大上传速度
                  <div class="tooltip-trigger">
                      <Info size={14} />
                      <span class="tooltip-text">限制全局上传速率（单位 MB/s），防止占用过多带宽。0 表示不限制。</span>
                  </div>
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
  </div>

  <div class="settings-section">
    <div class="section-title">高级配置</div>
    <div class="setting-list">
      
      <!-- BT Trackers -->
      <div class="setting-item vertical">
        <div class="setting-info">
          <div class="setting-header-row tracker-header">
            <div class="setting-name">
                BT Trackers
                <div class="tooltip-trigger">
                    <Info size={14} />
                    <span class="tooltip-text">
                        自定义 Tracker 服务器列表，每行一个。更改将在下载新磁力链接或种子时生效。
                    </span>
                </div>
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
        </div>

        {#if showTrackerPreview}
            <div class="tracker-preview-box" transition:fade={{ duration: 150 }}>
                <div class="preview-header">
                    <span>发现 {publicTrackers.length} 个新服务器</span>
                    <div class="preview-btns">
                        <button class="mini-btn append" onclick={appendTrackers}>
                            <Plus size={12} />
                            追加
                        </button>
                        <button class="mini-btn cancel" onclick={() => showTrackerPreview = false}>
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

        <div class="textarea-wrapper">
          <textarea 
            bind:value={btTrackers}
            onblur={handleBlur}
            placeholder="udp://tracker.opentrackr.org:1337/announce..."
            spellcheck="false"
          ></textarea>
        </div>
      </div>

    </div>
  </div>

    <!-- 提示 -->
    <div class="info-box">
        <HelpCircle size={14} />
        <span>提示：修改 BT 配置将会尝试实时应用到当前 Aria2 实例。</span>
    </div>

</div>

<style>
  .textarea-wrapper {
    width: 100%;
    margin-top: 12px;
  }

  textarea {
    width: 100%;
    height: 120px;
    background: var(--surface-hover);
    border: 1px solid var(--border-subtle);
    border-radius: 8px;
    padding: 10px;
    color: var(--text-primary);
    font-size: 12px;
    font-family: monospace;
    resize: vertical;
    outline: none;
    transition: all 0.2s;
    box-sizing: border-box;
  }

  textarea:focus {
    background: var(--input-bg);
    border-color: var(--accent-primary);
    box-shadow: 0 0 0 2px var(--accent-glow);
  }

  .input-wrapper input {
    background: var(--surface-hover);
    border: 1px solid var(--border-subtle);
    border-radius: 6px;
    padding: 6px 10px;
    color: var(--text-primary);
    font-size: 13px;
    width: 80px;
    outline: none;
    transition: all 0.2s;
    text-align: right;
  }

  .input-wrapper input:focus {
    border-color: var(--accent-primary);
    background: var(--input-bg);
    box-shadow: 0 0 0 2px var(--accent-glow);
  }

  .setting-control {
      display: flex;
      align-items: center;
      gap: 12px;
  }

  .setting-header-row {
      display: flex;
      align-items: center;
      gap: 8px;
  }

  .port-input-inline {
      display: flex;
      align-items: center;
      margin-left: 8px;
      gap: 6px;
      background: var(--surface-hover);
      padding: 2px 4px 2px 8px;
      border-radius: 6px;
      border: 1px solid transparent;
      transition: all 0.2s;
  }

  .port-input-inline:hover {
      border-color: var(--border-subtle);
  }

  .port-input-inline:focus-within {
      background: var(--input-bg);
      border-color: var(--accent-primary);
      box-shadow: 0 0 0 2px var(--accent-glow);
  }

  .port-input-inline.disabled {
      opacity: 0.5;
      pointer-events: none;
      filter: grayscale(1);
  }

  .port-label {
      font-size: 11px;
      color: var(--text-muted);
      font-weight: 500;
      white-space: nowrap;
  }

  .port-input-inline input {
      background: transparent;
      border: none;
      padding: 0;
      color: var(--text-primary);
      font-size: 13px;
      width: 54px;
      outline: none;
      font-family: monospace;
      height: 20px;
      text-align: center;
  }

  .port-input-inline input:hover {
      background: transparent;
      border-color: transparent;
  }

  .port-input-inline input:focus {
      background: transparent;
      border-color: transparent;
      width: 64px;
  }

  .port-controls {
      display: flex;
      flex-direction: column;
      gap: 1px;
      margin-left: 2px;
  }

  .port-btn {
      display: flex;
      align-items: center;
      justify-content: center;
      width: 14px;
      height: 10px;
      padding: 0;
      border: none;
      background: rgba(255, 255, 255, 0.05);
      color: var(--text-muted);
      cursor: pointer;
      border-radius: 2px;
      transition: all 0.1s;
  }

  .port-btn:hover {
      background: var(--surface-hover);
      color: var(--text-primary);
  }

  .port-btn:active {
      transform: translateY(1px);
  }

  /* 复用 DownloadSettings 的部分样式逻辑 */
  .input-wrapper.group {
      display: flex;
      align-items: center;
  }

  .input-wrapper.group input {
      border-top-right-radius: 0;
      border-bottom-right-radius: 0;
      width: 60px;
  }

  .unit {
    background: var(--surface-active);
    border: 1px solid var(--border-subtle);
    border-left: none;
    padding: 6px 8px; /* Match input padding height approx */
    font-size: 11px;
    color: var(--text-muted);
    border-top-right-radius: 6px;
    border-bottom-right-radius: 6px;
    height: 29px; /* Align with input height */
    box-sizing: border-box;
    display: flex;
    align-items: center;
  }

  .info-box {
      display: flex;
      align-items: center;
      gap: 8px;
      padding: 12px 14px;
      background: rgba(59, 130, 246, 0.08); /* Blue tint */
      border: 1px solid rgba(59, 130, 246, 0.15);
      border-radius: 10px;
      color: var(--text-secondary);
      font-size: 12px;
      margin-top: 16px;
      line-height: 1.4;
  }

  .inline-restart-hint {
      display: flex;
      align-items: center;
      gap: 6px;
      margin-top: 8px;
      padding: 6px 10px;
      background: rgba(245, 158, 11, 0.1);
      border: 1px solid rgba(245, 158, 11, 0.2);
      border-radius: 6px;
      color: #f59e0b;
      font-size: 11px;
      width: fit-content;
  }

  .text-relaunch-btn {
      background: #f59e0b;
      color: #000;
      border: none;
      padding: 2px 8px;
      border-radius: 4px;
      font-size: 10px;
      font-weight: 600;
      cursor: pointer;
      margin-left: 4px;
      transition: all 0.2s;
  }

  .text-relaunch-btn:hover {
      background: #d97706;
      transform: scale(1.02);
  }

  /* Tooltip Styles */
  .tooltip-trigger {
      position: relative;
      display: inline-flex;
      align-items: center;
      color: var(--text-muted);
      cursor: help;
      margin-left: 4px;
      transition: color 0.2s;
  }

  .tooltip-trigger:hover {
      color: var(--accent-primary);
  }

  .tooltip-text {
      visibility: hidden;
      width: 200px;
      background-color: var(--surface-active);
      color: var(--text-primary);
      text-align: left;
      border-radius: 6px;
      padding: 8px 10px;
      position: absolute;
      z-index: 10;
      bottom: 125%;
      left: 50%;
      margin-left: -100px;
      opacity: 0;
      transition: opacity 0.2s;
      font-size: 11px;
      font-weight: normal;
      line-height: 1.4;
      border: 1px solid var(--border-subtle);
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
      pointer-events: none;
  }

  .tooltip-trigger:hover .tooltip-text {
      visibility: visible;
      opacity: 1;
  }

  /* Tracker Enhancement Styles */
  .tracker-header {
      justify-content: space-between;
      width: 100%;
  }

  .action-btn {
      display: flex;
      align-items: center;
      gap: 6px;
      padding: 4px 10px;
      background: var(--surface-hover);
      border: 1px solid var(--border-subtle);
      border-radius: 6px;
      color: var(--text-secondary);
      font-size: 11px;
      cursor: pointer;
      transition: all 0.2s;
  }

  .action-btn:hover:not(:disabled) {
      background: var(--surface-active);
      color: var(--text-primary);
      border-color: var(--accent-primary);
  }

  .action-btn:disabled {
      opacity: 0.6;
      cursor: not-allowed;
  }

  .tracker-preview-box {
      margin-top: 12px;
      background: var(--surface-active);
      border: 1px solid var(--accent-dim);
      border-radius: 8px;
      padding: 10px;
      font-size: 12px;
  }

  .preview-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 8px;
      font-weight: 500;
      color: var(--text-primary);
  }

  .preview-btns {
      display: flex;
      gap: 6px;
  }

  .mini-btn {
      display: flex;
      align-items: center;
      gap: 4px;
      padding: 2px 8px;
      border-radius: 4px;
      font-size: 10px;
      border: 1px solid transparent;
      cursor: pointer;
      transition: all 0.2s;
  }

  .mini-btn.append {
      background: var(--accent-primary);
      color: white;
  }

  .mini-btn.append:hover {
      background: var(--accent-hover);
  }

  .mini-btn.cancel {
      background: var(--surface-hover);
      border-color: var(--border-subtle);
      color: var(--text-secondary);
  }

  .mini-btn.cancel:hover {
      background: var(--surface-active);
      color: var(--text-primary);
  }

  .preview-content {
      font-family: monospace;
      font-size: 11px;
      color: var(--text-muted);
      white-space: pre-wrap;
      max-height: 80px;
      overflow-y: auto;
      padding: 6px;
      background: rgba(0, 0, 0, 0.1);
      border-radius: 4px;
  }

  @keyframes spin {
      from { transform: rotate(0deg); }
      to { transform: rotate(360deg); }
  }
</style>
