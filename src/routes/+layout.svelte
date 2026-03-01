<script lang="ts">
	import "./layout.css";
	import favicon from "$lib/assets/favicon.svg";
	import { onMount } from "svelte";
	import { appSettings } from "$lib/services/settings";
	import {
		applyThemeToDocument,
		resolveThemeState,
		systemPrefersDark,
		particlesEnabled
	} from "$lib/services/theme";
	import ParticleBackground from "$lib/components/effects/ParticleBackground.svelte";
	import { bootApp, shutdownBootServices } from "$lib/services/boot";
	import { createLogger } from "$lib/utils/logger";
	import { invoke } from "@tauri-apps/api/core";

	const logger = createLogger("Layout");

	let { children } = $props();

	$effect(() => {
		const state = resolveThemeState($appSettings, $systemPrefersDark);
		applyThemeToDocument(state);
	});

		onMount(() => {
		bootApp()
			.catch((e) => {
				logger.error("Core boot failure", { error: e });
			})
			.finally(() => {
				void invoke("show_main_window").catch((e) => {
					logger.warn("show_main_window failed", { error: e });
				});
			});

		return () => {
			shutdownBootServices();
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
