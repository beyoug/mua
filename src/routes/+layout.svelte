<script lang="ts">
	import "./layout.css";
	import favicon from "$lib/assets/favicon.svg";
	import { onMount } from "svelte";
	import { appSettings } from "$lib/services/settings";
	import { systemPrefersDark, particlesEnabled } from "$lib/services/theme";
	import ParticleBackground from "$lib/components/effects/ParticleBackground.svelte";
	import { bootApp } from "$lib/services/boot";
	import { createLogger } from "$lib/utils/logger";

	const logger = createLogger("Layout");

	let { children } = $props();

	// 直接从 appSettings 订阅，确保最高级别的响应性追踪
	$effect(() => {
		const s = $appSettings;
		const themeId = s.theme;
		const mode =
			s.colorMode === "auto"
				? $systemPrefersDark
					? "dark"
					: "light"
				: s.colorMode;
		// 安全的 class 切换（不覆盖 Tailwind 等其他 class）
		const el = document.documentElement;

		// 移除旧的 theme-* class
		Array.from(el.classList).forEach((cls) => {
			if (cls.startsWith("theme-")) el.classList.remove(cls);
		});
		el.classList.remove("light", "dark");

		// 写入新 class
		el.classList.add(`theme-${themeId}`);
		el.classList.add(mode === "light" ? "light" : "dark");

		// 动态设置 color-scheme
		document.documentElement.style.colorScheme = mode as string;
	});

	onMount(() => {
		let cleanup: (() => void) | undefined;

		bootApp()
			.then((cb) => {
				cleanup = cb;
			})
			.catch((e) => {
				logger.error("Core boot failure", { error: e });
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
