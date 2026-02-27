<script lang="ts">
  import { page } from '$app/stores';
  import { dev } from '$app/environment';

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
  <button onclick={() => window.location.reload()}>Retry</button>
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
