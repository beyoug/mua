<!--
  TaskDetailsModal.svelte
  任务详情弹窗 - 显示任务详细信息
-->
<script lang="ts">
	import { X, File, Link, CheckCircle } from '@lucide/svelte';
	import { fade, scale } from 'svelte/transition';
	import type { DownloadState } from '$lib/types/download';

	interface Props {
		open?: boolean;
		filename: string;
		url: string;
		state: DownloadState;
		savePath?: string;
		onClose: () => void;
	}

	let { open = false, filename, url, state, savePath = '', onClose }: Props = $props();

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			onClose();
		}
	}

	function getStateLabel(s: DownloadState): string {
		switch (s) {
			case 'downloading': return '下载中';
			case 'paused': return '已暂停';
			case 'completed': return '已完成';
			case 'error': return '下载失败';
			case 'waiting': return '等待中';
			case 'cancelled': return '已取消';
			default: return s;
		}
	}

	async function copyToClipboard(text: string) {
		try {
			await navigator.clipboard.writeText(text);
		} catch (e) {
			console.error('Failed to copy', e);
		}
	}
</script>

{#if open}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div 
		class="modal-overlay" 
		in:fade={{ duration: 150 }} 
		out:fade={{ duration: 100 }}
		onkeydown={handleKeydown}
		onclick={onClose}
	>
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div 
			class="modal" 
			in:scale={{ duration: 150, start: 0.95, opacity: 0.5 }}
			out:fade={{ duration: 80 }}
			onclick={(e) => e.stopPropagation()}
		>
			<header class="modal-header">
				<h3>任务详情</h3>
				<button class="close-btn" onclick={onClose} aria-label="关闭">
					<X size={18} />
				</button>
			</header>

			<div class="modal-body">
				<!-- 文件名 -->
				<div class="detail-row">
					<div class="detail-label">
						<File size={14} />
						<span>文件名</span>
					</div>
					<div class="detail-value filename" title={filename}>
						{filename}
					</div>
				</div>

				<!-- URL -->
				<div class="detail-row">
					<div class="detail-label">
						<Link size={14} />
						<span>下载链接</span>
					</div>
					<div class="detail-value url">
						<span class="url-text" title={url}>{url}</span>
						<button class="copy-btn" onclick={() => copyToClipboard(url)} title="复制链接">
							复制
						</button>
					</div>
				</div>

				<!-- 状态 -->
				<div class="detail-row">
					<div class="detail-label">
						<CheckCircle size={14} />
						<span>状态</span>
					</div>
					<div class="detail-value">
						<span class="state-badge state-{state}">{getStateLabel(state)}</span>
					</div>
				</div>

				<!-- 保存路径 -->
				{#if savePath}
					<div class="detail-row">
						<div class="detail-label">
							<File size={14} />
							<span>保存路径</span>
						</div>
						<div class="detail-value path" title={savePath}>
							{savePath}
						</div>
					</div>
				{/if}
			</div>

			<footer class="modal-footer">
				<button class="btn btn-secondary" onclick={onClose}>关闭</button>
			</footer>
		</div>
	</div>
{/if}

<style>
	.modal-overlay {
		position: fixed;
		inset: 0;
		background: var(--dialog-overlay-bg, rgba(0, 0, 0, 0.5));
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}

	.modal {
		width: 90%;
		max-width: 480px;
		background: var(--dialog-bg);
		backdrop-filter: var(--glass-blur) var(--glass-saturate);
		-webkit-backdrop-filter: var(--glass-blur) var(--glass-saturate);
		border: 1px solid var(--glass-border);
		border-radius: 16px;
		box-shadow: var(--glass-shadow), 0 25px 50px -12px rgba(0, 0, 0, 0.25);
		overflow: hidden;
	}

	.modal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px 20px;
		border-bottom: 1px solid var(--border-subtle);
	}

	.modal-header h3 {
		font-size: 15px;
		font-weight: 600;
		color: var(--text-primary);
		margin: 0;
	}

	.close-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		background: transparent;
		border: none;
		border-radius: 6px;
		color: var(--text-muted);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.close-btn:hover {
		background: var(--surface-hover);
		color: var(--text-primary);
	}

	.modal-body {
		padding: 20px;
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.detail-row {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.detail-label {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 12px;
		font-weight: 500;
		color: var(--text-muted);
	}

	.detail-value {
		font-size: 13px;
		color: var(--text-primary);
		word-break: break-all;
	}

	.detail-value.filename {
		font-weight: 500;
	}

	.detail-value.url {
		display: flex;
		align-items: flex-start;
		gap: 8px;
	}

	.url-text {
		flex: 1;
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-secondary);
		line-height: 1.5;
	}

	.copy-btn {
		flex-shrink: 0;
		padding: 4px 10px;
		font-size: 11px;
		font-weight: 500;
		background: var(--surface-hover);
		border: 1px solid var(--border-subtle);
		border-radius: 6px;
		color: var(--text-secondary);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.copy-btn:hover {
		background: var(--surface-active);
		border-color: var(--accent-primary);
		color: var(--text-primary);
	}

	.detail-value.path {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-secondary);
	}

	.state-badge {
		display: inline-flex;
		padding: 4px 10px;
		font-size: 12px;
		font-weight: 500;
		border-radius: 6px;
		background: var(--surface-hover);
		color: var(--text-secondary);
	}

	.state-badge.state-downloading {
		background: var(--accent-subtle);
		color: var(--accent-text);
	}

	.state-badge.state-completed {
		background: var(--semantic-success-bg);
		color: var(--semantic-success);
	}

	.state-badge.state-paused {
		background: var(--semantic-warning-bg);
		color: var(--semantic-warning);
	}

	.state-badge.state-error {
		background: var(--semantic-danger-bg);
		color: var(--semantic-danger);
	}

	.state-badge.state-cancelled {
		background: var(--surface-hover);
		color: var(--text-muted);
	}

	.state-badge.state-waiting {
		background: var(--surface-hover);
		color: var(--text-muted);
	}

	.modal-footer {
		display: flex;
		justify-content: flex-end;
		padding: 16px 20px;
		border-top: 1px solid var(--border-subtle);
	}

	.btn {
		padding: 8px 16px;
		font-size: 13px;
		font-weight: 500;
		border-radius: 8px;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.btn-secondary {
		background: var(--surface-hover);
		border: 1px solid var(--border-subtle);
		color: var(--text-secondary);
	}

	.btn-secondary:hover {
		background: var(--surface-active);
		border-color: var(--border-normal);
		color: var(--text-primary);
	}
</style>
