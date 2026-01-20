<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';
	import { queryClient } from '$lib/config/query';
	import { QueryClientProvider } from '@tanstack/svelte-query';

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>Mua - Download Manager</title>
</svelte:head>

<QueryClientProvider client={queryClient}>
	{@render children()}
</QueryClientProvider>
