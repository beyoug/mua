<script lang="ts">
	import { Trash2 } from '@lucide/svelte';

	interface Props {
		open: boolean;
		title?: string;
		description?: string;
		confirmText?: string;
		onClose: () => void;
		onConfirm: (deleteFile: boolean) => void;
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

	function handleConfirm() {
		onConfirm(deleteFile);
		onClose();
	}

	// 监听 open 变化重置状态
	$effect(() => {
		if (open) {
			deleteFile = false;
		}
	});
	$effect(() => {
		if (open) {
			document.body.classList.add('no-scroll');
			document.documentElement.classList.add('no-scroll');
		} else {
			document.body.classList.remove('no-scroll');
			document.documentElement.classList.remove('no-scroll');
		}
		return () => {
			document.body.classList.remove('no-scroll');
			document.documentElement.classList.remove('no-scroll');
		};
	});
</script>

{#if open}
	<div 
		class="dialog-overlay" 
		onclick={onClose}
		role="button"
		tabindex="0"
		onkeydown={(e) => {
			if (e.key === 'Enter' || e.key === ' ') onClose();
		}}
	>
		<div 
			class="dialog" 
			role="dialog"
			aria-modal="true"
			onclick={(e) => e.stopPropagation()}
			onkeydown={(e) => e.stopPropagation()}
			tabindex="-1"
		>
			<div class="dialog-header">
				<div class="icon-wrapper">
					<Trash2 size={24} color="#ef4444" />
				</div>
				<h3>{title}</h3>
			</div>
			
			<div class="dialog-content">
				<p>{description}</p>
				
				{#if showDeleteFileOption}
					<label class="checkbox-label">
						<input type="checkbox" bind:checked={deleteFile} />
						<span>同时删除本地文件</span>
					</label>
				{/if}
			</div>

			<div class="dialog-footer">
				<button class="btn cancel" onclick={onClose}>取消</button>
				<button class="btn confirm" onclick={handleConfirm}>
					{confirmText}
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.dialog-overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.2);
		backdrop-filter: blur(4px);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
		animation: fade-in 0.2s ease;
	}

	.dialog {
		width: 400px;
		background: var(--bg-sidebar);
		backdrop-filter: blur(24px) saturate(180%);
		-webkit-backdrop-filter: blur(24px) saturate(180%);
		border: 1px solid var(--border-color);
		border-radius: 16px;
		box-shadow: 
			0 20px 48px rgba(0, 0, 0, 0.2),
			0 1px 2px rgba(255, 255, 255, 0.1) inset;
		padding: 24px;
		animation: scale-in 0.2s cubic-bezier(0.16, 1, 0.3, 1);
	}

	.dialog-header {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 16px;
		margin-bottom: 8px;
	}

	.icon-wrapper {
		width: 48px;
		height: 48px;
		border-radius: 12px;
		background: rgba(239, 68, 68, 0.1);
		display: flex;
		align-items: center;
		justify-content: center;
	}

	h3 {
		margin: 0;
		font-size: 18px;
		font-weight: 600;
		color: var(--text-primary);
	}

	.dialog-content {
		margin-bottom: 24px;
		text-align: center;
	}

	p {
		color: var(--text-secondary);
		font-size: 14px;
		margin: 0 0 16px;
		line-height: 1.5;
	}

	.checkbox-label {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		font-size: 14px;
		color: var(--text-primary);
		cursor: pointer;
		user-select: none;
		padding: 8px 12px;
		background: var(--bg-hover);
		border-radius: 8px;
		transition: background 0.2s;
	}

	.checkbox-label:hover {
		background: var(--border-light);
	}

	.dialog-footer {
		display: flex;
		gap: 12px;
		justify-content: center;
	}

	.btn {
		padding: 8px 24px;
		border-radius: 8px;
		font-size: 14px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s;
		border: none;
	}

	.btn.cancel {
		background: transparent;
		border: 1px solid var(--border-color);
		color: var(--text-primary);
	}

	.btn.cancel:hover {
		background: var(--bg-hover);
	}

	.btn.confirm {
		background: var(--danger-color, #ef4444);
		color: white;
		box-shadow: 0 2px 8px rgba(239, 68, 68, 0.3);
	}

	.btn.confirm:hover {
		background: #dc2626;
	}

	@keyframes fade-in {
		from { opacity: 0; }
		to { opacity: 1; }
	}

	@keyframes scale-in {
		from { opacity: 0; transform: scale(0.95); }
		to { opacity: 1; transform: scale(1); }
	}
</style>
