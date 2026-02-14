import type { DownloadState, DownloadTask } from '$lib/types/download';

function stateScore(state: DownloadState): number {
    if (state === 'active' || state === 'waiting' || state === 'paused') return 1;
    return 0;
}

function compareTasks(a: DownloadTask, b: DownloadTask): number {
    const sa = stateScore(a.state);
    const sb = stateScore(b.state);
    if (sa !== sb) return sb - sa;

    const byAddedAt = b.addedAt.localeCompare(a.addedAt);
    if (byAddedAt !== 0) return byAddedAt;
    return b.id.localeCompare(a.id);
}

export function sortTasks(tasks: DownloadTask[]): DownloadTask[] {
    return tasks.slice().sort(compareTasks);
}
