import { derived, type Readable } from 'svelte/store';
import type { DownloadConfig, DownloadStats, DownloadTask } from '$lib/types/download';
import {
    isActiveTask,
    isCompletedTask,
    isDownloadingTask,
    isPausedTask,
    isWaitingTask
} from '$lib/utils/downloadStates';
import { formatSpeed } from '$lib/utils/formatters';
import {
    downloadService
} from '$lib/services/download/service.svelte';
import type { TaskViewNav, BulkTrashPlan, SingleTaskRemovalPlan } from '$lib/services/download/types';

export { downloadService };
export type { TaskViewNav, BulkTrashPlan, SingleTaskRemovalPlan };
export { TaskController } from './taskController.svelte';

export const activeTasks: Readable<DownloadTask[]> = derived(
    { subscribe: downloadService.subscribeTasks },
    ($tasks) => $tasks.filter((task) => isActiveTask(task.state))
);

export const completeTasks: Readable<DownloadTask[]> = derived(
    { subscribe: downloadService.subscribeTasks },
    ($tasks) => $tasks.filter((task) => isCompletedTask(task.state))
);

export const allTasks: Readable<DownloadTask[]> = derived(
    { subscribe: downloadService.subscribeTasks },
    ($tasks) => $tasks
);

export const downloadStats: Readable<DownloadStats> = derived(
    { subscribe: downloadService.subscribeTasks },
    ($tasks) => {
        const activeDownloads = $tasks.filter(
            (task) => isDownloadingTask(task.state) || isWaitingTask(task.state)
        );
        const completedDownloads = $tasks.filter((task) => isCompletedTask(task.state));
        const totalSpeedBytes = activeDownloads.reduce((sum, task) => sum + (task.speed || 0), 0);

        return {
            totalSpeed: formatSpeed(totalSpeedBytes),
            totalSpeedBytes,
            activeCount: activeDownloads.length,
            completeCount: completedDownloads.length
        };
    }
);

export async function addDownloadTasks(configOrConfigs: DownloadConfig | DownloadConfig[]): Promise<void> {
    await downloadService.addDownloadTasks(configOrConfigs);
}

export async function pauseTask(id: string): Promise<void> {
    await downloadService.pauseTask(id);
}

export async function resumeTask(id: string): Promise<void> {
    await downloadService.resumeTask(id);
}

export async function cancelTask(id: string): Promise<void> {
    await downloadService.cancelTask(id);
}

export async function removeTask(id: string, deleteFile: boolean = false): Promise<void> {
    await downloadService.removeTask(id, deleteFile);
}

export async function removeTasks(ids: Set<string>, deleteFile: boolean = false): Promise<void> {
    await downloadService.removeTasks(ids, deleteFile);
}

export async function cancelTasks(ids: Set<string>): Promise<void> {
    await downloadService.cancelTasks(ids);
}

export async function pauseAll(): Promise<void> {
    await downloadService.pauseAll();
}

export async function resumeAll(): Promise<void> {
    await downloadService.resumeAll();
}

export async function openTaskFolder(id: string): Promise<void> {
    await downloadService.openTaskFolder(id);
}

export function hasDownloadingTasks(tasks: DownloadTask[]): boolean {
    return tasks.some((task) => isDownloadingTask(task.state) || isWaitingTask(task.state));
}

export function hasPausedTasks(tasks: DownloadTask[]): boolean {
    return tasks.some((task) => isPausedTask(task.state));
}
