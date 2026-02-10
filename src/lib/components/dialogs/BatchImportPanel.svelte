<script lang="ts">
    import { open } from '@tauri-apps/plugin-dialog';
    import { readTextFile } from '@tauri-apps/plugin-fs';
    import { parseImportContent, type ParsedTask } from '$lib/utils/imports/parser';
    import { FileJson, Trash2, Import, Info, FileText, X } from '@lucide/svelte';
    import { fade, slide } from 'svelte/transition';

    interface Props {
        onsubmit: (tasks: ParsedTask[]) => void;
    }

    let { onsubmit }: Props = $props();

    let tasks = $state<ParsedTask[]>([]);
    let filename = $state<string>('');
    let loading = $state(false);
    let error = $state<string | null>(null);
    let expandedIndex = $state<number | null>(null);

    async function handleFileSelect() {
        try {
            loading = true;
            error = null;
            
            const selected = await open({
                multiple: false,
                filters: [{
                    name: 'Config Files',
                    extensions: ['json', 'txt', 'curl', 'har'] 
                }]
            });

            if (selected && typeof selected === 'string') {
                filename = selected;
                const content = await readTextFile(selected);
                const parsed = parseImportContent(content);
                
                if (parsed.length === 0) {
                    error = '未识别到有效的下载任务';
                    tasks = [];
                } else {
                    tasks = parsed;
                }
            }
        } catch (e) {
            console.error('File import failed:', e);
            error = `读取文件失败: ${e instanceof Error ? e.message : String(e)}`;
        } finally {
            loading = false;
        }
    }

    function removeTask(index: number) {
        tasks = tasks.filter((_, i) => i !== index);
        if (expandedIndex === index) expandedIndex = null;
    }

    function toggleDetails(index: number) {
        expandedIndex = expandedIndex === index ? null : index;
    }

    function handleSubmit() {
        if (tasks.length > 0) {
            onsubmit(tasks);
        }
    }
</script>

<div class="batch-panel h-full flex flex-col">
    <!-- Header: File Selector -->
    <div class="flex items-center gap-3 p-4 border-b border-[var(--border-primary)] bg-[var(--bg-secondary)]/30">
        <button 
            class="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-[var(--bg-card)] hover:bg-[var(--bg-hover)] border border-[var(--border-primary)] transition-colors text-sm font-medium"
            onclick={handleFileSelect}
            disabled={loading}
        >
            {#if loading}
                <span class="animate-spin">⌛</span>
            {:else}
                <FileText size={16} />
            {/if}
            <span>选择文件</span>
        </button>
        
        <div class="flex-1 overflow-hidden">
            {#if filename}
                <span class="text-sm text-[var(--text-primary)] truncate block" title={filename}>
                    {filename}
                </span>
            {:else}
                <span class="text-sm text-[var(--text-secondary)]">
                    支持 JSON, TXT, cURL 格式
                </span>
            {/if}
        </div>

        {#if error}
            <span class="text-xs text-red-500 font-medium">{error}</span>
        {/if}
    </div>

    <!-- Main: Task List -->
    <div class="flex-1 overflow-y-auto p-2 space-y-2">
        {#if tasks.length === 0 && !loading}
            <div class="h-full flex flex-col items-center justify-center text-[var(--text-secondary)] opacity-60">
                <Import size={48} strokeWidth={1} class="mb-4" />
                <p>请选择文件导入任务</p>
            </div>
        {:else}
            {#each tasks as task, i}
                <div class="task-card bg-[var(--bg-card)] border border-[var(--border-primary)] rounded-lg overflow-hidden group hover:border-[var(--accent-primary)]/50 transition-colors">
                    <!-- Row -->
                    <div class="flex items-center p-2.5 gap-3">
                        <span class="text-xs font-mono text-[var(--text-secondary)] w-6 text-center">{i + 1}</span>
                        <div class="flex-1 min-w-0 flex flex-col">
                            <span class="text-sm font-medium text-[var(--text-primary)] truncate" title={task.url}>
                                {task.url}
                            </span>
                             {#if task.filename}
                                <span class="text-xs text-[var(--text-secondary)] truncate">
                                    💾 {task.filename}
                                </span>
                            {/if}
                        </div>
                        
                        <div class="flex items-center gap-1 opacity-60 group-hover:opacity-100 transition-opacity">
                            <button 
                                class="p-1.5 hover:bg-[var(--bg-hover)] rounded-md transition-colors"
                                class:text-[var(--accent-primary)]={expandedIndex === i}
                                onclick={() => toggleDetails(i)}
                                title="查看详情"
                            >
                                <Info size={14} />
                            </button>
                            <button 
                                class="p-1.5 hover:bg-red-500/10 hover:text-red-500 rounded-md transition-colors"
                                onclick={() => removeTask(i)}
                                title="移除"
                            >
                                <Trash2 size={14} />
                            </button>
                        </div>
                    </div>

                    <!-- Details (Expanded) -->
                    {#if expandedIndex === i}
                        <div transition:slide={{ duration: 200 }} class="bg-[var(--bg-secondary)]/50 p-3 text-xs border-t border-[var(--border-primary)] space-y-2 font-mono">
                            {#if task.headers}
                                <div>
                                    <span class="text-[var(--text-secondary)] block mb-1">Headers:</span>
                                    <div class="pl-2 border-l-2 border-[var(--border-primary)] text-[var(--text-primary)] whitespace-pre-wrap break-all">
                                        {Object.entries(task.headers).map(([k,v]) => `${k}: ${v}`).join('\n')}
                                    </div>
                                </div>
                            {/if}
                            {#if task.userAgent}
                                <div>
                                    <span class="text-[var(--text-secondary)] mr-2">User-Agent:</span>
                                    <span class="text-[var(--text-primary)] break-all">{task.userAgent}</span>
                                </div>
                            {/if}
                            {#if task.referer}
                                <div>
                                    <span class="text-[var(--text-secondary)] mr-2">Referer:</span>
                                    <span class="text-[var(--text-primary)] break-all">{task.referer}</span>
                                </div>
                            {/if}
                             {#if task.proxy}
                                <div>
                                    <span class="text-[var(--text-secondary)] mr-2">Proxy:</span>
                                    <span class="text-[var(--text-primary)]">{task.proxy}</span>
                                </div>
                            {/if}
                            {#if !task.headers && !task.userAgent && !task.referer && !task.proxy}
                                <span class="text-[var(--text-secondary)] italic">无额外配置</span>
                            {/if}
                        </div>
                    {/if}
                </div>
            {/each}
        {/if}
    </div>

    <!-- Footer: Stats & Action -->
    <div class="p-3 border-t border-[var(--border-primary)] bg-[var(--bg-card)] flex justify-between items-center">
        <span class="text-xs text-[var(--text-secondary)]">
            {#if tasks.length > 0}
                已解析 {tasks.length} 个任务
            {/if}
        </span>
        <button 
            class="btn-primary py-1.5 px-4 text-sm disabled:opacity-50 disabled:cursor-not-allowed"
            disabled={tasks.length === 0}
            onclick={handleSubmit}
        >
            <Import size={16} />
            <span>全部导入</span>
        </button>
    </div>
</div>

<style>
    /* Scoped styles mainly handled by Tailwind util classes */
    .task-card {
        transition: all 0.2s ease;
    }
</style>
