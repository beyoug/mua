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
            <button class="btn-ghost ui-btn-footer ui-btn-secondary ui-btn-focus ui-disabled" onclick={onOpenAdvanced} disabled={!canUseAdvanced}>
                <Settings size={14} />
                <span>高级设置</span>
            </button>
            {#if !canUseAdvanced}
                <span class="advanced-hint">混合链接不支持自定义设置</span>
            {/if}
        </div>
        <button class="btn-primary ui-btn-footer ui-btn-primary ui-btn-focus ui-disabled" onclick={onSubmit} disabled={!canSubmitNormal || isSubmitting}>
            {#if isSubmitting}
                <span>提交中...</span>
            {:else}
                <Download size={14} />
                <span>开始下载</span>
            {/if}
        </button>
    </div>
{:else}
    <button class="btn-primary ui-btn-footer ui-btn-primary ui-btn-focus ui-disabled" onclick={onCompleteAdvanced} disabled={isCustomUaInvalid}>
        完成设置
    </button>
{/if}

<style>
    .footer-layout {
        display: flex;
        width: 100%;
        justify-content: space-between;
        align-items: center;
        gap: 12px;
        padding-top: 2px;
    }

    .advanced-btn-wrapper {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .advanced-hint {
        font-size: 11px;
        color: color-mix(in srgb, var(--text-tertiary) 92%, transparent);
        padding-left: 2px;
        line-height: 1.3;
    }

    .btn-primary {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 6px;
        min-width: 126px;
        min-height: 36px;
        font-weight: 600;
        box-shadow:
            0 8px 16px -14px color-mix(in srgb, var(--accent-glow) 34%, transparent),
            inset 0 1px 0 color-mix(in srgb, #ffffff 22%, transparent);
    }

    .btn-primary:hover {
        transform: none;
        filter: none;
        box-shadow:
            var(--hover-ring-medium),
            0 8px 16px -14px color-mix(in srgb, var(--accent-glow) 28%, transparent);
    }

    .btn-primary:active {
        transform: translateY(0);
        filter: brightness(0.99);
    }

    .btn-ghost {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 6px;
        min-width: 126px;
        border: 1px dashed color-mix(in srgb, var(--accent-primary) 26%, transparent);
        background: transparent;
        color: var(--text-secondary);
        font-size: 13px;
        min-height: 36px;
        box-shadow: none;
    }

    .btn-ghost:hover {
        background: color-mix(in srgb, var(--accent-primary) 9%, transparent);
        border-color: color-mix(in srgb, var(--accent-primary) 38%, transparent);
        color: var(--text-primary);
        box-shadow:
            var(--hover-ring-medium),
            0 8px 16px -14px color-mix(in srgb, var(--accent-glow) 24%, transparent);
        transform: none;
    }

    .btn-ghost:active {
        transform: translateY(0);
    }

    .btn-primary:disabled,
    .btn-ghost:disabled {
        transform: none;
        filter: none;
    }
</style>
