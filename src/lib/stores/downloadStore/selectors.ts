import { derived, type Readable } from 'svelte/store';
import type { DownloadStats, DownloadState, DownloadTask } from '$lib/types/download';
import {
    isActiveTask,
    isCompletedTask,
    isDownloadingTask,
    isPausedTask,
    isWaitingTask
} from '$lib/utils/downloadStates';
import { formatSpeed } from '$lib/utils/formatters';
import { subscribeTasks } from './state';

function stateScore(state: DownloadState): number {
    if (state === 'active' || state === 'waiting' || state === 'paused') return 1;
    return 0;
}

function sortTasks(tasks: DownloadTask[]): DownloadTask[] {
    return tasks.slice().sort((a, b) => {
        const sa = stateScore(a.state);
        const sb = stateScore(b.state);
        if (sa !== sb) return sb - sa;
        return b.addedAt.localeCompare(a.addedAt);
    });
}

export const activeTasks: Readable<DownloadTask[]> = derived(
    { subscribe: subscribeTasks },
    ($tasks) => sortTasks($tasks.filter((task) => isActiveTask(task.state)))
);

export const completeTasks: Readable<DownloadTask[]> = derived(
    { subscribe: subscribeTasks },
    ($tasks) =>
        $tasks
            .filter((task) => isCompletedTask(task.state))
            .sort((a, b) => b.addedAt.localeCompare(a.addedAt))
);

export const allTasks: Readable<DownloadTask[]> = derived(
    { subscribe: subscribeTasks },
    ($tasks) => sortTasks($tasks)
);

export const downloadStats: Readable<DownloadStats> = derived(
    { subscribe: subscribeTasks },
    ($tasks) => {
        const activeDownloads = $tasks.filter(
            (d) => isDownloadingTask(d.state) || isWaitingTask(d.state)
        );
        const completedDownloads = $tasks.filter((d) => isCompletedTask(d.state));

        const totalSpeedBytes = activeDownloads
            .map((d) => d.speed || 0)
            .reduce((a, b) => a + b, 0);

        return {
            totalSpeed: formatSpeed(totalSpeedBytes),
            totalSpeedBytes,
            activeCount: activeDownloads.length,
            completeCount: completedDownloads.length
        };
    }
);

export function hasDownloadingTasks(tasks: DownloadTask[]): boolean {
    return tasks.some((t) => isDownloadingTask(t.state) || isWaitingTask(t.state));
}

export function hasPausedTasks(tasks: DownloadTask[]): boolean {
    return tasks.some((t) => isPausedTask(t.state));
}
