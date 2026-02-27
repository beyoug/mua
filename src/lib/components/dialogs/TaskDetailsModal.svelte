<!--
  TaskDetailsModal.svelte
  任务详情弹窗 - 使用 BaseModal 统一管理
-->
<script lang="ts">
	import { X, File, Folder, Link, CheckCircle, AlertCircle, LayoutDashboard, Shield, Settings, Copy, Info, Check, Clock, Calendar } from '@lucide/svelte';
	import { fade } from 'svelte/transition';
	import type { DownloadState } from '$lib/types/download';
    import BaseModal from '../common/BaseModal.svelte';
	import { createLogger } from '$lib/utils/logger';

	const logger = createLogger('TaskDetailsModal');

	interface Props {
		open?: boolean;
		filename: string;
		url: string;
		state: DownloadState;
		savePath?: string;
		errorMessage?: string;
		userAgent?: string;
		referer?: string;
		proxy?: string;
		headers?: string[];
		addedAt?: string;
		completedAt?: string | null;
		onOpenFolder?: () => void;
		onClose: () => void;
	}

	let { 
        open = false, 
        filename, 
        url, 
        state: downloadState, 
        savePath = '', 
        errorMessage = '', 
        userAgent = '', 
        referer = '', 
        proxy = '', 
        headers = [], 
        addedAt = '', 
        completedAt = null, 
        onOpenFolder,
        onClose 
    }: Props = $props();

    let activeTab = $state<'basic' | 'advanced'>('basic');
    let copiedField = $state<string | null>(null);

    // 提取文件扩展名
    // 提取文件扩展名
    const fileExtension = $derived(() => {
        if (!filename) return '未知';
        const lastDotIndex = filename.lastIndexOf('.');
        if (lastDotIndex === -1 || lastDotIndex === 0 || lastDotIndex === filename.length - 1) return '未知';
        return filename.substring(lastDotIndex + 1).toUpperCase();
    });

	function getStateLabel(s: DownloadState): string {
		switch (s) {
			case 'active': return '下载中';
			case 'paused': return '已暂停';
			case 'complete': return '已完成';
			case 'error': return '下载失败';
			case 'waiting': return '等待中';
			case 'removed': return '已取消';
			default: return s;
		}
	}

	function formatDateTime(isoString: string): string {
		if (!isoString) return '-';
		try {
			const date = new Date(isoString);
			return date.toLocaleString('zh-CN', {
				year: 'numeric',
				month: '2-digit',
				day: '2-digit',
				hour: '2-digit',
				minute: '2-digit',
				second: '2-digit',
				hour12: false
			}).replace(/\//g, '-');
		} catch (e) {
			return isoString;
		}
	}

	function getStopLabel(s: DownloadState): string {
		switch (s) {
			case 'complete': return '完成时间';
			case 'removed': return '取消时间';
			case 'error': return '失败时间';
			default: return '结束时间';
		}
	}

	async function copyToClipboard(text: string, field?: string) {
		try {
			await navigator.clipboard.writeText(text);
            if (field) {
                copiedField = field;
                setTimeout(() => { copiedField = null; }, 2000);
            }
		} catch (e) {
			logger.error('Failed to copy field value', { field, error: e });
		}
	}
</script>

<BaseModal 
    {open} 
    onClose={onClose} 
    size="md" 
    title="任务详情"
    minHeight="400px"
>
    {#snippet header()}
        <div class="details-header">
            <h3 class="modal-title">任务详情</h3>
            <div class="tab-sc-container">
                <div class="modal-tabs">
                    <button 
                        class="tab-btn" 
                        class:active={activeTab === 'basic'} 
                        onclick={() => activeTab = 'basic'}
                    >
                        <Info size={14} />
                        <span>基本信息</span>
                    </button>
                    <button 
                        class="tab-btn" 
                        class:active={activeTab === 'advanced'} 
                        onclick={() => activeTab = 'advanced'}
                    >
                        <Settings size={14} />
                        <span>高级设置</span>
                        {#if userAgent || referer || proxy || (headers && headers.length > 0)}
                            <span class="dot"></span>
                        {/if}
                    </button>
                    <div class="active-indicator" style="transform: translateX({activeTab === 'basic' ? '0' : '100%'})"></div>
                </div>
            </div>
        </div>
    {/snippet}

    <div class="details-body">
        {#if activeTab === 'basic'}
            <div class="tab-content" in:fade={{ duration: 150 }}>
                <div class="detail-row">
                    <div class="detail-label">
                        <File size={14} />
                        <span>文件信息</span>
                    </div>
                    <div class="detail-value">
                        <div class="detail-value-box identity-box">
                            <div class="identity-main">
                                <div class="filename-row">
                                    <span class="extension-badge" class:unknown={fileExtension() === '未知'}>{fileExtension()}</span>
                                    <span class="value-text filename" title={filename}>{filename}</span>
                                </div>
                                {#if savePath}
                                    <div class="identity-sub">
                                        <Folder size={12} class="sub-icon" />
                                        <span class="value-text path-text" title={savePath}>{savePath}</span>
                                        {#if ['active', 'paused', 'complete'].includes(downloadState)}
                                            <div class="sub-actions">
                                                <button class="item-copy-btn" onclick={onOpenFolder} title="在访达中打开">
                                                    <Folder size={12} />
                                                </button>
                                            </div>
                                        {/if}
                                    </div>
                                {/if}
                            </div>
                        </div>
                    </div>
                </div>

                <div class="detail-row">
                    <div class="detail-label">
                        <Link size={14} />
                        <span>下载链接</span>
                    </div>
                    <div class="detail-value">
                        <div class="detail-value-box">
                            <span class="value-text mono" title={url}>{url}</span>
                            <button class="item-copy-btn" onclick={() => copyToClipboard(url, 'url')} title="复制链接">
                                {#if copiedField === 'url'}
                                    <Check size={12} class="success" />
                                {:else}
                                    <Copy size={12} />
                                {/if}
                            </button>
                        </div>
                    </div>
                </div>

                <div class="detail-row">
                    <div class="detail-label">
                        <Calendar size={14} />
                        <span>任务状态与记录</span>
                    </div>
                    <div class="timeline-narrative">
                        <div class="detail-value-box narrative-row" class:error={downloadState === 'error'}>
                            <div class="timeline-segment">
                                <div class="status-pill ui-pill status-start">
                                    <div class="status-dot"></div>
                                    <span>开始</span>
                                </div>
                                <span class="timeline-text">于</span>
                                <span class="value-text mono">{formatDateTime(addedAt)}</span>
                            </div>

                            <div class="timeline-segment">
                                <div class="status-pill ui-pill status-{downloadState}">
                                    {#if downloadState === 'active'}
                                        <div class="status-dot pulsing"></div>
                                    {:else if downloadState === 'complete'}
                                        <Check size={10} />
                                    {:else if downloadState === 'error'}
                                        <AlertCircle size={10} />
                                    {:else if downloadState === 'paused'}
                                        <Info size={10} />
                                    {/if}
                                    <span>{getStateLabel(downloadState)}</span>
                                </div>
                                {#if completedAt}
                                    <span class="timeline-text">于</span>
                                    <span class="value-text mono">{formatDateTime(completedAt)}</span>
                                {/if}
                            </div>
                        </div>

                        {#if downloadState === 'error' && errorMessage}
                            <div class="detail-value-box error-reason-box">
                                <AlertCircle size={14} class="error-icon" />
                                <span class="value-text error-msg" title={errorMessage}>{errorMessage}</span>
                            </div>
                        {/if}
                    </div>
                </div>

            </div>
        {:else}
            <div class="tab-content" in:fade={{ duration: 150 }}>
                {#if !userAgent && !referer && !proxy && (!headers || headers.length === 0)}
                    <div class="empty-advanced">
                        <Info size={32} />
                        <p>未配置任何高级设置</p>
                    </div>
                {:else}
                    <div class="advanced-groups">
                        {#if userAgent}
                            <div class="detail-row">
                                <div class="detail-label">
                                    <LayoutDashboard size={14} />
                                    <span>User Agent</span>
                                </div>
                                <div class="detail-value">
                                    <div class="detail-value-box">
                                        <span class="value-text mono multi-line" title={userAgent}>{userAgent}</span>
                                        <button class="item-copy-btn" onclick={() => copyToClipboard(userAgent, 'ua')} title="复制 UA">
                                            {#if copiedField === 'ua'}
                                                <Check size={12} class="success" />
                                            {:else}
                                                <Copy size={12} />
                                            {/if}
                                        </button>
                                    </div>
                                </div>
                            </div>
                        {/if}

                        {#if referer}
                            <div class="detail-row">
                                <div class="detail-label">
                                    <Link size={14} />
                                    <span>Referer</span>
                                </div>
                                <div class="detail-value">
                                    <div class="detail-value-box">
                                        <span class="value-text mono" title={referer}>{referer}</span>
                                        <button class="item-copy-btn" onclick={() => copyToClipboard(referer, 'ref')} title="复制 Referer">
                                            {#if copiedField === 'ref'}
                                                <Check size={12} class="success" />
                                            {:else}
                                                <Copy size={12} />
                                            {/if}
                                        </button>
                                    </div>
                                </div>
                            </div>
                        {/if}

                        {#if proxy}
                            <div class="detail-row">
                                <div class="detail-label">
                                    <Shield size={14} />
                                    <span>代理服务器</span>
                                </div>
                                <div class="detail-value">
                                    <div class="detail-value-box">
                                        <span class="value-text mono" title={proxy}>{proxy}</span>
                                        <button class="item-copy-btn" onclick={() => copyToClipboard(proxy, 'proxy')} title="复制代理">
                                            {#if copiedField === 'proxy'}
                                                <Check size={12} class="success" />
                                            {:else}
                                                <Copy size={12} />
                                            {/if}
                                        </button>
                                    </div>
                                </div>
                            </div>
                        {/if}

                        {#if headers && headers.length > 0}
                            <div class="detail-row">
                                <div class="detail-label">
                                    <Settings size={14} />
                                    <span>自定义 Headers</span>
                                </div>
                                <div class="header-rows-list">
                                    {#each headers as header, i}
                                        <div class="detail-value-box header-item-row">
                                            <span class="value-text mono">{header}</span>
                                            <button class="item-copy-btn" onclick={() => copyToClipboard(header, `h-${i}`)} title="复制 Header">
                                                {#if copiedField === `h-${i}`}
                                                    <Check size={12} class="success" />
                                                {:else}
                                                    <Copy size={12} />
                                                {/if}
                                            </button>
                                        </div>
                                    {/each}
                                </div>
                            </div>
                        {/if}
                    </div>
                {/if}
            </div>
        {/if}
    </div>

    {#snippet footer()}
        <button class="btn-secondary ui-btn-footer ui-btn-secondary ui-btn-focus ui-disabled" onclick={onClose}>关闭</button>
    {/snippet}
</BaseModal>

<style>
    .details-header {
        display: flex;
        flex-direction: column;
        gap: 12px;
        width: 100%;
    }

    .modal-title {
        font-size: 16px;
        font-weight: 600;
        color: var(--text-primary);
        margin: 0;
    }

    /* Tab 样式重构 - Segmented Control Style */
    .tab-sc-container {
        padding: 4px 16px;
        width: 100%;
    }

    .modal-tabs {
        display: flex;
        position: relative;
        background: var(--input-bg, rgba(0, 0, 0, 0.08));
        border-radius: 12px;
        padding: 3px;
        box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.05);
    }

    .tab-btn {
        flex: 1;
        padding: 8px 0;
        background: transparent;
        border: none;
        font-size: 13px;
        font-weight: 500;
        color: var(--text-secondary);
        cursor: pointer;
        transition: color 0.3s ease;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 8px;
        position: relative;
        z-index: 1;
    }

    .tab-btn.active {
        color: var(--accent-primary);
        font-weight: 600;
    }

    .tab-btn .dot {
        position: absolute;
        top: 10px;
        right: 25%;
        width: 5px;
        height: 5px;
        background: var(--accent-primary);
        border-radius: 50%;
        box-shadow: 0 0 8px var(--accent-primary);
    }

    .active-indicator {
        position: absolute;
        top: 3px;
        bottom: 3px;
        left: 3px;
        width: calc((100% - 6px) / 2);
        background: var(--glass-bg, rgba(255, 255, 255, 0.1));
        backdrop-filter: blur(8px);
        border-radius: 9px;
        transition: transform 0.4s cubic-bezier(0.16, 1, 0.3, 1);
        border: 1px solid var(--border-normal, rgba(255, 255, 255, 0.15));
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
    }

    .details-body {
        padding: 20px;
        display: flex;
        flex-direction: column;
        gap: 16px;
    }

    .tab-content {
        display: flex;
        flex-direction: column;
        gap: 14px;
    }

    .detail-row {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.detail-label {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 10px;
		font-weight: 600;
		color: var(--text-tertiary);
        text-transform: uppercase;
        letter-spacing: 0.03em;
        padding-left: 1px;
	}

    .detail-value-box {
        display: flex;
        align-items: center;
        gap: 10px;
        background: var(--input-bg, rgba(0, 0, 0, 0.05));
        padding: 8px 12px;
        border-radius: 10px;
        border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.05));
        transition: all 0.2s ease;
        min-height: 36px;
    }

    .detail-value-box:hover {
        border-color: var(--border-normal);
        background: var(--surface-hover);
    }

    .detail-value-box.error {
        background: color-mix(in srgb, var(--semantic-danger) 8%, transparent);
        border-color: color-mix(in srgb, var(--semantic-danger) 28%, transparent);
    }

    .value-text {
        flex: 1;
        font-size: 13px;
        color: var(--text-primary);
        word-break: break-all;
        line-height: 1.4;
    }

    .value-text.mono {
        font-family: var(--font-mono, monospace);
        font-size: 12px;
        color: var(--text-secondary);
        font-variant-numeric: tabular-nums;
    }

    .value-text.filename {
        font-size: 14px;
        font-weight: 600;
        color: var(--text-primary);
        line-height: 1.4;
        word-break: break-all;
    }

    .filename-row {
        display: flex;
        align-items: center;
        gap: 8px;
        flex-wrap: wrap;
    }

    .extension-badge {
        font-size: 10px;
        font-weight: 700;
        color: var(--accent-primary);
        background: color-mix(in srgb, var(--accent-primary) 10%, transparent);
        padding: 2px 6px;
        border-radius: 4px;
        text-transform: uppercase;
        letter-spacing: 0.5px;
        border: 1px solid color-mix(in srgb, var(--accent-primary) 20%, transparent);
        flex-shrink: 0;
        align-self: flex-start;
        margin-top: 1px; /* Optical alignment with text */
    }

    .extension-badge.unknown {
        color: var(--semantic-warning);
        background: color-mix(in srgb, var(--semantic-warning) 10%, transparent);
        border-color: color-mix(in srgb, var(--semantic-warning) 20%, transparent);
    }

    .identity-box {
        display: flex;
        align-items: center;
        gap: 12px;
        padding: 10px 12px !important;
    }

    .identity-main {
        flex: 1;
        min-width: 0;
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .identity-sub {
        display: flex;
        align-items: center;
        gap: 6px;
        min-width: 0; 
        max-width: 100%;
    }

    .sub-actions {
        display: flex;
        align-items: center;
        gap: 4px;
        margin-left: 2px;
        flex-shrink: 0;
    }

    /* Small variant for inline buttons */
    .item-copy-btn.small {
        width: 20px;
        height: 20px;
        padding: 0;
    }

    :global(.sub-icon) {
        flex-shrink: 0;
        color: var(--text-tertiary);
    }

    .path-text {
        flex: 1;
        min-width: 0;
        font-size: 11px;
        color: var(--text-tertiary);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        font-variant-numeric: tabular-nums;
    }

    .status-pill {
        font-size: 11px;
    }

    .status-dot {
        width: 6px;
        height: 6px;
        border-radius: 50%;
        background: currentColor;
    }

    .status-dot.pulsing {
        animation: status-pulse 1.5s ease-in-out infinite;
    }

    @keyframes status-pulse {
        0% { opacity: 0.4; transform: scale(0.8); }
        50% { opacity: 1; transform: scale(1.1); }
        100% { opacity: 0.4; transform: scale(0.8); }
    }

    .status-pill.status-active {
        background: color-mix(in srgb, var(--accent-primary) 10%, transparent);
        color: var(--accent-primary);
    }

    .status-pill.status-complete {
        background: color-mix(in srgb, var(--semantic-success, #10b981) 10%, transparent);
        color: var(--semantic-success, #10b981);
    }

    .status-pill.status-paused {
        background: color-mix(in srgb, var(--semantic-warning) 10%, transparent);
        color: var(--semantic-warning);
    }

    .status-pill.status-error {
        background: color-mix(in srgb, var(--semantic-danger) 10%, transparent);
        color: var(--semantic-danger);
    }

    .status-pill.status-removed {
        background: color-mix(in srgb, var(--text-tertiary) 10%, transparent);
        color: var(--text-tertiary);
    }

    .status-pill.status-start {
        background: color-mix(in srgb, var(--semantic-success, #10b981) 10%, transparent);
        color: var(--semantic-success, #10b981);
    }

    .item-copy-btn {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 28px;
        height: 28px;
        background: var(--surface-hover);
        border: 1px solid var(--border-color);
        border-radius: 8px;
        color: var(--text-tertiary);
        cursor: pointer;
        transition: all 0.2s;
    }

    .item-copy-btn:hover {
        background: var(--surface-active);
        color: var(--accent-primary);
        border-color: var(--accent-primary);
    }

    .item-copy-btn :global(.success) { color: #10b981; }

    .header-rows-list { display: flex; flex-direction: column; gap: 8px; }

    .timeline-narrative {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .timeline-step {
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .narrative-row {
        display: flex;
        align-items: center;
        gap: 16px;
        padding: 4px 12px !important;
        min-height: 40px !important;
        overflow-x: auto;
        -ms-overflow-style: none;
        scrollbar-width: none;
    }

    .narrative-row::-webkit-scrollbar {
        display: none;
    }

    .timeline-segment {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 6px 0;
        white-space: nowrap;
        flex-shrink: 0;
    }

    .timeline-text {
        font-size: 11px;
        color: var(--text-tertiary);
        white-space: nowrap;
    }

    .error-reason-box {
        background: color-mix(in srgb, var(--semantic-danger) 10%, transparent) !important;
        border-color: color-mix(in srgb, var(--semantic-danger) 28%, transparent) !important;
        padding: 10px 12px !important;
    }

    .error-icon {
        color: var(--semantic-danger);
        flex-shrink: 0;
    }

    .error-msg {
        color: var(--semantic-danger) !important;
        font-weight: 500;
        font-size: 12px;
    }

    .empty-advanced {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        padding: 40px 0;
        color: var(--text-tertiary);
        gap: 12px;
    }

</style>
