<!--
  AddTaskDialog.svelte
  添加下载任务对话框 - 支持高级设置覆盖层
-->
<script lang="ts">
	import { X, Link, FolderOpen, Download, Settings, Globe, FileText, Shield, Gauge, ArrowLeft } from '@lucide/svelte';
	import { open as openDialog } from '@tauri-apps/plugin-dialog';
	import { fade, fly } from 'svelte/transition';

	interface Props {
		open: boolean;
		onClose: () => void;
		onSubmit?: (config: DownloadConfig) => void;
	}

	interface DownloadConfig {
		urls: string[];
		savePath: string;
		filename: string;
		userAgent: string;
		referer: string;
		headers: string;
		proxy: string;
		maxDownloadLimit: string;
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

	// 计算实际 User Agent
	const effectiveUserAgent = $derived(() => {
		if (selectedUaId === 'custom') return customUserAgent;
		return userAgents.find(ua => ua.id === selectedUaId)?.value || '';
	});

	function handleSubmit() {
		const urlList = urls.split('\n').map(u => u.trim()).filter(u => u.length > 0);
		if (urlList.length > 0) {
			const limit = maxDownloadLimitValue.trim() ? `${maxDownloadLimitValue}${maxDownloadLimitUnit}` : '';
			
			onSubmit?.({
				urls: urlList,
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
</script>

{#if open}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="dialog-overlay" onclick={onClose} onkeydown={handleKeydown}>
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div class="dialog" onclick={(e) => e.stopPropagation()}>
			<!-- 主面板 -->
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
					</label>
					<textarea
						id="urls"
						placeholder="输入下载 URL，每行一个..."
						bind:value={urls}
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
					<button class="btn btn-secondary" onclick={onClose}>取消</button>
					<button 
						class="btn btn-primary" 
						onclick={handleSubmit}
						disabled={!urls.trim()}
					>
						<Download size={14} />
						<span>开始下载</span>
					</button>
				</div>
			</footer>

			<!-- 高级设置覆盖层 -->
			{#if showAdvanced}
				<div class="advanced-overlay" transition:fade={{ duration: 150 }}>
					<div class="advanced-panel" transition:fly={{ y: 20, duration: 200 }}>
						<header class="panel-header">
							<button class="back-btn" onclick={() => showAdvanced = false}>
								<ArrowLeft size={18} />
							</button>
							<h3>高级设置</h3>
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
				</div>
			{/if}
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
		position: relative;
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
		gap: 16px;
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
		resize: none;
		height: 100px;
		transition: border-color 0s, box-shadow 0.15s ease;
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
		background: var(--border-light);
		border: 1px solid var(--border-color);
		border-radius: 10px;
		color: var(--text-primary);
		font-size: 14px;
		outline: none;
		transition: border-color 0s, box-shadow 0.15s ease;
	}

	.text-input:focus {
		border-color: var(--accent-primary);
		box-shadow: 0 0 0 3px var(--accent-active-bg);
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
		background: var(--border-light);
		border-color: var(--accent-primary);
		color: var(--accent-text);
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

	/* 高级设置覆盖层 */
	.advanced-overlay {
		position: absolute;
		inset: 0;
		/* 使用不透明背景确保完全遮挡 */
		background: #0f0f14;
		border-radius: 20px;
		display: flex;
		flex-direction: column;
		z-index: 10;
	}

	:global(html.light) .advanced-overlay {
		background: #f8fafc;
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
		width: 32px;
		height: 32px;
		background: transparent;
		border: none;
		border-radius: 8px;
		color: var(--text-muted);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.back-btn:hover {
		background: var(--border-light);
		color: var(--text-primary);
	}

	.panel-header h3 {
		font-size: 17px;
		font-weight: 600;
		color: var(--text-primary);
		margin: 0;
	}

	.panel-body {
		flex: 1;
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
		background: var(--border-light);
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
