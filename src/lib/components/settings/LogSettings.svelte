<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Eraser } from '@lucide/svelte';
  import { createLogger } from '$lib/utils/logger';
  import { startAria2LogStream, stopAria2LogStream, subscribeAria2Stdout } from '$lib/services/aria2';

  const logger = createLogger('LogSettings');

  let enabled = $state(false);
  let logs: string[] = $state([]);
  let unlisten: (() => void) | null = null;
  let transitionInFlight: Promise<void> | null = null;
  let scrollContainer: HTMLDivElement;

  async function startStream() {
    if (unlisten) return;

    try {
        await startAria2LogStream();
        unlisten = await subscribeAria2Stdout((line) => {
            logs = [...logs, line];
            // 限制日志条数，防止内存泄漏
            if (logs.length > 200) {
                logs = logs.slice(logs.length - 200);
            }
            scrollToBottom();
        });
    } catch (e) {
        logger.error('Failed to start aria2 log stream', { error: e });
    }
  }

  async function stopStream() {
    try {
        if (unlisten) {
            unlisten();
            unlisten = null;
        }
        await stopAria2LogStream();
    } catch (e) {
        logger.error('Failed to stop aria2 log stream', { error: e });
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
      if (transitionInFlight) return;

      enabled = !enabled;
      transitionInFlight = (enabled ? startStream() : stopStream())
        .catch((e) => {
          logger.error('Log stream toggle failed', { error: e });
          enabled = !enabled;
        })
        .finally(() => {
          transitionInFlight = null;
        });
  }
  
  function clearLogs() {
      logs = [];
  }

  onMount(() => {
      // 默认关闭，不自动开启
  });

  onDestroy(() => {
      // 组件销毁时强制关闭流
      void stopStream();
  });
</script>

<div class="settings-section">
  <div class="header-row">
      <div class="title-group">
          <h4 class="section-title">实时日志</h4>
          <span class="badge ui-badge warning">Performance Cost</span>
      </div>
      <div class="controls">
          <button class="icon-btn ui-btn-icon ui-btn-focus ui-disabled" onclick={clearLogs} title="清空日志">
              <Eraser size={14} />
          </button>
          <label class="switch">
            <input 
              type="checkbox" 
              checked={enabled}
              onchange={handleToggle}
            />
            <span class="slider"></span>
          </label>
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
        font-size: 11px;
        color: var(--semantic-warning);
    }

    .settings-section {
        display: flex;
        flex-direction: column;
        height: 100%;
    }

    .description {
        font-size: 12px;
        color: color-mix(in srgb, var(--text-secondary) 92%, transparent);
        margin-bottom: 16px;
        line-height: 1.45;
    }

    .controls {
        display: flex;
        align-items: center;
        gap: 12px;
    }

    .icon-btn {
        width: 32px;
        height: 32px;
    }

    .terminal-window {
        flex: 1;
        background: color-mix(in srgb, var(--dialog-bg) 82%, #000000);
        border-radius: 8px;
        border: none;
        padding: 12px;
        font-family: 'JetBrains Mono', 'Fira Code', ui-monospace, SFMono-Regular, monospace;
        font-size: 11px;
        line-height: 1.4;
        overflow: hidden;
        display: flex;
        flex-direction: column;
        min-height: 200px;
        box-shadow:
            inset 0 2px 4px rgba(0,0,0,0.2),
            inset 0 0 0 1px color-mix(in srgb, var(--accent-primary) 10%, transparent);
    }

    .terminal-logs {
        flex: 1;
        overflow: auto;
        color: color-mix(in srgb, var(--text-primary) 86%, #ffffff);
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
        color: var(--text-muted);
    }

    .blink {
        animation: blink 1.5s infinite;
    }

    .muted {
        opacity: 0.62;
    }

    @keyframes blink {
        0% { opacity: 0.3; }
        50% { opacity: 1; }
        100% { opacity: 0.3; }
    }
</style>
