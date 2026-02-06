<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/core';
  import { Terminal, Eraser } from '@lucide/svelte';

  let enabled = $state(false);
  let logs: string[] = $state([]);
  let unlisten: (() => void) | null = null;
  let scrollContainer: HTMLDivElement;

  async function startStream() {
    try {
        await invoke('start_log_stream');
        unlisten = await listen<string>('aria2-stdout', (event) => {
            logs = [...logs, event.payload];
            // 限制日志条数，防止内存泄漏
            if (logs.length > 200) {
                logs = logs.slice(logs.length - 200);
            }
            scrollToBottom();
        });
    } catch (e) {
        console.error('Failed to start log stream', e);
    }
  }

  async function stopStream() {
    try {
        if (unlisten) {
            unlisten();
            unlisten = null;
        }
        await invoke('stop_log_stream');
    } catch (e) {
        console.error('Failed to stop log stream', e);
    }
  }

  function scrollToBottom() {
      if (scrollContainer) {
          requestAnimationFrame(() => {
              scrollContainer.scrollTop = scrollContainer.scrollHeight;
          });
      }
  }

  function handleToggle() {
      enabled = !enabled;
      if (enabled) {
          startStream();
      } else {
          stopStream();
      }
  }
  
  function clearLogs() {
      logs = [];
  }

  onMount(() => {
      // 默认关闭，不自动开启
  });

  onDestroy(() => {
      // 组件销毁时强制关闭流
      stopStream();
  });
</script>

<div class="settings-section">
  <div class="header-row">
      <div class="title-group">
          <h4 class="section-title">实时日志</h4>
          <span class="badge">Performance Cost</span>
      </div>
      <div class="controls">
          <button class="icon-btn" onclick={clearLogs} title="清空日志">
              <Eraser size={14} />
          </button>
          <button 
            class="toggle-switch" 
            class:checked={enabled} 
            onclick={handleToggle}
            role="switch" 
            aria-checked={enabled}
            aria-label="Toggle Log Stream"
          >
            <div class="toggle-track"></div>
            <div class="toggle-thumb"></div>
          </button>
      </div>
  </div>

  <div class="description">
    开启后可查看 Aria2 核心的实时输出。此功能会消耗额外性能，关闭面板时将自动停止。
  </div>

  <div class="terminal-window">
      <div class="terminal-logs" bind:this={scrollContainer}>
          {#if logs.length === 0}
              <div class="empty-state">
                  {#if enabled}
                      <span class="blink">Waiting for logs...</span>
                  {:else}
                      <span class="muted">日志已关闭</span>
                  {/if}
              </div>
          {:else}
              {#each logs as log}
                  <div class="log-line">{log}</div>
              {/each}
              <div class="anchor"></div>
          {/if}
      </div>
  </div>
</div>

<style>
    .header-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-bottom: 8px;
    }

    .title-group {
        display: flex;
        align-items: baseline;
        gap: 8px;
    }
    
    .badge {
        font-size: 10px;
        color: var(--warning-color, #f59e0b);
        background: rgba(245, 158, 11, 0.1);
        padding: 2px 6px;
        border-radius: 4px;
        border: 1px solid rgba(245, 158, 11, 0.2);
    }

    .settings-section {
        display: flex;
        flex-direction: column;
        height: 100%;
    }

    .description {
        font-size: 12px;
        color: var(--text-secondary);
        margin-bottom: 16px;
    }

    .controls {
        display: flex;
        align-items: center;
        gap: 12px;
    }

    .icon-btn {
        background: transparent;
        border: none;
        color: var(--text-muted);
        cursor: pointer;
        padding: 4px;
        border-radius: 4px;
        transition: all 0.2s;
        display: flex;
    }

    .icon-btn:hover {
        background: var(--surface-hover);
        color: var(--text-primary);
    }

    .terminal-window {
        flex: 1;
        background: #1e1e1e; /* Fixed dark bg for terminal */
        border-radius: 8px;
        border: 1px solid var(--border-color);
        padding: 12px;
        font-family: 'JetBrains Mono', 'Fira Code', ui-monospace, SFMono-Regular, monospace;
        font-size: 11px;
        line-height: 1.4;
        overflow: hidden;
        display: flex;
        flex-direction: column;
        min-height: 200px;
        box-shadow: inset 0 2px 4px rgba(0,0,0,0.2);
    }

    .terminal-logs {
        flex: 1;
        overflow: auto;
        color: #d4d4d4;
        white-space: pre;
    }
    
    .log-line {
        margin-bottom: 2px;
    }

    .empty-state {
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        color: #666;
    }

    .blink {
        animation: blink 1.5s infinite;
    }

    .muted {
        opacity: 0.5;
    }

    @keyframes blink {
        0% { opacity: 0.3; }
        50% { opacity: 1; }
        100% { opacity: 0.3; }
    }

    /* Switch Style (Simplified) */
    .toggle-switch {
        position: relative;
        width: 36px;
        height: 20px;
        background: var(--input-bg);
        border-radius: 20px;
        border: 1px solid var(--border-color);
        cursor: pointer;
        padding: 0;
        transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    }

    .toggle-thumb {
        position: absolute;
        top: 2px;
        left: 2px;
        width: 14px;
        height: 14px;
        background: var(--text-muted);
        border-radius: 50%;
        transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
        box-shadow: 0 1px 2px rgba(0,0,0,0.1);
    }

    .toggle-switch.checked {
        background: var(--accent-primary);
        border-color: var(--accent-primary);
    }

    .toggle-switch.checked .toggle-thumb {
        transform: translateX(16px);
        background: white;
    }
</style>
