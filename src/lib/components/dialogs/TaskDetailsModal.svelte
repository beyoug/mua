<!--
  TaskDetailsModal.svelte
  任务详情弹窗 - 显示任务详细信息
-->
<script lang="ts">
	import { X, File, Link, CheckCircle, AlertCircle, LayoutDashboard, Shield, Settings, Copy, Info, Check } from '@lucide/svelte';
	import { fade, scale } from 'svelte/transition';
	import type { DownloadState } from '$lib/types/download';

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
		onClose: () => void;
	}

	let { open = false, filename, url, state: downloadState, savePath = '', errorMessage = '', userAgent = '', referer = '', proxy = '', headers = [], onClose }: Props = $props();

    let activeTab = $state<'basic' | 'advanced'>('basic');
    let copiedField = $state<string | null>(null);

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			onClose();
		}
	}

	function getStateLabel(s: DownloadState): string {
		switch (s) {
			case 'downloading': return '下载中';
			case 'paused': return '已暂停';
			case 'completed': return '已完成';
			case 'error': return '下载失败';
			case 'waiting': return '等待中';
			case 'cancelled': return '已取消';
			default: return s;
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
			console.error('Failed to copy', e);
		}
	}
</script>

{#if open}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div 
		class="task-details-modal-overlay" 
		in:fade={{ duration: 150 }} 
		out:fade={{ duration: 100 }}
		onkeydown={handleKeydown}
		onclick={onClose}
	>
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div 
			class="task-details-modal" 
			in:scale={{ duration: 150, start: 0.95, opacity: 0.5 }}
			out:fade={{ duration: 80 }}
			onclick={(e) => e.stopPropagation()}
		>
			<header class="modal-header">
				<h3>任务详情</h3>
				<button class="close-btn" onclick={onClose} aria-label="关闭">
					<X size={18} />
				</button>
			</header>

            <div class="modal-tabs">
                <button 
                    class="tab-btn" 
                    class:active={activeTab === 'basic'} 
                    onclick={() => activeTab = 'basic'}
                >
                    基本信息
                </button>
                <button 
                    class="tab-btn" 
                    class:active={activeTab === 'advanced'} 
                    onclick={() => activeTab = 'advanced'}
                >
                    高级设置
                    {#if userAgent || referer || proxy || (headers && headers.length > 0)}
                        <span class="dot"></span>
                    {/if}
                </button>
                <div class="active-indicator" style="transform: translateX({activeTab === 'basic' ? '0' : '100%'})"></div>
            </div>

			<div class="modal-body">
                {#if activeTab === 'basic'}
                    <div class="tab-content" in:fade={{ duration: 150 }}>
                        <div class="detail-row">
                            <div class="detail-label">
                                <File size={14} />
                                <span>文件名</span>
                            </div>
                            <div class="detail-value">
                                <div class="detail-value-box">
                                    <span class="value-text filename" title={filename}>{filename}</span>
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
                                <CheckCircle size={14} />
                                <span>当前状态</span>
                            </div>
                            <div class="detail-value">
                                <div class="detail-value-box" class:error={downloadState === 'error'}>
                                    <div class="status-pill status-{downloadState}">
                                        {#if downloadState === 'downloading'}
                                            <div class="status-dot pulsing"></div>
                                        {:else if downloadState === 'completed'}
                                            <Check size={10} />
                                        {:else if downloadState === 'error'}
                                            <AlertCircle size={10} />
                                        {:else if downloadState === 'paused'}
                                            <Info size={10} />
                                        {/if}
                                        <span>{getStateLabel(downloadState)}</span>
                                    </div>
                                    {#if downloadState === 'error' && errorMessage}
                                        <div class="status-error-divider"></div>
                                        <span class="value-text error-msg" title={errorMessage}>{errorMessage}</span>
                                    {/if}
                                </div>
                            </div>
                        </div>

                        {#if savePath}
                            <div class="detail-row">
                                <div class="detail-label">
                                    <File size={14} />
                                    <span>保存路径</span>
                                </div>
                                <div class="detail-value">
                                    <div class="detail-value-box">
                                        <span class="value-text mono" title={savePath}>{savePath}</span>
                                        <button class="item-copy-btn" onclick={() => copyToClipboard(savePath, 'path')} title="复制路径">
                                            {#if copiedField === 'path'}
                                                <Check size={12} class="success" />
                                            {:else}
                                                <Copy size={12} />
                                            {/if}
                                        </button>
                                    </div>
                                </div>
                            </div>
                        {/if}
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

			<footer class="modal-footer">
				<button class="btn btn-secondary" onclick={onClose}>关闭</button>
			</footer>
		</div>
	</div>
{/if}

<style>
	/* Use unique class names and global to escape stacking context */
	.task-details-modal-overlay {
		position: fixed;
		inset: 0;
		background: var(--dialog-overlay-bg, rgba(0, 0, 0, 0.5));
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 9999;
	}

	.task-details-modal {
		width: 90%;
		max-width: 480px;
        max-height: 85vh;
		background: var(--dialog-bg);
		backdrop-filter: var(--glass-blur) var(--glass-saturate);
		-webkit-backdrop-filter: var(--glass-blur) var(--glass-saturate);
		border: 1px solid var(--glass-border);
		border-radius: 16px;
		box-shadow: var(--glass-shadow), 0 25px 50px -12px rgba(0, 0, 0, 0.25);
		overflow: hidden;
		z-index: 10000;
        display: flex;
        flex-direction: column;
	}

	.modal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px 20px;
		border-bottom: 1px solid var(--border-subtle);
	}

	.modal-header h3 {
		font-size: 15px;
		font-weight: 600;
		color: var(--text-primary);
		margin: 0;
	}

	.close-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		background: transparent;
		border: none;
		border-radius: 6px;
		color: var(--text-muted);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.close-btn:hover {
		background: var(--surface-hover);
		color: var(--text-primary);
	}

	.modal-body {
		padding: 20px;
		display: flex;
		flex-direction: column;
		gap: 16px;
        overflow-y: auto;
        flex: 1;
        /* 自定义滚动条样式 */
        scrollbar-width: thin;
        scrollbar-color: var(--border-subtle) transparent;
	}

    .modal-body::-webkit-scrollbar {
        width: 6px;
    }

    .modal-body::-webkit-scrollbar-track {
        background: transparent;
    }

    .modal-body::-webkit-scrollbar-thumb {
        background: var(--border-subtle);
        border-radius: 10px;
    }

    .modal-body::-webkit-scrollbar-thumb:hover {
        background: var(--border-normal);
    }

	.detail-row {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.detail-label {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 11px;
		font-weight: 600;
		color: var(--text-tertiary);
        text-transform: uppercase;
        letter-spacing: 0.05em;
        padding-left: 2px;
	}

	.detail-value {
		width: 100%;
	}

    .detail-value-box {
        display: flex;
        align-items: center;
        gap: 10px;
        background: rgba(255, 255, 255, 0.03);
        padding: 8px 12px;
        border-radius: 10px;
        border: 1px solid var(--border-subtle);
        transition: all 0.2s ease;
        min-height: 38px;
    }

    .detail-value-box:hover {
        border-color: var(--border-normal);
        background: rgba(255, 255, 255, 0.05);
    }

    .detail-value-box.error {
        background: rgba(239, 68, 68, 0.05);
        border-color: rgba(239, 68, 68, 0.2);
        box-shadow: 0 0 12px rgba(239, 68, 68, 0.05);
    }

    .status-error-divider {
        width: 1px;
        height: 14px;
        background: rgba(239, 68, 68, 0.2);
        flex-shrink: 0;
    }

    .value-text {
        flex: 1;
        font-size: 13px;
        color: var(--text-primary);
        word-break: break-all;
        line-height: 1.4;
    }

    .value-text.mono {
        font-family: var(--font-mono);
        font-size: 11px;
        color: var(--text-secondary);
    }

    .value-text.filename {
        font-weight: 500;
    }

    .value-text.multi-line {
        display: -webkit-box;
        -webkit-line-clamp: 2;
        -webkit-box-orient: vertical;
        overflow: hidden;
    }

    .value-text.error-msg {
        color: var(--semantic-danger);
        font-family: var(--font-mono);
        font-size: 11px;
        opacity: 0.85;
        font-weight: 500;
    }

    .status-pill {
        display: inline-flex;
        align-items: center;
        gap: 6px;
        padding: 4px 10px;
        font-size: 11px;
        font-weight: 600;
        border-radius: 20px;
        white-space: nowrap;
        flex-shrink: 0;
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

    .status-pill.status-downloading {
        background: color-mix(in srgb, var(--accent-primary) 10%, transparent);
        color: var(--accent-primary);
    }

    .status-pill.status-completed {
        background: color-mix(in srgb, var(--semantic-success) 10%, transparent);
        color: var(--semantic-success);
    }

    .status-pill.status-paused {
        background: color-mix(in srgb, var(--semantic-warning) 10%, transparent);
        color: var(--semantic-warning);
    }

    .status-pill.status-error {
        background: color-mix(in srgb, #ef4444 12%, transparent);
        color: #ef4444;
        padding: 4px 8px;
    }

    .status-pill.status-cancelled,
    .status-pill.status-waiting {
        background: rgba(255, 255, 255, 0.05);
        color: var(--text-tertiary);
    }

    .item-copy-btn {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 26px;
        height: 26px;
        background: var(--surface-hover);
        border: 1px solid var(--border-subtle);
        border-radius: 6px;
        color: var(--text-tertiary);
        cursor: pointer;
        transition: all 0.15s ease;
        flex-shrink: 0;
    }

    .item-copy-btn:hover {
        background: var(--surface-active);
        color: var(--accent-primary);
        border-color: var(--accent-primary);
    }

    .item-copy-btn :global(.success) {
        color: var(--semantic-success);
    }

    /* Tab 样式 */
    .modal-tabs {
        display: flex;
        position: relative;
        padding: 0 20px;
        border-bottom: 1px solid var(--border-subtle);
        background: rgba(255, 255, 255, 0.02);
    }

    .tab-btn {
        flex: 1;
        padding: 12px 0;
        background: transparent;
        border: none;
        font-size: 13px;
        font-weight: 500;
        color: var(--text-tertiary);
        cursor: pointer;
        transition: all 0.2s ease;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 6px;
        position: relative;
    }

    .tab-btn:hover {
        color: var(--text-primary);
    }

    .tab-btn.active {
        color: var(--accent-primary);
    }

    .tab-btn .dot {
        width: 4px;
        height: 4px;
        border-radius: 50%;
        background: var(--accent-primary);
        opacity: 0.6;
    }

    .active-indicator {
        position: absolute;
        bottom: 0;
        left: 20px;
        width: calc((100% - 40px) / 2);
        height: 2px;
        background: var(--accent-primary);
        transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
        box-shadow: 0 0 10px rgba(var(--accent-primary-rgb), 0.5);
    }

    .tab-content {
        display: flex;
        flex-direction: column;
        gap: 20px;
    }

    /* 对齐调整 */
    .advanced-groups {
        display: flex;
        flex-direction: column;
        gap: 18px;
    }

    /* Header 列表重构样式 */
    .header-rows-list {
        display: flex;
        flex-direction: column;
        gap: 8px;
        margin-top: 4px;
    }

    .header-item-row {
        min-height: 34px;
    }

    .empty-advanced {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        padding: 60px 0;
        color: var(--text-tertiary);
        gap: 16px;
        opacity: 0.8;
    }

    .empty-advanced p {
        margin: 0;
        font-size: 13px;
    }

    .modal-footer {
        display: flex;
        justify-content: flex-end;
        padding: 16px 20px;
        border-top: 1px solid var(--border-subtle);
    }

    .btn {
        padding: 8px 16px;
        font-size: 13px;
        font-weight: 500;
        border-radius: 8px;
        cursor: pointer;
        transition: all 0.15s ease;
    }

    .btn-secondary {
        background: var(--surface-hover);
        border: 1px solid var(--border-subtle);
        color: var(--text-secondary);
    }

    .btn-secondary:hover {
        background: var(--surface-active);
        border-color: var(--border-normal);
        color: var(--text-primary);
    }
</style>
