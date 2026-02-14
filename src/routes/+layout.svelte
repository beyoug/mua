<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';
	import { onMount } from 'svelte';
	import { appSettings } from '$lib/services/settings';
    import { systemPrefersDark, particlesEnabled } from '$lib/services/theme';
	import ParticleBackground from '$lib/components/effects/ParticleBackground.svelte';
	import { bootApp } from '$lib/services/boot';
	import { createLogger } from '$lib/utils/logger';

	const logger = createLogger('Layout');

	let { children } = $props();

	// 直接从 appSettings 订阅，确保最高级别的响应性追踪
	$effect(() => {
		const s = $appSettings;
		const themeId = s.theme;
		const mode = s.colorMode === 'auto' ? ($systemPrefersDark ? 'dark' : 'light') : s.colorMode;
		
		const classes = [`theme-${themeId}`];
		if (mode === 'light') {
			classes.push('light');
		}
		document.documentElement.className = classes.join(' ');
		
		// 动态设置 color-scheme
		document.documentElement.style.colorScheme = mode as string;
	});

	onMount(() => {
		let cleanup: (() => void) | undefined;

		bootApp().then(cb => {
			cleanup = cb;
		}).catch(e => {
			logger.error('Core boot failure', { error: e });
		});

		return () => {
			if (cleanup) cleanup();
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
