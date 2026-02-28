<!--
  BaseModal.svelte
  统一弹窗基础组件 - 封装背景遮罩、动画、滚动锁定及布局稳定性
-->
<script lang="ts">
	import { fade, scale } from 'svelte/transition';
	import { createScrollLockEffect } from '$lib';
	import { X } from '@lucide/svelte';
    import type { Snippet } from 'svelte';

	interface Props {
		open: boolean;
		onClose: () => void;
		title?: string;
		showClose?: boolean;
		size?: 'sm' | 'md' | 'lg' | 'xl';
		minHeight?: string;
        maxHeight?: string;
        closeOnClickOutside?: boolean;
        closeOnEscape?: boolean;
        children?: Snippet;
        header?: Snippet;
        footer?: Snippet;
        className?: string;
	}

	let { 
        open, 
        onClose, 
        title, 
        showClose = true, 
        size = 'md', 
        minHeight = 'auto',
        maxHeight = '90vh',
        closeOnClickOutside = true,
        closeOnEscape = true,
        children,
        header,
        footer,
        className = ''
    }: Props = $props();

	// 滚动锁定
	$effect(() => {
		return createScrollLockEffect(open);
	});

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape' && open && closeOnEscape) {
			onClose();
		}
	}

    const sizeClasses = {
        sm: 'size-sm',
        md: 'size-md',
        lg: 'size-lg',
        xl: 'size-xl'
    };
</script>

{#if open}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div 
		class="modal-overlay" 
		in:fade={{ duration: 200 }} 
		out:fade={{ duration: 150 }}
		onkeydown={handleKeydown}
        onclick={() => closeOnClickOutside && onClose()}
	>
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div 
			class="modal-container {sizeClasses[size]} {className}" 
			style:--min-height={minHeight}
            style:--max-height={maxHeight}
			in:scale={{ duration: 200, start: 0.96, opacity: 0 }}
			out:fade={{ duration: 100 }}
			onclick={(e) => e.stopPropagation()}
		>
			{#if header || title}
				<header class="modal-header">
					{#if header}
						{@render header()}
					{:else if title}
						<h2 class="modal-title">{title}</h2>
					{/if}
					
					{#if showClose}
						<button class="close-btn" onclick={onClose} aria-label="关闭">
							<X size={18} />
						</button>
					{/if}
				</header>
			{/if}

			<div class="modal-body">
				{#if children}
					{@render children()}
				{/if}
			</div>

			{#if footer}
				<footer class="modal-footer">
					{@render footer()}
				</footer>
			{/if}
		</div>
	</div>
{/if}

<style>
	.modal-overlay {
		position: fixed;
		inset: 0;
		background: var(--dialog-overlay-bg, rgba(0, 0, 0, 0.4));
		backdrop-filter: blur(10px) saturate(115%);
		-webkit-backdrop-filter: blur(10px) saturate(115%);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 2000;
	}

	.modal-container {
		background: color-mix(in srgb, var(--dialog-bg) 96%, transparent);
		backdrop-filter: var(--glass-blur) var(--glass-saturate);
		-webkit-backdrop-filter: var(--glass-blur) var(--glass-saturate);
		border: none;
		border-radius: 18px;
		overflow: hidden;
		box-shadow: var(--glass-shadow), 0 24px 48px -28px rgba(0, 0, 0, 0.55);
		display: flex;
		flex-direction: column;
		width: 90%;
		min-height: var(--min-height);
        max-height: var(--max-height);
		transition: box-shadow 0.3s ease, background-color 0.3s ease;
	}

    /* 尺寸预设 */
    .size-sm { max-width: 400px; }
    .size-md { max-width: 520px; }
    .size-lg { max-width: 640px; }
    .size-xl { max-width: 800px; }

	.modal-header {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		padding: 16px 24px;
		border-bottom: none;
		background: color-mix(in srgb, var(--glass-elevated-bg, var(--dialog-bg)) 76%, transparent);
	}

	.modal-title {
		font-size: 16px;
		font-weight: 600;
		color: var(--text-primary);
		margin: 0;
		letter-spacing: -0.01em;
	}

	.close-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		background: var(--control-bg);
		border: none;
		border-radius: 8px;
		color: var(--text-muted);
		cursor: pointer;
		transition: all 0.15s ease;
        margin-right: -8px;
		flex-shrink: 0;
	}

	.close-btn:hover {
		background: var(--control-bg-hover);
		color: var(--text-primary);
	}

	.close-btn:focus-visible {
		outline: none;
		box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent-primary) 20%, transparent);
	}

	.modal-body {
		flex: 1;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
        
        /* 继承父级容器的滚动条样式（如果有的话） */
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

	.modal-footer {
		padding: 16px 24px;
		border-top: none;
		display: flex;
		justify-content: flex-end;
		gap: 12px;
        background: color-mix(in srgb, var(--glass-elevated-bg, var(--dialog-bg)) 72%, transparent);
	}
</style>
