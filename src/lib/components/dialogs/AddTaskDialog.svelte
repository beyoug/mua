<!--
  AddTaskDialog.svelte
  添加下载任务对话框 - 使用 BaseModal 统一管理
-->
<script lang="ts">
	import { Link, FolderOpen, Download, Settings, FileText, ArrowLeft, AlertCircle, ChevronRight, FileUp } from '@lucide/svelte';
	import { open as openDialog } from '@tauri-apps/plugin-dialog';
    // @ts-ignore
	import { confirm } from '@tauri-apps/plugin-dialog';
	import { fade, slide } from 'svelte/transition';
	import type { DownloadConfig } from '$lib/types/download';
	import { isValidDownloadUrl, isMagnetUrl } from '$lib';
	import { appSettings, saveAppSettings } from '$lib/stores/settings';
	import BaseModal from '../common/BaseModal.svelte';
	import UaSelector from './UaSelector.svelte';
    import AdvancedSettingsPanel from './AdvancedSettingsPanel.svelte';

	interface Props {
		open: boolean;
		onClose: () => void;
		onSubmit?: (config: DownloadConfig | DownloadConfig[]) => void;
		onTorrentSelect?: (path: string) => void;
	}

	let { open, onClose, onSubmit, onTorrentSelect }: Props = $props();


	// 基础设置 (Normal Tab)
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
    
    // 提交条件：URL 非空且验证通过
	const canSubmitNormal = $derived(!validateInputUrls(urls) && urls.trim().length > 0);
    
    const isCustomUaInvalid = $derived(selectedUaValue === 'custom' && !customUserAgent.trim());

    // 动态链接类型检测：判断是否包含混合类型（普通链接 + Magnet）
    const hasMixedLinks = $derived.by(() => {
        const lines = urls.split('\n').map(l => l.trim()).filter(l => l);
        if (lines.length < 2) return false;
        const hasMagnet = lines.some(l => isMagnetUrl(l));
        const hasNormal = lines.some(l => !isMagnetUrl(l));
        return hasMagnet && hasNormal;
    });
    // 只有非混合链接时才能使用高级设置
    const canUseAdvanced = $derived(!hasMixedLinks);

	let uaSelectorRef = $state<UaSelector>();
	let isSubmitting = $state(false);

    // 验证多行 URL
    function validateInputUrls(input: string): string {
        if (!input.trim()) return ''; // 空时不报错，但 disable button (如果也没 torrent)
        
        const lines = input.split('\n').map(l => l.trim()).filter(l => l);
        if (lines.length === 0) return '';
        
        for (let i = 0; i < lines.length; i++) {
            if (!isValidDownloadUrl(lines[i])) {
                return lines.length > 1 ? `第 ${i + 1} 行链接无效` : '无效的链接格式';
            }
        }
        return '';
    }

	async function handleSubmit() {
		if (isSubmitting) return;
        
        const urlError = validateInputUrls(urls);
		if (urlError || !urls.trim()) {
            validationError = urlError || '请输入下载链接';
			return;
        }

		isSubmitting = true;
		
		try {
            // 混合链接时忽略高级设置，使用默认值
            const useAdvanced = canUseAdvanced;
            const limitStr = useAdvanced ? String(maxDownloadLimitValue || '').trim() : '';
            const limit = limitStr ? `${limitStr}${maxDownloadLimitUnit}` : '';
            const finalUa = useAdvanced ? effectiveUserAgent : '';
            
            const configs: DownloadConfig[] = [];

            // 1. Add URL tasks
            if (urls.trim()) {
                const lines = urls.split('\n').map(l => l.trim()).filter(l => l);
                const isMulti = lines.length > 1;
                
                configs.push(...lines.map(url => ({
                    urls: [url],
                    savePath,
                    filename: isMulti ? '' : filename,
                    userAgent: finalUa,
                    referer: useAdvanced ? referer : '',
                    headers: useAdvanced ? headers : '',
                    proxy: useAdvanced ? proxy : '',
                    maxDownloadLimit: limit
                })));
            }


            if (configs.length > 0) {
			    onSubmit?.(configs);
            }

            // Save UA history
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
        isSelectingFile = false;
	}

    let isSelectingFile = $state(false);

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

    async function selectTorrentFile() {
        if (isSelectingFile) return;
        isSelectingFile = true;
        try {
            const selected = await openDialog({
                multiple: false,
                filters: [{ name: 'Torrent Files', extensions: ['torrent'] }],
                title: '选择种子文件'
            });
            
            if (selected && onTorrentSelect) {
                onTorrentSelect(selected as string);
            }
        } catch (e) {
            console.error('Select torrent failed:', e);
        } finally {
            isSelectingFile = false;
        }
    }

    function openAdvanced() {
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
        
        // 格式化：去除每行首尾空格，去除空行
        if (urls) {
            urls = urls.split('\n')
                .map(l => l.trim())
                .filter(l => l)
                .join('\n');
        }

		if (urls.trim()) {
			validationError = validateInputUrls(urls);
		} else {
			validationError = '';
		}
	}

	function handleUrlInput() {
		if (validationTimer) clearTimeout(validationTimer);
        // 简单防抖
		validationTimer = setTimeout(() => {
			if (urls.trim()) {
				validationError = validateInputUrls(urls);
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
    minHeight="520px"
    showClose={!showAdvanced}
    closeOnClickOutside={false}
    closeOnEscape={false}
>
    {#snippet header()}
        <div class="header-container">
            {#if !showAdvanced}
                <div class="dialog-title">
                    <Link size={16} />
                    <span>添加任务</span>
                </div>
            {:else}
                <div class="advanced-header">
                    <button class="back-link" onclick={handleBack}>
                        <ArrowLeft size={18} />
                    </button>
                    <div class="breadcrumb">
                        <span class="crumb-parent">添加任务</span>
                        <ChevronRight size={14} class="crumb-sep" />
                        <span class="crumb-current">高级设置</span>
                    </div>
                </div>
            {/if}
        </div>
    {/snippet}

    <div class="modal-content-stack">
        {#if !showAdvanced}
            <!-- 主面板 -->
            <div class="view-main" in:fade={{ duration: 150 }}>
                    <div class="dialog-body" in:fade={{ duration: 150 }}>
                        <!-- 下载链接 -->
                        <div class="form-group">
                            <label for="urls">
                                <Link size={14} />
                                <span>下载链接 (支持 Magnet)</span>
                                <div style="margin-left: auto; display: flex; gap: 8px;">
                                    <button 
                                        class="btn-xs-secondary" 
                                        onclick={selectTorrentFile}
                                        disabled={isSelectingFile}
                                    >
                                        {#if isSelectingFile}
                                            <span class="spin">
                                                <AlertCircle size={12} />
                                            </span>
                                            <span>打开中...</span>
                                        {:else}
                                            <FileUp size={12} />
                                            <span>打开种子文件</span>
                                        {/if}
                                    </button>
                                </div>
                            </label>

                             {#if validationError}
                                <span class="error-inline" style="margin-top: 4px;">
                                    <AlertCircle size={12} />
                                    {validationError}
                                </span>
                            {/if}

                            <textarea
                                id="urls"
                                placeholder="输入 HTTP/HTTPS/Magnet 链接，每行一个"
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
                                placeholder="留空则使用默认文件名"
                                bind:value={filename}
                                disabled={false}
                            />
                        </div>
                    </div>
            </div>
        {:else}
            <!-- 高级设置面板 -->
            <div in:fade={{ duration: 150 }}>
                <AdvancedSettingsPanel
                    bind:uaSelectorRef={uaSelectorRef}
                    {selectedUaValue}
                    {customUserAgent}
                    {referer}
                    {headers}
                    {proxy}
                    {maxDownloadLimitValue}
                    {maxDownloadLimitUnit}
                    onUaValueChange={(v) => selectedUaValue = v}
                    onCustomUaChange={(v) => customUserAgent = v}
                    onRefererChange={(v) => referer = v}
                    onHeadersChange={(v) => headers = v}
                    onProxyChange={(v) => proxy = v}
                    onLimitValueChange={(v) => maxDownloadLimitValue = v}
                    onLimitUnitChange={(v) => maxDownloadLimitUnit = v}
                />
            </div>
        {/if}
    </div>

    {#snippet footer()}
        {#if !showAdvanced}
            <div class="footer-layout">
                <div class="advanced-btn-wrapper">
                    <button class="btn-ghost" onclick={openAdvanced} disabled={!canUseAdvanced}>
                        <Settings size={14} />
                        <span>高级设置</span>
                    </button>
                    {#if !canUseAdvanced}
                        <span class="advanced-hint">混合链接不支持自定义设置</span>
                    {/if}
                </div>
                <button 
                    class="btn-primary" 
                    onclick={handleSubmit}
                    disabled={!canSubmitNormal || isSubmitting}
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
    .header-container {
        display: flex;
        align-items: center;
        width: 100%;
    }

    .dialog-title {
        display: flex;
        align-items: center;
        gap: 8px;
        font-size: 15px;
        font-weight: 600;
        color: var(--text-primary);
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
        min-height: 0; /* 防止溢出 */
    }

    .view-main {
        grid-area: 1 / 1;
        padding: 24px;
        display: flex;
        flex-direction: column;
        gap: 20px;
        height: 100%;
    }

    .dialog-body {
        display: flex;
        flex-direction: column;
        gap: 16px;
        flex: 1;
    }

    .form-group {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }



    .form-group label {
        display: flex;
        align-items: center;
        gap: 6px;
        font-size: 13px;
        color: var(--text-secondary);
        width: 100%;
    }

    .error-inline {
        display: flex;
        align-items: center;
        gap: 4px;
        /* margin-left: auto; */
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

    textarea {
        height: 100px;
        resize: none;
        white-space: nowrap;
        overflow-x: auto;
    }
    textarea.error { border-color: var(--danger-color); }

    .advanced-btn-wrapper {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .advanced-hint {
        font-size: 11px;
        color: var(--text-tertiary);
    }

    .btn-ghost:disabled {
        opacity: 0.4;
        cursor: not-allowed;
        pointer-events: none;
    }

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



    .btn-xs-secondary {
        display: flex;
        align-items: center;
        gap: 4px;
        padding: 4px 8px;
        background: var(--bg-tertiary);
        border: 1px solid var(--border-color);
        color: var(--text-secondary);
        border-radius: 6px;
        font-size: 12px;
        cursor: pointer;
        transition: all 0.2s;
    }
    
    .btn-xs-secondary:hover {
        background: var(--bg-hover);
        color: var(--primary-color);
        border-color: var(--primary-color);
    }


</style>
