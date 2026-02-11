<!--
  AddTaskDialog.svelte
  添加下载任务对话框 - 使用 BaseModal 统一管理
-->
<script lang="ts">
	import { Link, ArrowLeft, ChevronRight } from '@lucide/svelte';
	import { fade } from 'svelte/transition';
	import type { DownloadConfig } from '$lib/types/download';
	import BaseModal from '../common/BaseModal.svelte';
	import UaSelector from './UaSelector.svelte';
    import AdvancedSettingsPanel from './AdvancedSettingsPanel.svelte';
    import BasicForm from './add-task/BasicForm.svelte';
    import FooterActions from './add-task/FooterActions.svelte';
    import { useAddTaskDialog } from './add-task/useAddTaskDialog.svelte';

	interface Props {
		open: boolean;
		onClose: () => void;
		onSubmit?: (config: DownloadConfig | DownloadConfig[]) => void | Promise<void>;
		onTorrentSelect?: (path: string) => void;
	}

	let { open, onClose, onSubmit, onTorrentSelect }: Props = $props();
	let uaSelectorRef = $state<UaSelector | undefined>(undefined);
	const controller = useAddTaskDialog({
		onClose: () => onClose(),
		onSubmit: (configs) => onSubmit?.(configs),
		onTorrentSelect: (path) => onTorrentSelect?.(path)
	});
</script>

<BaseModal 
    {open} 
    onClose={onClose} 
    size="md" 
    minHeight="520px"
    showClose={!controller.showAdvanced}
    closeOnClickOutside={false}
    closeOnEscape={false}
>
    {#snippet header()}
        <div class="header-container">
            {#if !controller.showAdvanced}
                <div class="dialog-title">
                    <Link size={16} />
                    <span>添加任务</span>
                </div>
            {:else}
                <div class="advanced-header">
                    <button class="back-link" onclick={controller.handleBack}>
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
        {#if !controller.showAdvanced}
            <div class="view-main" in:fade={{ duration: 150 }}>
                <BasicForm
                    urls={controller.urls}
                    filename={controller.filename}
                    savePath={controller.savePath}
                    validationError={controller.validationError}
                    isSelectingFile={controller.isSelectingFile}
                    onUrlsChange={controller.setUrls}
                    onFilenameChange={controller.setFilename}
                    onUrlInput={controller.handleUrlInput}
                    onUrlBlur={controller.handleUrlBlur}
                    onSelectFolder={controller.selectFolder}
                    onSelectTorrentFile={controller.selectTorrentFile}
                />
            </div>
        {:else}
            <div in:fade={{ duration: 150 }}>
                <AdvancedSettingsPanel
                    bind:uaSelectorRef={uaSelectorRef}
                    selectedUaValue={controller.selectedUaValue}
                    customUserAgent={controller.customUserAgent}
                    referer={controller.referer}
                    headers={controller.headers}
                    proxy={controller.proxy}
                    maxDownloadLimitValue={controller.maxDownloadLimitValue}
                    maxDownloadLimitUnit={controller.maxDownloadLimitUnit}
                    onUaValueChange={controller.setSelectedUaValue}
                    onCustomUaChange={controller.setCustomUserAgent}
                    onRefererChange={controller.setReferer}
                    onHeadersChange={controller.setHeaders}
                    onProxyChange={controller.setProxy}
                    onLimitValueChange={controller.setMaxDownloadLimitValue}
                    onLimitUnitChange={controller.setMaxDownloadLimitUnit}
                />
            </div>
        {/if}
    </div>

    {#snippet footer()}
        <FooterActions
            showAdvanced={controller.showAdvanced}
            canUseAdvanced={controller.canUseAdvanced}
            canSubmitNormal={controller.canSubmitNormal}
            isSubmitting={controller.isSubmitting}
            isCustomUaInvalid={controller.isCustomUaInvalid}
            onOpenAdvanced={controller.openAdvanced}
            onSubmit={() => controller.handleSubmit(uaSelectorRef)}
            onCompleteAdvanced={controller.completeAdvanced}
        />
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
</style>
