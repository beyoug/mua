<!--
  SettingsPanel.svelte
  浮动设置面板 - 容器组件
-->
<script lang="ts">
	import { X } from '@lucide/svelte';
    import { fade, scale } from 'svelte/transition';
    import { loadAria2Config } from '$lib/stores/aria2Config';
    import { loadAppSettings } from '$lib/stores/settings';
	import { createScrollLockEffect } from '$lib';
    import BasicSettings from './BasicSettings.svelte';
    import AdvancedSettings from './AdvancedSettings.svelte';

	interface Props {
		open: boolean;
		onClose: () => void;
	}

	let { open, onClose }: Props = $props();
    let activeTab: 'basic' | 'advanced' = $state('basic');

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			onClose();
		}
	}

	// Load config when panel opens
	$effect(() => {
		if (open) {
			loadAria2Config();
            loadAppSettings();
		}
	});

	// 使用统一的滚动锁定工具
	$effect(() => {
		return createScrollLockEffect(open);
	});
</script>

{#if open}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="panel-overlay" 
		in:fade={{ duration: 150 }} 
		out:fade={{ duration: 100 }}
		onclick={onClose} 
		onkeydown={handleKeydown}>
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div class="panel" 
			in:scale={{ duration: 150, start: 0.98, opacity: 0.5 }}
			out:fade={{ duration: 80 }}
			onclick={(e) => e.stopPropagation()}>
			<header class="panel-header">
				<h2>设置</h2>
				<button class="close-btn" onclick={onClose}>
					<X size={18} />
				</button>
			</header>

			<div class="panel-body">
				<div class="tabs">
					<button 
						class="tab-btn" 
						class:active={activeTab === 'basic'} 
						onclick={() => activeTab = 'basic'}
					>
						基本设置
					</button>
					<button 
						class="tab-btn" 
						class:active={activeTab === 'advanced'} 
						onclick={() => activeTab = 'advanced'}
					>
						高级配置
					</button>
				</div>

				{#if activeTab === 'basic'}
                    <BasicSettings />
				{/if}

				{#if activeTab === 'advanced'}
					<AdvancedSettings />
				{/if}
			</div>
		</div>
	</div>
{/if}

<style>
	.panel-overlay {
		position: fixed;
		inset: 0;
		background: var(--dialog-overlay-bg, rgba(0, 0, 0, 0.2));
		backdrop-filter: blur(2px);
		z-index: 2000;
	}

	.panel {
		position: fixed;
		/* 与任务列表区域对齐：Sidebar (200px) + 左边距 (12px) + 间距 (12px) = 224px */
		left: 224px;
		top: 12px;
		right: 12px;
		bottom: 12px;
		background: var(--dialog-bg);
		backdrop-filter: var(--glass-blur) var(--glass-saturate);
		-webkit-backdrop-filter: var(--glass-blur) var(--glass-saturate);
		border: 1px solid var(--glass-border);
		border-radius: 16px;
		box-shadow: var(--glass-shadow);
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.panel-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 14px 18px;
		border-bottom: 1px solid var(--border-color);
	}

	.panel-header h2 {
		font-size: 15px;
		font-weight: 500;
		color: var(--text-primary);
		margin: 0;
	}

	.close-btn {
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
		transition: all 0.15s ease;
	}

	.close-btn:hover {
		background: var(--input-bg);
		color: var(--text-primary);
	}

	.panel-body {
		flex: 1;
		padding: 16px 18px;
		overflow-y: auto;
	}

    .tabs {
        display: flex;
        gap: 8px;
        margin-bottom: 24px;
        background: var(--input-bg);
        padding: 4px;
        border-radius: 10px;
        border: 1px solid var(--border-normal);
    }

    .tab-btn {
        flex: 1;
        padding: 6px;
        font-size: 13px;
        color: var(--text-secondary);
        background: transparent;
        border: none;
        border-radius: 8px;
        cursor: pointer;
        transition: all 0.2s ease;
        font-weight: 500;
    }

    .tab-btn:hover {
        color: var(--text-primary);
    }

    .tab-btn.active {
        background: var(--dialog-bg);
        color: var(--accent-primary);
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
        font-weight: 600;
    }
</style>
