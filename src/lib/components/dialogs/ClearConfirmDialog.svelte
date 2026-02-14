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
                <Trash2 size={20} color="var(--danger-soft-text)" />
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
		<button class="btn-secondary" onclick={onClose} disabled={isSubmitting}>取消</button>
		<button class="btn-danger" onclick={handleConfirm} disabled={isSubmitting}>
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
        background: var(--danger-icon-soft-bg);
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
        background: var(--control-bg);
        border: 1px solid var(--control-border);
        border-radius: 10px;
        transition: all 0.2s;
        width: fit-content;
    }

    .checkbox-label:hover {
        background: var(--control-bg-hover);
        border-color: var(--control-border-hover);
    }

    .checkbox-text {
        font-weight: 400;
    }

	.btn-secondary {
        padding: 8px 18px;
        border-radius: 8px;
        font-size: 13px;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.2s;
        border: 1px solid var(--border-color);
        background: transparent;
        color: var(--text-primary);
	}

	.btn-secondary:disabled,
	.btn-danger:disabled {
		opacity: 0.6;
		cursor: not-allowed;
		transform: none;
	}

    .btn-secondary:hover {
        background: var(--control-bg-hover);
    }

    .btn-danger {
        padding: 8px 18px;
        border-radius: 8px;
        font-size: 13px;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.2s;
        border: none;
        background: var(--danger-action-bg);
        color: var(--accent-btn-text);
        box-shadow: var(--danger-action-shadow);
    }

    .btn-danger:hover {
        background: var(--danger-action-bg-hover);
        transform: translateY(-1px);
        box-shadow: var(--danger-action-shadow-hover);
    }
</style>
