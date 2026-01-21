<!--
  AddTaskDialog.svelte
  添加下载任务对话框
-->
<script lang="ts">
	import { X, Link, FolderOpen, Download } from '@lucide/svelte';
	import { open as openDialog } from '@tauri-apps/plugin-dialog';

	interface Props {
		open: boolean;
		onClose: () => void;
		onSubmit?: (urls: string[], savePath: string) => void;
	}

	let { open, onClose, onSubmit }: Props = $props();

	let urls = $state('');
	let savePath = $state('~/Downloads');

	function handleSubmit() {
		const urlList = urls.split('\n').map(u => u.trim()).filter(u => u.length > 0);
		if (urlList.length > 0) {
			onSubmit?.(urlList, savePath);
			urls = '';
			onClose();
		}
	}

	async function selectFolder() {
		try {
			const selected = await openDialog({
				directory: true,
				multiple: false,
				title: '选择下载目录'
			});
			if (selected) {
				savePath = selected as string;
			}
		} catch (e) {
			// 非 Tauri 环境或用户取消
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			onClose();
		}
	}
</script>

{#if open}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="dialog-overlay" onclick={onClose} onkeydown={handleKeydown}>
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div class="dialog" onclick={(e) => e.stopPropagation()}>
			<header class="dialog-header">
				<h2>添加下载任务</h2>
				<button class="close-btn" onclick={onClose}>
					<X size={18} />
				</button>
			</header>

			<div class="dialog-body">
				<div class="form-group">
					<label for="urls">
						<Link size={14} />
						<span>下载链接</span>
					</label>
					<textarea
						id="urls"
						placeholder="输入下载 URL..."
						bind:value={urls}
						rows="5"
					></textarea>
				</div>

				<div class="form-group">
					<label>
						<FolderOpen size={14} />
						<span>保存位置</span>
					</label>
					<button class="path-selector" onclick={selectFolder}>
						<span class="path-text">{savePath}</span>
						<FolderOpen size={14} />
					</button>
				</div>
			</div>

			<footer class="dialog-footer">
				<button class="btn btn-secondary" onclick={onClose}>取消</button>
				<button 
					class="btn btn-primary" 
					onclick={handleSubmit}
					disabled={!urls.trim()}
				>
					<Download size={14} />
					<span>开始下载</span>
				</button>
			</footer>
		</div>
	</div>
{/if}

<style>
	.dialog-overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.6);
		backdrop-filter: blur(4px);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 2000;
	}

	.dialog {
		width: 560px;
		max-width: 90vw;
		background: var(--bg-sidebar);
		backdrop-filter: blur(24px) saturate(180%);
		-webkit-backdrop-filter: blur(24px) saturate(180%);
		border: 1px solid var(--border-color);
		border-radius: 20px;
		overflow: hidden;
		box-shadow: 
			0 24px 48px rgba(0, 0, 0, 0.2),
			0 1px 2px rgba(255, 255, 255, 0.1) inset;
		animation: dialog-appear 0.25s cubic-bezier(0.25, 0.46, 0.45, 0.94);
	}

	@keyframes dialog-appear {
		from {
			opacity: 0;
			transform: scale(0.95) translateY(10px);
		}
		to {
			opacity: 1;
			transform: scale(1) translateY(0);
		}
	}

	.dialog-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px 24px;
		border-bottom: 1px solid var(--border-color);
	}

	.dialog-header h2 {
		font-size: 17px;
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
		background: var(--border-light);
		color: var(--text-primary);
	}

	.dialog-body {
		padding: 24px;
		display: flex;
		flex-direction: column;
		gap: 20px;
	}

	.form-group {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.form-group label {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 13px;
		color: var(--text-secondary);
	}

	.form-group textarea {
		padding: 12px 14px;
		background: var(--border-light);
		border: 1px solid var(--border-color);
		border-radius: 10px;
		color: var(--text-primary);
		font-size: 14px;
		font-family: inherit;
		outline: none;
		resize: vertical;
		min-height: 100px;
		transition: all 0.15s ease;
	}

	.form-group textarea:focus {
		border-color: var(--accent-primary);
		box-shadow: 0 0 0 3px var(--accent-active-bg);
	}

	.form-group textarea::placeholder {
		color: var(--text-muted);
	}

	.path-selector {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 14px;
		background: var(--border-light);
		border: 1px solid var(--border-color);
		border-radius: 10px;
		color: var(--text-secondary);
		font-size: 14px;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.path-selector:hover {
		border-color: var(--accent-primary);
	}

	.path-text {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.dialog-footer {
		display: flex;
		justify-content: flex-end;
		gap: 10px;
		padding: 16px 24px;
		border-top: 1px solid var(--border-color);
	}

	.btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 10px 18px;
		border: none;
		border-radius: 8px;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.btn-secondary {
		background: var(--border-light);
		color: var(--text-secondary);
	}

	.btn-secondary:hover {
		background: var(--border-color);
		color: var(--text-primary);
	}

	.btn-primary {
		background: linear-gradient(135deg, var(--accent-primary), var(--accent-secondary));
		color: white;
		box-shadow: 0 2px 8px var(--accent-glow);
	}

	.btn-primary:hover:not(:disabled) {
		transform: translateY(-1px);
		box-shadow: 0 4px 12px var(--accent-glow);
	}

	.btn-primary:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
</style>

