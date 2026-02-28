<!--
  TorrentConfigDialog.svelte
  种子任务配置弹窗 - 拖拽/选择 torrent 后弹出
-->
<script lang="ts">
    import { Magnet, FolderOpen, Download, Network, RefreshCw, Plus, Import, Loader2 } from '@lucide/svelte';
    import { fade } from 'svelte/transition';
    import type { TorrentInfo } from '$lib/types/torrent';
    import { fetchTrackers as fetchTrackersService } from '$lib/services/aria2';
    import { formatBytes } from '$lib';
    import { appSettings, updateAppSettings } from '$lib/services/settings';
    import { createLogger } from '$lib/utils/logger';
    import { pickSingleDirectory } from '$lib/utils/dialog';
    import BaseModal from '../common/BaseModal.svelte';
    import TorrentFileSelector from './TorrentFileSelector.svelte';

    const logger = createLogger('TorrentConfigDialog');

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
            const path = await pickSingleDirectory('选择保存位置', savePath);
            if (path) savePath = path;
        } catch (e) {
            logger.warn('Failed to select torrent save directory', { error: e });
        }
    }

    async function fetchTrackers() {
        if (isFetchingTrackers) return;
        isFetchingTrackers = true;
        try {
            publicTrackers = await fetchTrackersService();
            showTrackerPreview = true;
        } catch (e) {
            logger.error('Failed to fetch trackers', { error: e });
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

    async function handleConfirm() {
        if (trackers.trim()) {
            try {
                await updateAppSettings({ btTrackers: trackers });
            } catch (e) {
                logger.error('Failed to save trackers', { error: e });
            }
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
                <div class="warning-title">
                    <span>⚠️ 种子信息预解析失败</span>
                </div>
                <p class="warning-desc">
                    {parseError}。您仍可点击下方按钮直接提交，由 Aria2 尝试完整解析。
                </p>
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
                                <Loader2 size={16} style="animation: spin 1s linear infinite;" />
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
            <button class="path-selector ui-field" onclick={selectFolder}>
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
                    <button class="mini-action ui-mini-action ui-btn-focus ui-disabled" onclick={importGlobalTrackers} title="从全局设置导入">
                        <Import size={12} />
                        <span>导入全局</span>
                    </button>
                    <button class="mini-action ui-mini-action ui-btn-focus ui-disabled" onclick={fetchTrackers} disabled={isFetchingTrackers}>
                        {#if isFetchingTrackers}
                                    <RefreshCw size={12} style="animation: spin 1s linear infinite;" />
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
                            <button class="mini-btn ui-btn-mini ui-btn-primary ui-btn-focus ui-disabled primary" onclick={appendTrackers}>
                                <Plus size={12} />
                                追加
                            </button>
                            <button class="mini-btn ui-btn-mini ui-btn-secondary ui-btn-focus ui-disabled" onclick={() => showTrackerPreview = false}>
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
                class="tracker-textarea ui-field"
                bind:value={trackers}
                placeholder="每行一个 Tracker URL&#10;udp://tracker.opentrackr.org:1337/announce"
                spellcheck="false"
                rows="4"
            ></textarea>
        </div>
    </div>

    {#snippet footer()}
        <div class="footer-layout">
            <button class="btn-cancel ui-btn-footer ui-btn-secondary ui-btn-focus ui-disabled" onclick={onCancel}>取消</button>
            <button class="btn-confirm ui-btn-footer ui-btn-primary ui-btn-focus ui-disabled" onclick={handleConfirm}>
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
        border: none;
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
        padding: 12px 14px;
        background: color-mix(in srgb, var(--semantic-warning) 10%, transparent);
        border: none;
        box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--semantic-warning) 30%, transparent);
        border-radius: 10px;
        color: var(--semantic-warning);
        display: flex;
        flex-direction: column;
        gap: 4px;
    }
 
    .warning-title {
        font-weight: 600;
        font-size: 13px;
    }
 
    .warning-desc {
        font-size: 12px;
        line-height: 1.5;
        margin: 0;
        opacity: 0.9;
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
        font-size: 11px;
    }

    /* 路径选择器 */
    .path-selector {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 10px 14px;
        font-size: 13px;
        cursor: pointer;
        text-align: left;
    }

    .path-selector:hover {
        box-shadow: var(--hover-ring-medium);
    }

    .path-text {
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    /* Tracker 预览 */
    .tracker-preview {
        background: var(--surface-active);
        border: none;
        box-shadow: inset 0 0 0 1px var(--accent-dim);
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

    .mini-btn.primary {
        color: white;
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
        font-size: 12px;
        font-family: monospace;
        resize: vertical;
        line-height: 1.5;
    }

    /* Footer */
    .footer-layout {
        display: flex;
        width: 100%;
        justify-content: flex-end;
        gap: 10px;
    }

    .btn-confirm {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 6px;
        color: white;
    }

    @keyframes spin {
        from { transform: rotate(0deg); }
        to { transform: rotate(360deg); }
    }
</style>
