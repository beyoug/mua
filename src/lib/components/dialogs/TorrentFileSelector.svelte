<script lang="ts">
  import { slide } from 'svelte/transition';
  import { Check, File, Folder } from '@lucide/svelte';
  import type { TorrentInfo, TorrentFile } from '$lib/api/cmd';
  import { formatBytes } from '$lib/utils/format';

  let { torrentInfo, onSelectionChange } = $props<{
    torrentInfo: TorrentInfo;
    onSelectionChange: (selectedFiles: string | undefined) => void;
  }>();

  let selectedIndices = $state<Set<number>>(new Set());

  // Initialize with all files selected by default
  $effect(() => {
    if (torrentInfo) {
      const all = new Set(torrentInfo.files.map((f) => f.index));
      selectedIndices = all;
      notifyChange();
    }
  });

  function toggleFile(index: number) {
    if (selectedIndices.has(index)) {
      selectedIndices.delete(index);
    } else {
      selectedIndices.add(index);
    }
    // trigget reactivity
    selectedIndices = new Set(selectedIndices);
    notifyChange();
  }

  function toggleAll() {
    if (selectedIndices.size === torrentInfo.files.length) {
      selectedIndices = new Set();
    } else {
      selectedIndices = new Set(torrentInfo.files.map((f) => f.index));
    }
    notifyChange();
  }

  function notifyChange() {
    if (selectedIndices.size === 0 || selectedIndices.size === torrentInfo.files.length) {
      // If none or all selected, undefined means "download all" (default behavior) or handled by logic
      // Usually "select-file" option in aria2: 
      // "Specify file to download by index... multiple indexes can be specified... 1-4,7"
      // If all selected, we can omit the option or pass all indexes.
      // If NONE selected, that's weird, maybe don't download anything?
        if (selectedIndices.size === 0) {
            onSelectionChange(""); // invalid/none
        } else {
            onSelectionChange(undefined); // all (default)
        }
    } else {
      // Convert Set to sorted array and then to string ranges
      const sorted = Array.from(selectedIndices).sort((a, b) => a - b);
      // Aria2 indices are 1-based!!!
      // Wait, let's allow aria2 1-based check.
      // aria2c man page: "index... 1, 2-5".
      // My TorrentInfo.files index usually comes from 0-based iteration in Rust.
      // So I need to add 1 to each index.
      
      const oneBased = sorted.map(i => i + 1);
      onSelectionChange(oneBased.join(','));
    }
  }

  function isSelected(index: number) {
    return selectedIndices.has(index);
  }
</script>

<div class="torrent-selector">
  <div class="header">
    <div class="info">
      <h3>{torrentInfo.name}</h3>
      <span class="meta">
        {torrentInfo.files.length} 文件 · {formatBytes(torrentInfo.total_length)}
      </span>
    </div>
    <button class="btn-text" onclick={toggleAll}>
      {selectedIndices.size === torrentInfo.files.length ? '取消全选' : '全选'}
    </button>
  </div>

  <div class="file-list">
    {#each torrentInfo.files as file}
      <button
        class="file-item"
        class:selected={isSelected(file.index)}
        onclick={() => toggleFile(file.index)}
      >
        <div class="checkbox">
          {#if isSelected(file.index)}
            <Check size={12} color="var(--primary-color)" />
          {/if}
        </div>
        <div class="file-info">
            <span class="name" title={file.path}>{file.path}</span>
            <span class="size">{formatBytes(file.length)}</span>
        </div>
      </button>
    {/each}
  </div>
</div>

<style>
  .torrent-selector {
    background: var(--bg-secondary);
    border-radius: 8px;
    border: 1px solid var(--border-color);
    margin-top: 12px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    max-height: 200px;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    background: var(--bg-tertiary);
    border-bottom: 1px solid var(--border-color);
  }

  .info h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 500;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 200px;
  }

  .meta {
    font-size: 11px;
    color: var(--text-secondary);
  }

  .btn-text {
    background: none;
    border: none;
    color: var(--primary-color);
    font-size: 12px;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 4px;
  }
  
  .btn-text:hover {
      background: var(--bg-hover);
  }

  .file-list {
    overflow-y: auto;
    flex: 1;
    padding: 8px 0;
  }

  .file-item {
    display: flex;
    align-items: center;
    width: 100%;
    padding: 6px 16px;
    background: none;
    border: none;
    text-align: left;
    cursor: pointer;
    gap: 12px;
    transition: background-color 0.1s;
  }

  .file-item:hover {
    background: var(--bg-hover);
  }

  .checkbox {
    width: 16px;
    height: 16px;
    border-radius: 4px;
    border: 1px solid var(--border-color);
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-primary);
  }

  .file-item.selected .checkbox {
    border-color: var(--primary-color);
    background: var(--primary-color-dim);
  }

  .file-info {
    flex: 1;
    display: flex;
    justify-content: space-between;
    min-width: 0;
    font-size: 12px;
  }

  .name {
      color: var(--text-primary);
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      margin-right: 12px;
  }

  .size {
      color: var(--text-secondary);
      flex-shrink: 0;
  }
</style>
