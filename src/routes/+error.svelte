<script lang="ts">
  import { page } from '$app/stores';
</script>

<div class="error-page">
  <h1>{$page.status}: {$page.error?.message}</h1>
  {#if ($page.error as any)?.stack}
    <pre>{($page.error as any).stack}</pre>
  {/if}
  <p>Path: {$page.url.pathname}</p>
  <button onclick={() => window.location.reload()}>Retry</button>
</div>

<style>
  .error-page {
    padding: 2rem;
    color: var(--text-primary, #333);
    background: var(--bg-primary, #fff);
    height: 100vh;
    overflow: auto;
  }
  pre {
    background: #f1f1f1;
    padding: 1rem;
    overflow-x: auto;
  }
</style>
