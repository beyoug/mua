<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';
	import { onMount } from 'svelte';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { currentTheme, effectiveColorMode, particlesEnabled } from '$lib/stores/theme';
	import ParticleBackground from '$lib/components/effects/ParticleBackground.svelte';
	import { initNotifications, cleanupNotifications } from '$lib/services/notifications';

	import { listen } from '@tauri-apps/api/event';
	import { message } from '@tauri-apps/plugin-dialog';

	let { children } = $props();

	// 订阅主题和颜色模式变化，动态更新 html 类
	$effect(() => {
		const themeId = $currentTheme;
		const mode = $effectiveColorMode;
		const classes = [`theme-${themeId}`];
		if (mode === 'light') {
			classes.push('light');
		}
		document.documentElement.className = classes.join(' ');
		
		// 动态设置 color-scheme
		document.documentElement.style.colorScheme = mode;
	});

	onMount(() => {
		// 禁用右键菜单
		document.addEventListener('contextmenu', (e) => e.preventDefault());

		let unlisten_sidecar: (() => void) | undefined;

		const init = async () => {
			try {
				const appWindow = getCurrentWindow();
				await appWindow.show();
				await appWindow.setFocus();

				// 初始化通知服务
				await initNotifications();

				// 监听 Aria2 Sidecar 错误
				unlisten_sidecar = await listen('aria2-sidecar-error', async (event: any) => {
					const payload = event.payload;
					console.error('Aria2 Sidecar Error:', payload);
					// 简单的防抖或限流可以在这里做，但目前直接弹窗
					await message(
						`Aria2 Service Error: ${payload.message}\n\nCode: ${payload.code}\nSignal: ${payload.signal}\n\nLog:\n${payload.stderr}`,
						{
							title: 'Aria2 Sidecar Error',
							kind: 'error'
						}
					);
				});
			} catch (e) {
				// 非 Tauri 环境忽略
				console.warn('Non-Tauri environment or cleanup error', e);
			}
		};

		init();

		return () => {
			if (unlisten_sidecar) unlisten_sidecar();
			cleanupNotifications();
		};
	});
</script>
<svelte:head>
	<link rel="icon" href={favicon} />
	<title>Mua - Download Manager</title>
</svelte:head>

{#if $particlesEnabled}
	<ParticleBackground />
{/if}

<div class="app-layout">
	{@render children()}
</div>

<style>
	.app-layout {
		display: flex;
		min-height: 100vh;
	}
</style>

