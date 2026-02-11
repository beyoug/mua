import { get, writable } from 'svelte/store';
import type { DownloadTask } from '$lib/types/download';

const tasksStore = writable<DownloadTask[]>([]);

export const subscribeTasks = tasksStore.subscribe;
export const setTasks = tasksStore.set;
export const updateTasks = tasksStore.update;

const pendingStateChanges = new Map<string, number>();
const pendingLockTimeout = 5000;

export function addPendingLock(id: string) {
    pendingStateChanges.set(id, Date.now());
}

export function clearPendingLock(id: string) {
    pendingStateChanges.delete(id);
}

export function hasPendingLock(id: string): boolean {
    const timestamp = pendingStateChanges.get(id);
    if (timestamp === undefined) return false;
    if (Date.now() - timestamp > pendingLockTimeout) {
        pendingStateChanges.delete(id);
        return false;
    }
    return true;
}

export function snapshotTasks(): DownloadTask[] {
    return get({ subscribe: subscribeTasks });
}
