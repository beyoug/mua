<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';
	import { queryClient } from '$lib/config/query';
	import { QueryClientProvider } from '@tanstack/svelte-query';
	import { onMount } from 'svelte';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { currentTheme, effectiveColorMode, particlesEnabled } from '$lib/stores/theme';
	import ParticleBackground from '$lib/components/ParticleBackground.svelte';

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

	onMount(async () => {
		// 禁用右键菜单
		document.addEventListener('contextmenu', (e) => e.preventDefault());

		try {
			const appWindow = getCurrentWindow();
			await appWindow.show();
			await appWindow.setFocus();
		} catch (e) {
			// 非 Tauri 环境忽略
		}
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>Mua - Download Manager</title>
</svelte:head>

{#if $particlesEnabled}
	<ParticleBackground />
{/if}

<QueryClientProvider client={queryClient}>
	<div class="app-layout">
		{@render children()}
	</div>
</QueryClientProvider>

<style>
	.app-layout {
		display: flex;
		min-height: 100vh;
	}
</style>

