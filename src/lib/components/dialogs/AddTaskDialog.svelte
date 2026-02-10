<!--
  AddTaskDialog.svelte
  添加下载任务对话框 - 使用 BaseModal 统一管理
-->
<script lang="ts">
	import { X, Link, FolderOpen, Download, Settings, Globe, FileText, Shield, Gauge, ArrowLeft, AlertCircle, ChevronRight, ChevronDown } from '@lucide/svelte';
	import { open as openDialog } from '@tauri-apps/plugin-dialog';
	import { fade } from 'svelte/transition';
	import type { DownloadConfig } from '$lib/types/download';
	import { isValidDownloadUrl, validateUrl } from '$lib';
	import { appSettings, saveAppSettings } from '$lib/stores/settings';
	import BaseModal from '../common/BaseModal.svelte';
	import UaSelector from './UaSelector.svelte';



	interface Props {
		open: boolean;
		onClose: () => void;
		onSubmit?: (config: DownloadConfig) => void;
	}

	let { open, onClose, onSubmit }: Props = $props();

	// 基础设置
	let urls = $state('');
	let savePath = $state($appSettings.defaultSavePath || '~/Downloads');
	let filename = $state('');

	// 高级设置管理
	let showAdvanced = $state(false);
    let advancedSnapshot = $state<any>(null);
	let selectedUaValue = $state('');
	let customUserAgent = $state('');
	let referer = $state('');
	let headers = $state('');
	let proxy = $state('');
	let maxDownloadLimitValue = $state('');
	let maxDownloadLimitUnit = $state('M');

	// URL 验证
	let validationError = $state<string>('');
	let validationTimer: ReturnType<typeof setTimeout> | null = null;



	const effectiveUserAgent = $derived(selectedUaValue === 'custom' ? customUserAgent : selectedUaValue);
	const canSubmit = $derived(isValidDownloadUrl(urls.trim()));
    const isCustomUaInvalid = $derived(selectedUaValue === 'custom' && !customUserAgent.trim());

	let uaSelectorRef = $state<UaSelector>();
	let isSubmitting = $state(false);

	async function handleSubmit() {
		if (isSubmitting) return;
		isSubmitting = true;

		const error = validateUrl(urls);
		validationError = error;
		if (error) {
			isSubmitting = false;
			return;
		}
		
		const trimmedUrl = urls.trim();
		try {
            const limitStr = String(maxDownloadLimitValue || '').trim();
            const limit = limitStr ? `${limitStr}${maxDownloadLimitUnit}` : '';
            const finalUa = effectiveUserAgent;
			
			onSubmit?.({
				urls: [trimmedUrl],
				savePath,
				filename,
				userAgent: finalUa,
				referer,
				headers,
				proxy,
				maxDownloadLimit: limit
			});

            if (finalUa && uaSelectorRef && !uaSelectorRef.isBuiltinUa(finalUa)) {
                let history = [...($appSettings.uaHistory || [])];
                history = [finalUa, ...history.filter(ua => ua !== finalUa)];
                if (history.length > 10) history = history.slice(0, 10);
                await saveAppSettings({ ...$appSettings, uaHistory: history });
            }

			resetForm();
			onClose();
		} catch (e) {
			console.error('Failed to add task:', e);
			validationError = typeof e === 'string' ? e : '添加任务失败，请检查 Aria2 服务是否正常';
		} finally {
			isSubmitting = false;
		}
	}

	function resetForm() {
		urls = '';
        savePath = $appSettings.defaultSavePath || '~/Downloads';
		filename = '';
		selectedUaValue = '';
		customUserAgent = '';
        isSubmitting = false;
		referer = '';
		headers = '';
		proxy = '';
		maxDownloadLimitValue = '';
		maxDownloadLimitUnit = 'M';
		showAdvanced = false;
		validationError = '';
	}

	async function selectFolder() {
		try {
			const selected = await openDialog({
				directory: true,
				multiple: false,
				title: '选择下载目录'
			});
			if (selected) savePath = selected as string;
		} catch (e) {}
	}

    function openAdvanced() {
        // 进入高级设置前记录快照，用于取消时还原
        advancedSnapshot = {
            selectedUaValue,
            customUserAgent,
            referer,
            headers,
            proxy,
            maxDownloadLimitValue,
            maxDownloadLimitUnit
        };
        showAdvanced = true;
    }

    function handleBack() {
        if (advancedSnapshot) {
            // 还原到进入前的状态
            selectedUaValue = advancedSnapshot.selectedUaValue;
            customUserAgent = advancedSnapshot.customUserAgent;
            referer = advancedSnapshot.referer;
            headers = advancedSnapshot.headers;
            proxy = advancedSnapshot.proxy;
            maxDownloadLimitValue = advancedSnapshot.maxDownloadLimitValue;
            maxDownloadLimitUnit = advancedSnapshot.maxDownloadLimitUnit;
            advancedSnapshot = null;
        }
        showAdvanced = false;
    }

	// 清理定时器
	$effect(() => {
		return () => {
			if (validationTimer) clearTimeout(validationTimer);
		};
	});

	function handleUrlBlur() {
		if (validationTimer) {
			clearTimeout(validationTimer);
			validationTimer = null;
		}
		if (urls.trim()) {
			validationError = validateUrl(urls);
		} else {
			validationError = '';
		}
	}

	function handleUrlInput() {
		if (validationTimer) clearTimeout(validationTimer);
		validationTimer = setTimeout(() => {
			if (urls.trim()) {
				validationError = validateUrl(urls);
			} else {
				validationError = '';
			}
		}, 500);
	}
</script>

<BaseModal 
    {open} 
    onClose={onClose} 
    size="md" 
    minHeight="480px"
    showClose={!showAdvanced}
    closeOnClickOutside={false}
    closeOnEscape={false}
>
    {#snippet header()}
        {#if !showAdvanced}
            <h2 class="modal-title">添加下载任务</h2>
        {:else}
            <div class="advanced-header">
                <button class="back-link" onclick={handleBack}>
                    <ArrowLeft size={18} />
                </button>
                <div class="breadcrumb">
                    <span class="crumb-parent">添加下载任务</span>
                    <ChevronRight size={14} class="crumb-sep" />
                    <span class="crumb-current">高级设置</span>
                </div>
            </div>
        {/if}
    {/snippet}

    <div class="modal-content-stack">
        {#if !showAdvanced}
            <!-- 主面板 -->
            <div class="view-main" in:fade={{ duration: 150 }}>
                <div class="dialog-body">
                    <!-- 下载链接 -->
                    <div class="form-group">
                        <label for="urls">
                            <Link size={14} />
                            <span>下载链接</span>
                            {#if validationError}
                                <span class="error-inline">
                                    <AlertCircle size={12} />
                                    {validationError}
                                </span>
                            {/if}
                        </label>
                        <textarea
                            id="urls"
                            placeholder="输入单个下载 URL（支持 http/https/ftp 协议）"
                            bind:value={urls}
                            oninput={handleUrlInput}
                            onblur={handleUrlBlur}
                            class:error={!!validationError}
                        ></textarea>
                    </div>

                    <!-- 保存位置 -->
                    <div class="form-group">
                        <label>
                            <FolderOpen size={14} />
                            <span>保存位置</span>
                        </label>
                        <button class="path-selector" onclick={selectFolder}>
                            <span class="path-text">{savePath}</span>
                            <FolderOpen size={14} />
                        </button>
                    </div>

                    <!-- 保存文件名 -->
                    <div class="form-group">
                        <label>
                            <FileText size={14} />
                            <span>保存文件名</span>
                        </label>
                        <input
                            type="text"
                            class="text-input"
                            placeholder="留空则使用原始文件名"
                            bind:value={filename}
                        />
                    </div>
                </div>
            </div>
        {:else}
            <!-- 高级设置面板 -->
            <div class="advanced-panel" in:fade={{ duration: 150 }}>
                <div class="panel-body">
                    <div class="form-row">
                        <label>
                            <Globe size={14} />
                            <span>User Agent</span>
                        </label>
                        <UaSelector
                            bind:this={uaSelectorRef}
                            selectedValue={selectedUaValue}
                            customValue={customUserAgent}
                            onValueChange={(v) => selectedUaValue = v}
                            onCustomChange={(v) => customUserAgent = v}
                        />
                    </div>

                    <!-- Referer -->
                    <div class="form-row">
                        <label>
                            <Link size={14} />
                            <span>Referer</span>
                        </label>
                        <input type="text" placeholder="https://example.com" bind:value={referer} />
                    </div>

                    <!-- 自定义 Header -->
                    <div class="form-row">
                        <label>
                            <FileText size={14} />
                            <span>自定义 Header</span>
                        </label>
                        <textarea 
                            placeholder="Key: Value (每行一个)" 
                            bind:value={headers}
                            rows="2"
                            class="headers-textarea"
                        ></textarea>
                    </div>

                    <!-- 代理服务器 -->
                    <div class="form-row">
                        <label>
                            <Shield size={14} />
                            <span>代理服务器</span>
                        </label>
                        <input type="text" placeholder="[user:pass@]host:port (支持 http/socks5)" bind:value={proxy} />
                    </div>

                    <!-- 速度限制 -->
                    <div class="form-row">
                        <label>
                            <Gauge size={14} />
                            <span>速度限制</span>
                        </label>
                        <div class="input-group">
                            <input 
                                type="number" 
                                min="0" 
                                placeholder="0" 
                                class="grouped-input"
                                bind:value={maxDownloadLimitValue} 
                            />
                            <div class="input-divider"></div>
                            <div class="select-wrapper">
                                <select class="grouped-select" bind:value={maxDownloadLimitUnit}>
                                    <option value="M">MB/s</option>
                                    <option value="K">KB/s</option>
                                </select>
                                <ChevronDown size={14} class="select-icon" />
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        {/if}
    </div>

    {#snippet footer()}
        {#if !showAdvanced}
            <div class="footer-layout">
                <button class="btn-ghost" onclick={openAdvanced}>
                    <Settings size={14} />
                    <span>高级设置</span>
                </button>
                <button 
                    class="btn-primary" 
                    onclick={handleSubmit}
                    disabled={!canSubmit || isSubmitting}
                >
                    {#if isSubmitting}
                        <span>提交中...</span>
                    {:else}
                        <Download size={14} />
                        <span>开始下载</span>
                    {/if}
                </button>
            </div>
        {:else}
            <button 
                class="btn-primary" 
                onclick={() => { showAdvanced = false; advancedSnapshot = null; }}
                disabled={isCustomUaInvalid}
            >
                完成设置
            </button>
        {/if}
    {/snippet}
</BaseModal>

<style>
    .modal-title {
		font-size: 16px;
		font-weight: 600;
		color: var(--text-primary);
		margin: 0;
	}

    .advanced-header {
        display: flex;
        align-items: center;
        gap: 12px;
    }

    .back-link {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 32px;
        height: 32px;
        background: transparent;
        border: none;
        border-radius: 8px;
        color: var(--text-muted);
        cursor: pointer;
        transition: all 0.2s;
        margin-left: -8px;
    }

    .back-link:hover {
        background: var(--surface-hover);
        color: var(--text-primary);
    }

    .breadcrumb {
        display: flex;
        align-items: center;
        gap: 4px;
        font-size: 14px;
    }

    .crumb-parent { color: var(--text-muted); }
    :global(.crumb-sep) { color: var(--text-tertiary); opacity: 0.5; }
    .crumb-current { color: var(--text-primary); font-weight: 500; }

    .modal-content-stack {
        display: grid;
        grid-template-rows: 1fr;
        grid-template-columns: 1fr;
        flex: 1;
    }

    .view-main, .advanced-panel {
        grid-area: 1 / 1;
        padding: 24px;
        display: flex;
        flex-direction: column;
        gap: 20px;
    }

    .dialog-body, .panel-body {
        display: flex;
        flex-direction: column;
        gap: 16px;
    }

    .form-group, .form-row {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .form-group label, .form-row label {
        display: flex;
        align-items: center;
        gap: 6px;
        font-size: 13px;
        color: var(--text-secondary);
    }

    .error-inline {
        display: flex;
        align-items: center;
        gap: 4px;
        margin-left: auto;
        font-size: 12px;
        color: var(--danger-color);
    }

    textarea, input, .path-selector {
        padding: 12px 14px;
        background: var(--input-bg, rgba(255, 255, 255, 0.05));
        border: 1px solid var(--border-color, rgba(255, 255, 255, 0.1));
        border-radius: 10px;
        color: var(--text-primary);
        font-size: 14px;
        outline: none;
        transition: all 0.2s ease;
    }

    textarea:focus, input:focus {
        border-color: var(--accent-primary);
        box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent-primary) 15%, transparent);
    }

    textarea { height: 100px; resize: none; }
    textarea.error { border-color: var(--danger-color); }

    .path-selector {
        display: flex;
        align-items: center;
        justify-content: space-between;
        cursor: pointer;
        text-align: left;
    }

    .path-selector:hover { border-color: var(--accent-primary); }
    .path-text { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

    .footer-layout {
        display: flex;
        width: 100%;
        justify-content: space-between;
        align-items: center;
    }

    .btn-primary {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 10px 20px;
        background: linear-gradient(135deg, var(--accent-primary), var(--accent-secondary));
        color: white;
        border: none;
        border-radius: 10px;
        font-size: 14px;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.2s;
        box-shadow: 0 4px 12px var(--accent-glow);
    }

    .btn-primary:hover:not(:disabled) {
        transform: translateY(-1px);
        filter: brightness(1.1);
    }

    .btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }

    .btn-ghost {
        display: flex;
        align-items: center;
        gap: 6px;
        padding: 8px 14px;
        background: transparent;
        border: 1px dashed var(--border-color);
        color: var(--text-muted);
        border-radius: 8px;
        font-size: 13px;
        cursor: pointer;
        transition: all 0.2s;
    }

    .btn-ghost:hover {
        border-color: var(--accent-primary);
        color: var(--accent-primary);
        background: color-mix(in srgb, var(--accent-primary) 5%, transparent);
    }

    .form-row input:focus,
    .form-row textarea:focus {
        border-color: var(--accent-primary);
        background: var(--surface-hover);
    }

    .headers-textarea {
        width: 100%;
        padding: 10px 14px;
        background: var(--input-bg);
        border: 1px solid var(--border-color);
        border-radius: 10px;
        color: var(--text-primary);
        font-size: 13px;
        outline: none;
        transition: all 0.2s;
        resize: vertical;
        min-height: 80px;
        font-family: var(--font-mono, monospace);
        line-height: 1.5;
    }



    .input-group {
        display: flex;
        align-items: stretch;
        background: var(--input-bg, rgba(255, 255, 255, 0.05));
        border: 1px solid var(--border-color, rgba(255, 255, 255, 0.1));
        border-radius: 12px;
        overflow: hidden;
        transition: all 0.2s ease;
    }

    .input-group:focus-within {
        border-color: var(--accent-primary);
        box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent-primary) 15%, transparent);
    }

    .grouped-input {
        flex: 1;
        background: transparent;
        border: none;
        padding: 12px 14px;
        color: var(--text-primary);
        font-size: 14px;
        outline: none;
        min-width: 0;
    }

    .input-divider {
        width: 1px;
        background: var(--border-color, rgba(255, 255, 255, 0.1));
        margin: 8px 0;
    }

    .select-wrapper {
        position: relative;
        display: flex;
        align-items: center;
        padding-right: 12px;
    }

    .grouped-select {
        background: transparent;
        border: none;
        padding: 0 28px 0 16px;
        color: var(--text-secondary);
        font-size: 13px;
        font-weight: 500;
        outline: none;
        cursor: pointer;
        transition: color 0.2s;
        -webkit-appearance: none;
        appearance: none;
        text-align: left;
        z-index: 1;
    }

    .grouped-select:hover {
        color: var(--text-primary);
    }

    .grouped-select:hover + :global(.select-icon) {
        color: var(--text-primary);
    }

    :global(.select-icon) {
        position: absolute;
        right: 12px;
        pointer-events: none;
        color: var(--text-tertiary);
        transition: color 0.2s;
    }

    /* 针对 select 的 Firefox 样式微调 */
    @-moz-document url-prefix() {
        .grouped-select {
            padding: 0 12px;
        }
    }
</style>
