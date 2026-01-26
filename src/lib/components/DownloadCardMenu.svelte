<script lang="ts">
	import { X, Copy, Folder, Info } from '@lucide/svelte';
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
    }

    let { 
        show, 
        downloadState, 
        url, 
        onClose, 
        onCopy, 
        onOpenFolder, 
        onDetails, 
        onCancelOrDelete 
    }: Props = $props();
</script>

{#if show}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="menu-backdrop" onclick={onClose}></div>
    
    <div 
        class="dropdown-menu" 
        transition:scale={{ duration: 150, start: 0.95 }}
    >
        <button class="menu-item" onclick={onCopy} disabled={!url}>
            <Copy size={14} />
            <span>复制链接</span>
        </button>
        <button class="menu-item" onclick={onOpenFolder}>
            <Folder size={14} />
            <span>打开文件夹</span>
        </button>
        <button class="menu-item" onclick={onDetails}>
            <Info size={14} />
            <span>查看详情</span>
        </button>
        <div class="menu-divider"></div>
        <button class="menu-item danger" onclick={onCancelOrDelete}>
            <X size={14} />
            <span>{['completed', 'cancelled', 'error'].includes(downloadState) ? '删除任务' : '取消下载'}</span>
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
