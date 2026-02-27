<!--
  ClearConfirmDialog.svelte
  清空/删除确认对话框 - 使用 BaseModal 统一管理
-->
<script lang="ts">
	import { Trash2 } from '@lucide/svelte';
	import BaseModal from '../common/BaseModal.svelte';

	interface Props {
		open: boolean;
		title?: string;
		description?: string;
		confirmText?: string;
		onClose: () => void;
		onConfirm: (deleteFile: boolean) => void | Promise<void>;
		showDeleteFileOption?: boolean;
	}

	let {
		open,
		title = '确认清空',
		description = '确定要清空这些任务吗？此操作无法撤销。',
		confirmText = '清空',
		showDeleteFileOption = true,
		onClose,
		onConfirm
	}: Props = $props();

	let deleteFile = $state(false);
	let isSubmitting = $state(false);

	async function handleConfirm() {
		if (isSubmitting) return;
		isSubmitting = true;
		try {
			await onConfirm(deleteFile);
			onClose();
		} finally {
			isSubmitting = false;
		}
	}

	// 监听 open 变化重置状态
	$effect(() => {
		if (open) {
			deleteFile = false;
			isSubmitting = false;
		}
	});
</script>

<BaseModal 
    {open} 
    onClose={onClose} 
    size="sm"
    showClose={true}
>
    {#snippet header()}
        <div class="confirm-header">
            <div class="icon-wrapper">
        <Trash2 size={20} color="var(--semantic-danger)" />
            </div>
            <h3 class="modal-title">{title}</h3>
        </div>
    {/snippet}

    <div class="confirm-body">
        <p class="description">{description}</p>
        
        {#if showDeleteFileOption}
            <label class="checkbox-label">
                <input type="checkbox" bind:checked={deleteFile} />
                <span class="checkbox-text">同时删除本地文件</span>
            </label>
        {/if}
    </div>

	{#snippet footer()}
		<button class="btn-secondary ui-btn-footer ui-btn-secondary ui-btn-focus ui-disabled" onclick={onClose} disabled={isSubmitting}>取消</button>
		<button class="btn-danger ui-btn-footer ui-btn-danger ui-btn-focus ui-disabled" onclick={handleConfirm} disabled={isSubmitting}>
			{confirmText}
		</button>
	{/snippet}
</BaseModal>

<style>
    .confirm-header {
        display: flex;
        align-items: center;
        gap: 12px;
    }

    .icon-wrapper {
        width: 36px;
        height: 36px;
        border-radius: 10px;
        background: color-mix(in srgb, var(--semantic-danger) 10%, transparent);
        display: flex;
        align-items: center;
        justify-content: center;
        flex-shrink: 0;
    }

    .modal-title {
        font-size: 16px;
        font-weight: 600;
        color: var(--text-primary);
        margin: 0;
    }

    .confirm-body {
        padding: 24px;
        display: flex;
        flex-direction: column;
        gap: 20px;
        text-align: left;
    }

    .description {
        color: var(--text-secondary);
        font-size: 14px;
        margin: 0;
        line-height: 1.6;
    }

    .checkbox-label {
        display: inline-flex;
        align-items: center;
        gap: 10px;
        font-size: 14px;
        color: var(--text-primary);
        cursor: pointer;
        padding: 10px 14px;
        background: var(--surface-hover, rgba(255, 255, 255, 0.03));
        border: 1px solid var(--border-color, rgba(255, 255, 255, 0.05));
        border-radius: 10px;
        transition: all 0.2s;
        width: fit-content;
    }

    .checkbox-label:hover {
        background: var(--surface-active, rgba(255, 255, 255, 0.06));
        border-color: var(--border-normal);
    }

    .checkbox-text {
        font-weight: 400;
    }

</style>
