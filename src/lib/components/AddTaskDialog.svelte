<!--
  AddTaskDialog.svelte
  添加下载任务对话框 - 支持高级设置覆盖层
-->
<script lang="ts">
	import { X, Link, FolderOpen, Download, Settings, Globe, FileText, Shield, Gauge, ArrowLeft, AlertCircle, ChevronRight } from '@lucide/svelte';
	import { open as openDialog } from '@tauri-apps/plugin-dialog';
	import { fade, scale } from 'svelte/transition';
	import type { DownloadConfig } from '$lib/types/download';
	import { createScrollLockEffect, isValidDownloadUrl, validateUrl } from '$lib';

	interface Props {
		open: boolean;
		onClose: () => void;
		onSubmit?: (config: DownloadConfig) => void;
	}

	// 预设 User Agent
	const userAgents = [
		{ id: 'default', name: '默认', value: '' },
		{ id: 'chrome', name: 'Chrome', value: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36' },
		{ id: 'firefox', name: 'Firefox', value: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:120.0) Gecko/20100101 Firefox/120.0' },
		{ id: 'safari', name: 'Safari', value: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Safari/605.1.15' },
		{ id: 'custom', name: '自定义', value: '' }
	];

	let { open, onClose, onSubmit }: Props = $props();

	// 基础设置
	let urls = $state('');
	let savePath = $state('~/Downloads');
	let filename = $state('');

	// 高级设置面板
	let showAdvanced = $state(false);
	let selectedUaId = $state('default');
	let customUserAgent = $state('');
	let referer = $state('');
	let headers = $state('');
	let proxy = $state('');
	let maxDownloadLimitValue = $state('');
	let maxDownloadLimitUnit = $state('M');

	// URL 验证
	let validationError = $state<string>('');
	let validationTimer: ReturnType<typeof setTimeout> | null = null;

	// 计算实际 User Agent
	const effectiveUserAgent = $derived(() => {
		if (selectedUaId === 'custom') return customUserAgent;
		return userAgents.find(ua => ua.id === selectedUaId)?.value || '';
	});

	// 注：URL 验证函数已迁移至 utils/validators.ts

	// 计算是否可以提交
	const canSubmit = $derived(() => {
		const trimmed = urls.trim();
		if (!trimmed) return false;
		return isValidDownloadUrl(trimmed);
	});

	function handleSubmit() {
		// 执行验证
		const error = validateUrl(urls);
		validationError = error;
		
		if (error) {
			return; // 阻止提交
		}
		
		const trimmedUrl = urls.trim();
		const limit = maxDownloadLimitValue.trim() ? `${maxDownloadLimitValue}${maxDownloadLimitUnit}` : '';
		
		onSubmit?.({
			urls: [trimmedUrl],
			savePath,
			filename,
			userAgent: effectiveUserAgent(),
			referer,
			headers,
			proxy,
			maxDownloadLimit: limit
		});
		resetForm();
		onClose();
	}

	function resetForm() {
		urls = '';
		filename = '';
		selectedUaId = 'default';
		customUserAgent = '';
		referer = '';
		headers = '';
		proxy = '';
		maxDownloadLimitValue = '';
		maxDownloadLimitUnit = 'M';
		showAdvanced = false;
		validationError = '';
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
			if (showAdvanced) {
				showAdvanced = false;
			} else {
				onClose();
			}
		}
	}

	// 使用统一的滚动锁定工具
	$effect(() => {
		return createScrollLockEffect(open);
	});

	// 清理定时器（组件卸载时）
	$effect(() => {
		return () => {
			if (validationTimer) {
				clearTimeout(validationTimer);
			}
		};
	});

	// 当URL输入框失去焦点时立即验证
	function handleUrlBlur() {
		// 取消防抖定时器
		if (validationTimer) {
			clearTimeout(validationTimer);
			validationTimer = null;
		}
		
		// 立即验证
		if (urls.trim()) {
			const error = validateUrl(urls);
			validationError = error;
		} else {
			// 如果输入为空，清除错误提示
			validationError = '';
		}
	}

	// 当用户输入时防抖验证
	function handleUrlInput() {
		// 清除之前的定时器
		if (validationTimer) {
			clearTimeout(validationTimer);
		}
		
		// 设置新的防抖定时器 (500ms)
		validationTimer = setTimeout(() => {
			if (urls.trim()) {
				const error = validateUrl(urls);
				validationError = error;
			} else {
				validationError = '';
			}
		}, 500);
	}
</script>

{#if open}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="dialog-overlay" 
		in:fade={{ duration: 150 }} 
		out:fade={{ duration: 100 }}
		onkeydown={handleKeydown}>
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div class="dialog" 
			in:scale={{ duration: 150, start: 0.95, opacity: 0.5 }}
			out:fade={{ duration: 80 }}
			onclick={(e) => e.stopPropagation()}>
			{#if !showAdvanced}
				<!-- 主面板 -->
				<div class="view-main" transition:fade={{ duration: 200 }}>
					<header class="dialog-header">
						<h2>添加下载任务</h2>
						<button class="close-btn" onclick={onClose}>
							<X size={18} />
						</button>
					</header>

					<div class="dialog-body">
						<!-- 下载链接 -->
						<div class="form-group">
							<label for="urls">
								<Link size={14} />
								<span>下载链接</span>
								{#if validationError}
									<span class="error-inline">
										<AlertCircle size={12} />
										{validationError}
									</span>
								{/if}
							</label>
							<textarea
								id="urls"
								placeholder="输入单个下载 URL（支持 http/https/ftp 协议）"
								bind:value={urls}
								oninput={handleUrlInput}
								onblur={handleUrlBlur}
								class:error={!!validationError}
							></textarea>
						</div>

						<!-- 保存位置 -->
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

						<!-- 保存文件名 -->
						<div class="form-group">
							<label>
								<FileText size={14} />
								<span>保存文件名</span>
							</label>
							<input
								type="text"
								class="text-input"
								placeholder="留空则使用原始文件名"
								bind:value={filename}
							/>
						</div>
					</div>

					<footer class="dialog-footer">
						<button class="btn btn-advanced" onclick={() => showAdvanced = true}>
							<Settings size={14} />
							<span>高级设置</span>
						</button>
						<div class="footer-right">
							<button 
								class="btn btn-primary" 
								onclick={handleSubmit}
								disabled={!canSubmit()}
							>
								<Download size={14} />
								<span>开始下载</span>
							</button>
						</div>
					</footer>
				</div>
			{:else}
				<!-- 高级设置面板 (直接渲染，共享背景) -->
				<div class="advanced-panel" transition:fade={{ duration: 200 }}>
					<header class="panel-header">
						<button class="back-btn" onclick={() => showAdvanced = false}>
							<ArrowLeft size={18} />
						</button>
						<div class="breadcrumb">
							<span class="crumb-parent">添加下载任务</span>
							<ChevronRight size={14} class="crumb-sep" />
							<span class="crumb-current">高级设置</span>
						</div>
					</header>

					<div class="panel-body">
						<!-- User Agent -->
						<div class="form-row">
							<label>
								<Globe size={14} />
								<span>User Agent</span>
							</label>
							<div class="ua-selector">
								<select bind:value={selectedUaId}>
									{#each userAgents as ua}
										<option value={ua.id}>{ua.name}</option>
									{/each}
								</select>
								{#if selectedUaId === 'custom'}
									<input
										type="text"
										placeholder="输入自定义 User Agent"
										bind:value={customUserAgent}
									/>
								{/if}
							</div>
						</div>

						<!-- Referer -->
						<div class="form-row">
							<label>
								<Link size={14} />
								<span>Referer</span>
							</label>
							<input
								type="text"
								placeholder="https://example.com"
								bind:value={referer}
							/>
						</div>

						<!-- 自定义 Header -->
						<div class="form-row">
							<label>
								<FileText size={14} />
								<span>自定义 Header</span>
							</label>
							<input
								type="text"
								placeholder="Key: Value (多个用分号分隔)"
								bind:value={headers}
							/>
						</div>

						<!-- 代理服务器 -->
						<div class="form-row">
							<label>
								<Shield size={14} />
								<span>代理服务器</span>
							</label>
							<input
								type="text"
								placeholder="http://host:port 或 socks5://host:port"
								bind:value={proxy}
							/>
						</div>

						<!-- 速度限制 -->
						<div class="form-row">
							<label>
								<Gauge size={14} />
								<span>速度限制</span>
							</label>
							<div class="rate-limit-input">
								<input
									type="number"
									min="0"
									placeholder="0"
									bind:value={maxDownloadLimitValue}
								/>
								<select bind:value={maxDownloadLimitUnit}>
									<option value="M">MB/s</option>
									<option value="K">KB/s</option>
								</select>
							</div>
						</div>
					</div>

					<footer class="panel-footer">
						<button class="btn btn-primary" onclick={() => showAdvanced = false}>
							确定
						</button>
					</footer>
				</div>
			{/if}
		</div>
	</div>
{/if}

<style>
	.dialog-overlay {
		position: fixed;
		inset: 0;
		background: var(--dialog-overlay-bg, rgba(0, 0, 0, 0.5));
		backdrop-filter: blur(8px);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 2000;
	}

	.dialog {
		width: 520px;
		max-width: 90vw;
		background: var(--dialog-bg);
		backdrop-filter: var(--glass-blur) var(--glass-saturate);
		-webkit-backdrop-filter: var(--glass-blur) var(--glass-saturate);
		border: 1px solid var(--glass-border);
		border-radius: 18px;
		overflow: hidden;
		box-shadow: var(--glass-shadow);
		position: relative;
		/* Grid Stack for Transition */
		display: grid;
		grid-template-rows: 1fr;
		grid-template-columns: 1fr;
	}
	
	.view-main, .advanced-panel {
		grid-area: 1 / 1;
		width: 100%;
		display: flex;
		flex-direction: column;
	}

	.dialog-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px 24px;
		border-bottom: 1px solid var(--border-color);
	}

	.dialog-header h2 {
		font-size: 16px;
		font-weight: 500;
		color: var(--text-primary);
		margin: 0;
		letter-spacing: -0.01em;
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
		background: var(--input-bg);
		color: var(--text-primary);
	}

	.dialog-body {
		padding: 24px;
		display: flex;
		flex-direction: column;
		gap: 16px;
		height: 340px;
		overflow-y: auto;
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
		font-weight: 400;
		color: var(--text-secondary);
	}

	/* Inline 错误提示 */
	.error-inline {
		display: flex;
		align-items: center;
		gap: 4px;
		margin-left: auto;
		font-size: 12px;
		color: var(--danger-color);
		font-weight: 400;
	}

	.form-group textarea {
		padding: 12px 14px;
		background: var(--input-bg);
		border: 1px solid var(--border-normal);
		border-radius: 10px;
		color: var(--text-primary);
		font-size: 14px;
		font-weight: 400;
		font-family: inherit;
		outline: none;
		resize: none;
		height: 100px;
		transition: border-color 0s, box-shadow 0.15s ease;
	}

	.form-group textarea:focus {
		border-color: var(--border-strong);
		box-shadow: 0 0 0 3px var(--surface-active);
	}

	.form-group textarea::placeholder {
		color: var(--text-muted);
	}

	/* 错误状态 */
	.form-group textarea.error {
		border-color: var(--danger-color);
	}

	.form-group textarea.error:focus {
		border-color: var(--danger-color);
		box-shadow: 0 0 0 3px rgba(239, 68, 68, 0.15);
	}

	.path-selector {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 14px;
		background: var(--input-bg);
		border: 1px solid var(--border-color);
		border-radius: 10px;
		color: var(--text-secondary);
		font-size: 14px;
		cursor: pointer;
		transition: border-color 0s;
	}

	.path-selector:hover {
		border-color: var(--accent-primary);
	}

	.path-text {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.text-input {
		padding: 12px 14px;
		background: var(--input-bg);
		border: 1px solid var(--border-normal);
		border-radius: 10px;
		color: var(--text-primary);
		font-size: 14px;
		font-weight: 400;
		outline: none;
		transition: border-color 0s, box-shadow 0.15s ease;
	}

	.text-input:focus {
		border-color: var(--border-strong);
		box-shadow: 0 0 0 3px var(--surface-active);
	}

	.text-input::placeholder {
		color: var(--text-muted);
	}

	.dialog-footer {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 16px 24px;
		border-top: 1px solid var(--border-color);
	}

	.footer-right {
		display: flex;
		gap: 10px;
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

	.btn-advanced {
		background: transparent;
		border: 1px dashed var(--border-color);
		color: var(--text-muted);
		padding: 8px 14px;
	}

	.btn-advanced:hover {
		background: var(--input-bg);
		border-color: var(--accent-primary);
		color: var(--accent-text);
	}

	.btn-secondary {
		background: var(--input-bg);
		color: var(--text-secondary);
		font-weight: 400;
	}

	.btn-secondary:hover {
		background: var(--surface-active);
		color: var(--text-primary);
	}

	.btn-primary {
		background: linear-gradient(135deg, var(--accent-primary), var(--accent-secondary));
		color: var(--accent-btn-text, #ffffff);
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



	.advanced-panel {
		display: flex;
		flex-direction: column;
		height: 100%;
	}

	.panel-header {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 16px 24px;
		border-bottom: 1px solid var(--border-color);
	}

	.back-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		background: transparent;
		border: none;
		border-radius: 8px;
		color: var(--text-muted);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.back-btn:hover {
		background: var(--input-bg);
		color: var(--text-primary);
	}

	.breadcrumb {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 14px;
	}
	
	.crumb-parent {
		color: var(--text-secondary);
		font-weight: 400;
	}
	
	/* Global style for lucide icon if needed, or inline style */
	:global(.crumb-sep) {
		color: var(--text-tertiary);
		opacity: 0.7;
	}

	.crumb-current {
		color: var(--text-primary);
		font-weight: 600;
		font-size: 15px;
	}

	.panel-body {
		height: 340px;
		padding: 24px;
		display: flex;
		flex-direction: column;
		gap: 16px;
		overflow-y: auto;
	}

	.form-row {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.form-row label {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 13px;
		color: var(--text-secondary);
	}

	.form-row input,
	.form-row select {
		padding: 10px 12px;
		background: var(--input-bg);
		border: 1px solid var(--border-color);
		border-radius: 8px;
		color: var(--text-primary);
		font-size: 13px;
		outline: none;
		transition: border-color 0s, box-shadow 0.15s ease;
	}

	.form-row input:focus,
	.form-row select:focus {
		border-color: var(--accent-primary);
		box-shadow: 0 0 0 2px var(--accent-active-bg);
	}

	.form-row input::placeholder {
		color: var(--text-muted);
	}

	.form-row select {
		cursor: pointer;
		appearance: none;
		background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%239ca3af' stroke-width='2'%3E%3Cpath d='m6 9 6 6 6-6'/%3E%3C/svg%3E");
		background-repeat: no-repeat;
		background-position: right 12px center;
		padding-right: 32px;
	}

	.ua-selector {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.panel-footer {
		padding: 16px 24px;
		border-top: 1px solid var(--border-color);
		display: flex;
		justify-content: flex-end;
	}

	.rate-limit-input {
		display: flex;
		gap: 8px;
	}

	.rate-limit-input input {
		flex: 1;
	}

	.rate-limit-input select {
		width: 100px;
	}
</style>
