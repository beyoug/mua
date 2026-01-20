<script lang="ts">
	import { APP_NAME, APP_VERSION } from '$lib/config/constants';
</script>

<main class="flex min-h-screen flex-col items-center justify-center bg-zinc-950 text-zinc-50">
	<div class="flex flex-col items-center gap-6 text-center">
		<!-- Logo placeholder -->
		<div class="flex h-24 w-24 items-center justify-center rounded-2xl bg-gradient-to-br from-emerald-500 to-teal-600 text-4xl font-bold shadow-lg shadow-emerald-500/20">
			M
		</div>

		<!-- App name -->
		<h1 class="text-4xl font-bold tracking-tight">
			{APP_NAME}
		</h1>

		<!-- Description -->
		<p class="max-w-md text-zinc-400">
			现代化的 Aria2 下载管理器，基于 Tauri 2.0 构建
		</p>

		<!-- Status card -->
		<div class="mt-8 flex flex-col gap-3 rounded-xl border border-zinc-800 bg-zinc-900/50 p-6 backdrop-blur">
			<div class="flex items-center gap-2 text-sm">
				<span class="h-2 w-2 rounded-full bg-yellow-500"></span>
				<span class="text-zinc-400">aria2c 未连接</span>
			</div>
			<p class="text-xs text-zinc-500">
				请确保 aria2c 正在运行并配置正确的 RPC 端口
			</p>
		</div>

		<!-- Version -->
		<p class="mt-4 text-xs text-zinc-600">
			v{APP_VERSION}
		</p>
	</div>
</main>
