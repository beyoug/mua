<script lang="ts">
    import { Download, Settings } from '@lucide/svelte';

    interface Props {
        showAdvanced: boolean;
        canUseAdvanced: boolean;
        canSubmitNormal: boolean;
        isSubmitting: boolean;
        isCustomUaInvalid: boolean;
        onOpenAdvanced: () => void;
        onSubmit: () => void;
        onCompleteAdvanced: () => void;
    }

    const {
        showAdvanced,
        canUseAdvanced,
        canSubmitNormal,
        isSubmitting,
        isCustomUaInvalid,
        onOpenAdvanced,
        onSubmit,
        onCompleteAdvanced
    }: Props = $props();
</script>

{#if !showAdvanced}
    <div class="footer-layout">
        <div class="advanced-btn-wrapper">
            <button class="btn-ghost" onclick={onOpenAdvanced} disabled={!canUseAdvanced}>
                <Settings size={14} />
                <span>高级设置</span>
            </button>
            {#if !canUseAdvanced}
                <span class="advanced-hint">混合链接不支持自定义设置</span>
            {/if}
        </div>
        <button class="btn-primary" onclick={onSubmit} disabled={!canSubmitNormal || isSubmitting}>
            {#if isSubmitting}
                <span>提交中...</span>
            {:else}
                <Download size={14} />
                <span>开始下载</span>
            {/if}
        </button>
    </div>
{:else}
    <button class="btn-primary" onclick={onCompleteAdvanced} disabled={isCustomUaInvalid}>
        完成设置
    </button>
{/if}

<style>
    .footer-layout {
        display: flex;
        width: 100%;
        justify-content: space-between;
        align-items: center;
    }

    .advanced-btn-wrapper {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .advanced-hint {
        font-size: 11px;
        color: var(--text-tertiary);
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

    .btn-primary:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

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

    .btn-ghost:disabled {
        opacity: 0.4;
        cursor: not-allowed;
        pointer-events: none;
    }
</style>
