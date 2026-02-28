<script lang="ts">
  import { page } from '$app/stores';
  import { dev } from '$app/environment';
  import { recoverAppFromRuntimeFailure } from '$lib/services/boot';

  let recovering = $state(false);

  async function handleRetry() {
    recovering = true;
    try {
      await recoverAppFromRuntimeFailure();
      recovering = false;
      return;
    } catch {
      recovering = false;
    }
  }

  function getErrorStack(error: unknown): string | null {
    if (!error || typeof error !== 'object') {
      return null;
    }

    const maybeStack = (error as { stack?: unknown }).stack;
    return typeof maybeStack === 'string' ? maybeStack : null;
  }
</script>

<div class="error-page">
  <h1>{$page.status}: {$page.error?.message}</h1>
  {#if dev && getErrorStack($page.error)}
    <pre>{getErrorStack($page.error)}</pre>
  {/if}
  <p>Path: {$page.url.pathname}</p>
  <button onclick={handleRetry} disabled={recovering}>{recovering ? 'Recovering...' : 'Retry'}</button>
</div>

<style>
  .error-page {
    padding: 2rem;
    color: var(--text-primary, #333);
    background: var(--bg-base, #fff);
    height: 100vh;
    overflow: auto;
  }
  pre {
    background: #f1f1f1;
    padding: 1rem;
    overflow-x: auto;
  }
</style>
