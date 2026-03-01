<script lang="ts">
	import { onDestroy } from 'svelte';
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

	onDestroy(() => {
		// 组件销毁时强制关闭流
		void stopStream();
	});
</script>

<div class="settings-section log-settings">
  <div class="header-row">
    <div class="title-group">
      <div class="title-line">
        <h4 class="section-title">实时日志</h4>
        <span class="badge ui-badge warning">Performance Cost</span>
      </div>
      <div class="description">开启后可查看 Aria2 核心的实时输出。此功能会消耗额外性能，关闭面板时将自动停止。</div>
    </div>

    <div class="controls">
      <button class="icon-btn ui-btn-icon ui-btn-focus ui-disabled" onclick={clearLogs} title="清空日志" aria-label="清空日志">
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

  <div class="stream-note ui-status-card info" role="status" aria-live="polite">
    <span class="status-dot" class:live={enabled}></span>
    <span>{enabled ? '日志流已开启，自动滚动到最新输出' : '日志流未开启，开启后将实时显示 Aria2 输出'}</span>
  </div>

  <div class="terminal-window">
    <div class="terminal-toolbar">
      <span class="terminal-title">Aria2 STDOUT</span>
      <span class="terminal-state" class:active={enabled}>{enabled ? 'Streaming' : 'Idle'}</span>
    </div>

    <div class="terminal-logs" bind:this={scrollContainer}>
      {#if logs.length === 0}
        <div class="empty-state">
          {#if enabled}
            <span class="empty-title blink">Waiting for logs...</span>
            <span class="empty-desc">核心已启动日志监听，输出将实时显示在这里</span>
          {:else}
            <span class="empty-title muted">日志已关闭</span>
            <span class="empty-desc">打开右上角开关以查看 Aria2 实时输出</span>
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
  .log-settings {
    display: flex;
    flex-direction: column;
    height: 100%;
    gap: 12px;
  }

  .header-row {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
  }

  .title-group {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .title-line {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .badge {
    font-size: 11px;
    color: var(--semantic-warning);
  }

  .description {
    font-size: 12px;
    line-height: 1.5;
    color: color-mix(in srgb, var(--text-secondary) 95%, transparent);
    max-width: 72ch;
  }

  .controls {
    display: flex;
    align-items: center;
    gap: 10px;
    align-self: center;
    padding: 4px 8px;
    border-radius: 10px;
    background: color-mix(in srgb, var(--glass-elevated-bg, var(--control-bg)) 78%, transparent);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--glass-border) 34%, transparent);
  }

  .icon-btn {
    width: 30px;
    height: 30px;
  }

  .stream-note {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    width: fit-content;
    max-width: 100%;
    font-size: 11px;
    color: var(--text-secondary);
  }

  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 999px;
    background: color-mix(in srgb, var(--text-muted) 72%, transparent);
    flex-shrink: 0;
    transition: background-color 0.2s ease, box-shadow 0.2s ease;
  }

  .status-dot.live {
    background: var(--semantic-success);
    box-shadow: 0 0 0 4px color-mix(in srgb, var(--semantic-success) 22%, transparent);
  }

  .terminal-window {
    flex: 1;
    min-height: 220px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border-radius: 10px;
    border: none;
    background:
      linear-gradient(
        180deg,
        color-mix(in srgb, var(--dialog-bg) 74%, #04070d),
        color-mix(in srgb, var(--dialog-bg) 84%, #000)
      );
    box-shadow:
      inset 0 0 0 1px color-mix(in srgb, var(--accent-primary) 10%, transparent),
      inset 0 8px 18px color-mix(in srgb, #000 22%, transparent);
  }

  .terminal-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    border-bottom: 1px solid color-mix(in srgb, var(--glass-border) 28%, transparent);
    background: color-mix(in srgb, var(--dialog-bg) 84%, transparent);
  }

  .terminal-title {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    font-weight: 600;
    color: color-mix(in srgb, var(--text-muted) 88%, transparent);
  }

  .terminal-state {
    padding: 2px 7px;
    border-radius: 999px;
    font-size: 10px;
    font-weight: 600;
    color: color-mix(in srgb, var(--text-muted) 92%, transparent);
    background: color-mix(in srgb, var(--control-bg) 72%, transparent);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--border-subtle) 34%, transparent);
  }

  .terminal-state.active {
    color: var(--semantic-success);
    background: color-mix(in srgb, var(--semantic-success) 16%, transparent);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--semantic-success) 26%, transparent);
  }

  .terminal-logs {
    flex: 1;
    overflow: auto;
    white-space: pre;
    font-family: var(--font-mono);
    font-size: 11px;
    line-height: 1.48;
    padding: 12px;
    color: color-mix(in srgb, var(--text-primary) 88%, #ffffff);
  }

  .log-line {
    margin-bottom: 3px;
    padding: 0 4px;
    border-radius: 6px;
  }

  .log-line:hover {
    background: color-mix(in srgb, var(--accent-primary) 12%, transparent);
  }

  .empty-state {
    min-height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    text-align: center;
    padding: 18px;
    color: var(--text-muted);
  }

  .empty-title {
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.02em;
  }

  .empty-desc {
    font-size: 11px;
    line-height: 1.45;
    color: color-mix(in srgb, var(--text-muted) 88%, transparent);
    max-width: 42ch;
  }

  .blink {
    animation: blink 1.5s infinite;
  }

  .muted {
    opacity: 0.62;
  }

  @keyframes blink {
    0% {
      opacity: 0.3;
    }

    50% {
      opacity: 1;
    }

    100% {
      opacity: 0.3;
    }
  }

  @media (max-width: 720px) {
    .header-row {
      flex-direction: column;
      align-items: stretch;
      gap: 10px;
    }

    .controls {
      align-self: flex-start;
    }

    .stream-note {
      width: 100%;
    }
  }
</style>
