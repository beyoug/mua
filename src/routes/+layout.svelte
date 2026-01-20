<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';
	import { queryClient } from '$lib/config/query';
	import { QueryClientProvider } from '@tanstack/svelte-query';
	import { onMount } from 'svelte';
	import { getCurrentWindow } from '@tauri-apps/api/window';

	let { children } = $props();

	onMount(async () => {
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

<QueryClientProvider client={queryClient}>
	<main class="main-content">
		{@render children()}
	</main>
</QueryClientProvider>

<style>
	.main-content {
		min-height: 100vh;
		padding-top: 8px;
		padding-left: 32px;
		padding-right: 32px;
		padding-bottom: 24px;
		overflow-x: hidden;
		box-sizing: border-box;
	}
</style>
