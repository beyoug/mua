<!--
  TorrentConfigDialog.svelte
  种子任务配置弹窗 - 拖拽/选择 torrent 后弹出
-->
<script lang="ts">
    import { Magnet, FolderOpen, Download, Network, RefreshCw, Plus, Import, Loader2 } from '@lucide/svelte';
    import { open as openDialog } from '@tauri-apps/plugin-dialog';
    import { invoke } from '@tauri-apps/api/core';
    import { fade } from 'svelte/transition';
    import type { TorrentInfo } from '$lib/api/cmd';
    import { formatBytes } from '$lib';
    import { appSettings, saveAppSettings } from '$lib/stores/settings';
    import BaseModal from '../common/BaseModal.svelte';
    import TorrentFileSelector from './TorrentFileSelector.svelte';

    export interface TorrentDialogResult {
        torrentPath: string;
        selectedFiles?: string;
        trackers: string;
        savePath: string;
    }

    interface Props {
        open: boolean;
        torrentInfo: TorrentInfo | null;
        torrentPath: string;
        parseError?: string;
        onConfirm: (result: TorrentDialogResult) => void;
        onCancel: () => void;
    }

    let { open, torrentInfo, torrentPath, parseError = '', onConfirm, onCancel }: Props = $props();

    const isLoading = $derived(!torrentInfo && !parseError);

    let savePath = $state($appSettings.defaultSavePath || '~/Downloads');
    let trackers = $state('');
    let selectedFiles = $state<string | undefined>(undefined);

    // Tracker 获取
    let isFetchingTrackers = $state(false);
    let publicTrackers = $state<string[]>([]);
    let showTrackerPreview = $state(false);

    // 打开时从全局设置同步 tracker
    $effect(() => {
        if (open) {
            savePath = $appSettings.defaultSavePath || '~/Downloads';
            trackers = $appSettings.btTrackers || '';
            selectedFiles = undefined;
            publicTrackers = [];
            showTrackerPreview = false;
        }
    });

    async function selectFolder() {
        try {
            const selected = await openDialog({
                directory: true,
                defaultPath: savePath,
                title: '选择保存位置'
            });
            if (selected) savePath = selected as string;
        } catch (_) {}
    }

    async function fetchTrackers() {
        if (isFetchingTrackers) return;
        isFetchingTrackers = true;
        try {
            publicTrackers = await invoke<string[]>('fetch_public_trackers');
            showTrackerPreview = true;
        } catch (e) {
            console.error('Failed to fetch trackers:', e);
        } finally {
            isFetchingTrackers = false;
        }
    }

    function appendTrackers() {
        if (publicTrackers.length === 0) return;
        const newTrackers = publicTrackers.join('\n');
        if (trackers.trim()) {
            trackers = trackers.trim() + '\n' + newTrackers;
        } else {
            trackers = newTrackers;
        }
        showTrackerPreview = false;
    }

    function importGlobalTrackers() {
        const global = $appSettings.btTrackers || '';
        if (!global.trim()) return;
        if (trackers.trim()) {
            // 去重合并
            const existing = new Set(trackers.split('\n').map(l => l.trim()).filter(l => l));
            const incoming = global.split('\n').map(l => l.trim()).filter(l => l);
            const newOnes = incoming.filter(t => !existing.has(t));
            if (newOnes.length > 0) {
                trackers = trackers.trim() + '\n' + newOnes.join('\n');
            }
        } else {
            trackers = global;
        }
    }

    function handleConfirm() {
        console.log('[TorrentConfig] handleConfirm clicked, torrentPath:', torrentPath);

        // Tracker 保存不阻塞提交（fire-and-forget）
        if (trackers.trim()) {
            saveAppSettings({ ...$appSettings, btTrackers: trackers })
                .catch(e => console.error('Failed to save trackers:', e));
        }

        onConfirm({
            torrentPath,
            selectedFiles,
            trackers,
            savePath
        });
    }
</script>

<BaseModal
    {open}
    onClose={onCancel}
    size="md"
    minHeight="400px"
    closeOnClickOutside={false}
    closeOnEscape={true}
>
    {#snippet header()}
        <div class="dialog-title">
            <Magnet size={16} />
            <span>种子任务配置</span>
        </div>
    {/snippet}

    <div class="config-body">
        {#if parseError}
            <div class="parse-warning">
                <span>⚠️ 解析失败，但仍可提交任务（由 aria2 处理）</span>
            </div>
        {/if}

        <!-- 种子信息 -->
        {#if isLoading}
            <div class="torrent-info-bar loading-bar">
                <div class="info-item" style="flex:1;overflow:hidden">
                    <span class="info-label">文件</span>
                    <span class="info-value" title={torrentPath}>{torrentPath.split('/').pop() || torrentPath}</span>
                </div>
                <div class="loading-content">
                    <Loader2 size={16} class="spin" />
                    <span>解析中</span>
                </div>
            </div>
        {:else if torrentInfo}
            <div class="torrent-info-bar">
                <div class="info-item">
                    <span class="info-label">名称</span>
                    <span class="info-value" title={torrentInfo.name}>{torrentInfo.name}</span>
                </div>
                <div class="info-item">
                    <span class="info-label">大小</span>
                    <span class="info-value">{formatBytes(torrentInfo.total_length)}</span>
                </div>
                <div class="info-item">
                    <span class="info-label">文件数</span>
                    <span class="info-value">{torrentInfo.files.length}</span>
                </div>
            </div>

            <!-- 文件选择 -->
            {#if torrentInfo.files.length > 1}
                <div class="section">
                    <div class="section-header">文件选择</div>
                    <TorrentFileSelector 
                        {torrentInfo}
                        onSelectionChange={(s) => selectedFiles = s} 
                    />
                </div>
            {/if}
        {/if}

        <!-- 保存位置 -->
        <div class="section">
            <div class="section-header">
                <FolderOpen size={13} />
                <span>保存位置</span>
            </div>
            <button class="path-selector" onclick={selectFolder}>
                <span class="path-text">{savePath}</span>
                <FolderOpen size={14} />
            </button>
        </div>

        <!-- Trackers -->
        <div class="section">
            <div class="section-header">
                <Network size={13} />
                <span>Trackers</span>
                <div class="tracker-btns">
                    <button class="mini-action" onclick={importGlobalTrackers} title="从全局设置导入">
                        <Import size={12} />
                        <span>导入全局</span>
                    </button>
                    <button class="mini-action" onclick={fetchTrackers} disabled={isFetchingTrackers}>
                        {#if isFetchingTrackers}
                            <RefreshCw size={12} class="spin" />
                        {:else}
                            <RefreshCw size={12} />
                        {/if}
                        <span>获取公共</span>
                    </button>
                </div>
            </div>

            {#if showTrackerPreview}
                <div class="tracker-preview" transition:fade={{ duration: 150 }}>
                    <div class="preview-header">
                        <span>发现 {publicTrackers.length} 个 Tracker</span>
                        <div class="preview-btns">
                            <button class="mini-btn primary" onclick={appendTrackers}>
                                <Plus size={12} />
                                追加
                            </button>
                            <button class="mini-btn" onclick={() => showTrackerPreview = false}>
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

            <textarea
                class="tracker-textarea"
                bind:value={trackers}
                placeholder="每行一个 Tracker URL&#10;udp://tracker.opentrackr.org:1337/announce"
                spellcheck="false"
                rows="4"
            ></textarea>
        </div>
    </div>

    {#snippet footer()}
        <div class="footer-layout">
            <button class="btn-cancel" onclick={onCancel}>取消</button>
            <button class="btn-confirm" onclick={handleConfirm}>
                <Download size={14} />
                <span>开始下载</span>
            </button>
        </div>
    {/snippet}
</BaseModal>

<style>
    .dialog-title {
        display: flex;
        align-items: center;
        gap: 8px;
        font-size: 15px;
        font-weight: 600;
        color: var(--text-primary);
    }

    .config-body {
        padding: 20px 24px;
        display: flex;
        flex-direction: column;
        gap: 16px;
    }

    /* 种子信息条 */
    .torrent-info-bar {
        display: flex;
        gap: 16px;
        padding: 12px 14px;
        background: var(--surface-hover);
        border: 1px solid var(--border-subtle);
        border-radius: 10px;
    }

    .torrent-info-bar.loading-bar {
        justify-content: space-between;
        align-items: center;
        gap: 12px;
    }

    .loading-content {
        display: flex;
        align-items: center;
        gap: 10px;
        color: var(--text-secondary);
        font-size: 13px;
    }

    .parse-warning {
        padding: 10px 14px;
        background: color-mix(in srgb, var(--warning-color, #f59e0b) 10%, transparent);
        border: 1px solid color-mix(in srgb, var(--warning-color, #f59e0b) 30%, transparent);
        border-radius: 10px;
        color: var(--warning-color, #f59e0b);
        font-size: 12px;
    }

    .info-item {
        display: flex;
        flex-direction: column;
        gap: 2px;
        min-width: 0;
    }

    .info-item:first-child {
        flex: 1;
        overflow: hidden;
    }

    .info-label {
        font-size: 11px;
        color: var(--text-tertiary);
        font-weight: 500;
    }

    .info-value {
        font-size: 13px;
        color: var(--text-primary);
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    /* 区块 */
    .section {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .section-header {
        display: flex;
        align-items: center;
        gap: 6px;
        font-size: 13px;
        color: var(--text-secondary);
        font-weight: 500;
    }

    .tracker-btns {
        margin-left: auto;
        display: flex;
        gap: 6px;
    }

    .mini-action {
        display: flex;
        align-items: center;
        gap: 4px;
        padding: 3px 8px;
        background: var(--surface-hover);
        border: 1px solid var(--border-subtle);
        border-radius: 6px;
        color: var(--text-secondary);
        font-size: 11px;
        cursor: pointer;
        transition: all 0.2s;
    }

    .mini-action:hover:not(:disabled) {
        background: var(--surface-active);
        color: var(--text-primary);
        border-color: var(--accent-primary);
    }

    .mini-action:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    /* 路径选择器 */
    .path-selector {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 10px 14px;
        background: var(--input-bg, rgba(255, 255, 255, 0.05));
        border: 1px solid var(--border-color, rgba(255, 255, 255, 0.1));
        border-radius: 10px;
        color: var(--text-primary);
        font-size: 13px;
        cursor: pointer;
        transition: all 0.2s;
        text-align: left;
    }

    .path-selector:hover {
        border-color: var(--accent-primary);
    }

    .path-text {
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    /* Tracker 预览 */
    .tracker-preview {
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
        border: 1px solid var(--border-subtle);
        background: var(--surface-hover);
        color: var(--text-secondary);
        cursor: pointer;
        transition: all 0.2s;
    }

    .mini-btn.primary {
        background: var(--accent-primary);
        color: white;
        border-color: transparent;
    }

    .mini-btn.primary:hover {
        filter: brightness(1.1);
    }

    .preview-content {
        font-family: monospace;
        font-size: 11px;
        color: var(--text-muted);
        white-space: pre-wrap;
        max-height: 60px;
        overflow-y: auto;
        padding: 6px;
        background: rgba(0, 0, 0, 0.1);
        border-radius: 4px;
    }

    /* Tracker 输入 */
    .tracker-textarea {
        width: 100%;
        height: 100px;
        padding: 10px 14px;
        background: var(--input-bg, rgba(255, 255, 255, 0.05));
        border: 1px solid var(--border-color, rgba(255, 255, 255, 0.1));
        border-radius: 10px;
        color: var(--text-primary);
        font-size: 12px;
        font-family: monospace;
        outline: none;
        resize: vertical;
        transition: all 0.2s;
        line-height: 1.5;
    }

    .tracker-textarea:focus {
        border-color: var(--accent-primary);
        box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent-primary) 15%, transparent);
    }

    /* Footer */
    .footer-layout {
        display: flex;
        width: 100%;
        justify-content: flex-end;
        gap: 10px;
    }

    .btn-cancel {
        padding: 8px 16px;
        background: transparent;
        border: 1px solid var(--border-color);
        color: var(--text-secondary);
        border-radius: 10px;
        font-size: 13px;
        cursor: pointer;
        transition: all 0.2s;
    }

    .btn-cancel:hover {
        background: var(--surface-hover);
        color: var(--text-primary);
    }

    .btn-confirm {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 8px 18px;
        background: linear-gradient(135deg, var(--accent-primary), var(--accent-secondary));
        color: white;
        border: none;
        border-radius: 10px;
        font-size: 13px;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.2s;
        box-shadow: 0 4px 12px var(--accent-glow);
    }

    .btn-confirm:hover:not(:disabled) {
        transform: translateY(-1px);
        filter: brightness(1.1);
    }

    .btn-confirm:disabled {
        opacity: 0.6;
        cursor: not-allowed;
        box-shadow: none;
    }

    .spin {
        animation: spin 1s linear infinite;
    }

    @keyframes spin {
        from { transform: rotate(0deg); }
        to { transform: rotate(360deg); }
    }
</style>
