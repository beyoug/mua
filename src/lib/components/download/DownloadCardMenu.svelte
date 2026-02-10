<script lang="ts">
	import { X, Copy, Folder, Info, RefreshCw } from '@lucide/svelte';
	import { scale } from 'svelte/transition';

    interface Props {
        show: boolean;
        downloadState: string;
        url?: string;
        onClose: () => void;
        onCopy: () => void;
        onOpenFolder: () => void;
        onDetails: () => void;
        onCancelOrDelete: () => void;
        onRedownload?: () => void;
    }

    let { 
        show, 
        downloadState, 
        url, 
        onClose, 
        onCopy, 
        onOpenFolder, 
        onDetails, 
        onCancelOrDelete,
        onRedownload
    }: Props = $props();
    let menuRef = $state<HTMLElement | null>(null);
    let isPopup = $state(false);

    $effect(() => {
        if (show && menuRef) {
            const rect = menuRef.getBoundingClientRect();
            const viewportHeight = window.innerHeight;
            // 如果底部超出屏幕，则向上弹出
            // 这里我们预判一下，如果当前是 top 定位，bottom 会是多少
            // 但因为已经是 mounted 了，直接看 rect.bottom 即可
            // 为了避免闪烁，最理想是 visibility: hidden 先算，但 svelte transition 可能冲突
            // 实测：直接判断 rect.bottom > viewportHeight 即可，虽然可能有一帧的 layout shift，但一般很快
            if (rect.bottom > viewportHeight - 20) {
                isPopup = true;
            }
        } else {
            // 关闭时重置，保证下次打开默认是向下（除非立刻又打开）
            // 注意：不要在 show 为 true 时重置为 false，否则会死循环（如果上面判断变成 false）
            // 但我们在 show=false 时重置是安全的
            if (!show) isPopup = false;
        }
    });
</script>

{#if show}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="menu-backdrop" onclick={onClose}></div>
    
    <div 
        class="dropdown-menu" 
        class:pop-up={isPopup}
        bind:this={menuRef}
        transition:scale={{ duration: 150, start: 0.95 }}
    >
        {#if ['complete', 'removed', 'error', 'missing'].includes(downloadState)}
            <button class="menu-item" onclick={() => { onRedownload?.(); onClose(); }}>
                <RefreshCw size={14} />
                <span>重新下载</span>
            </button>
        {/if}
        <button class="menu-item" onclick={onCopy} disabled={!url}>
            <Copy size={14} />
            <span>复制链接</span>
        </button>
        {#if ['active', 'paused', 'complete'].includes(downloadState)}
            <button class="menu-item" onclick={onOpenFolder}>
                <Folder size={14} />
                <span>打开文件夹</span>
            </button>
        {/if}
        <button class="menu-item" onclick={onDetails}>
            <Info size={14} />
            <span>查看详情</span>
        </button>
        <div class="menu-divider"></div>
        <button class="menu-item danger" onclick={onCancelOrDelete}>
            <X size={14} />
            <span>{['complete', 'removed', 'error'].includes(downloadState) ? '删除任务' : '取消下载'}</span>
        </button>
    </div>
{/if}

<style>
	.menu-backdrop {
		position: fixed;
		inset: 0;
		z-index: 90;
		cursor: default;
	}

	.dropdown-menu {
		position: absolute;
		top: calc(100% + 4px);
		right: 0;
		width: 140px; /* 稍微窄一点 */
		background: var(--glass-menu-bg); /* 65% 透明度 */
		backdrop-filter: blur(20px);
		-webkit-backdrop-filter: blur(20px);
		border: 1px solid var(--border-strong);
		border-radius: 12px;
		box-shadow: 
			0 4px 6px -1px rgba(0, 0, 0, 0.1),
			0 10px 15px -3px rgba(0, 0, 0, 0.1),
			0 0 0 1px rgba(255, 255, 255, 0.05); /* 更加立体的阴影 */
		padding: 4px;
		display: flex;
		flex-direction: column;
		gap: 2px;
		z-index: 100;
		transform-origin: top right;
	}

    .dropdown-menu.pop-up {
        top: auto;
        bottom: calc(100% + 4px);
        transform-origin: bottom right;
    }

	.menu-item {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 8px 10px;
		background: transparent;
		border: none;
		border-radius: 8px;
		color: var(--text-primary);
		font-size: 13px;
		cursor: pointer;
		text-align: left;
		transition: all 0.15s ease;
	}

	.menu-item:hover {
		background: var(--surface-active);
		color: var(--accent-primary);
	}

	.menu-item:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.menu-item.danger {
		color: var(--danger-color);
	}

	.menu-item.danger:hover {
		background: rgba(239, 68, 68, 0.1);
	}

	.menu-divider {
		height: 1px;
		background: var(--border-subtle);
		margin: 4px 0;
	}
</style>
