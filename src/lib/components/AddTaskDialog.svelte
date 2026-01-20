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
		background: #18181b;
		border: 1px solid rgba(255, 255, 255, 0.1);
		border-radius: 16px;
		overflow: hidden;
	}

	.dialog-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px 24px;
		border-bottom: 1px solid rgba(255, 255, 255, 0.08);
	}

	.dialog-header h2 {
		font-size: 17px;
		font-weight: 600;
		color: rgba(255, 255, 255, 0.95);
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
		color: rgba(255, 255, 255, 0.5);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.close-btn:hover {
		background: rgba(255, 255, 255, 0.1);
		color: rgba(255, 255, 255, 0.9);
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
		color: rgba(255, 255, 255, 0.6);
	}

	.form-group label .hint {
		margin-left: auto;
		font-size: 12px;
		color: rgba(255, 255, 255, 0.35);
	}

	.form-group textarea {
		padding: 12px 14px;
		background: rgba(255, 255, 255, 0.05);
		border: 1px solid rgba(255, 255, 255, 0.1);
		border-radius: 10px;
		color: rgba(255, 255, 255, 0.95);
		font-size: 14px;
		font-family: inherit;
		outline: none;
		resize: vertical;
		min-height: 100px;
		transition: all 0.15s ease;
	}

	.form-group textarea:focus {
		border-color: rgb(16, 185, 129);
		background: rgba(255, 255, 255, 0.08);
	}

	.form-group textarea::placeholder {
		color: rgba(255, 255, 255, 0.3);
	}

	.path-selector {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 14px;
		background: rgba(255, 255, 255, 0.05);
		border: 1px solid rgba(255, 255, 255, 0.1);
		border-radius: 10px;
		color: rgba(255, 255, 255, 0.7);
		font-size: 14px;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.path-selector:hover {
		border-color: rgba(255, 255, 255, 0.2);
		background: rgba(255, 255, 255, 0.08);
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
		border-top: 1px solid rgba(255, 255, 255, 0.08);
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
		background: rgba(255, 255, 255, 0.1);
		color: rgba(255, 255, 255, 0.7);
	}

	.btn-secondary:hover {
		background: rgba(255, 255, 255, 0.15);
		color: rgba(255, 255, 255, 0.9);
	}

	.btn-primary {
		background: linear-gradient(135deg, rgb(16, 185, 129), rgb(20, 184, 166));
		color: white;
	}

	.btn-primary:hover:not(:disabled) {
		background: linear-gradient(135deg, rgb(5, 150, 105), rgb(13, 148, 136));
	}

	.btn-primary:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
</style>

