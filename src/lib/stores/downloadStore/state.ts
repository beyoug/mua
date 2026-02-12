import { get, writable } from 'svelte/store';
import type { DownloadTask } from '$lib/types/download';

const tasksStore = writable<DownloadTask[]>([]);

export const subscribeTasks = tasksStore.subscribe;
export const setTasks = tasksStore.set;
export const updateTasks = tasksStore.update;

const pendingStateChanges = new Map<string, number>();
const pendingLockTimeout = 5000;

interface DeleteTransaction {
    id: string;
    opId: string;
    startedAt: number;
    snapshot: DownloadTask;
    index: number;
}

const inFlightDeleteTransactions = new Map<string, DeleteTransaction>();

function createOpId(): string {
    const random = Math.random().toString(36).slice(2, 8);
    return `${Date.now()}-${random}`;
}

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

export function hasDeleteTransaction(id: string): boolean {
    return inFlightDeleteTransactions.has(id);
}

export function beginDeleteTransaction(task: DownloadTask, index: number): string {
    const opId = createOpId();
    inFlightDeleteTransactions.set(task.id, {
        id: task.id,
        opId,
        startedAt: Date.now(),
        snapshot: task,
        index
    });
    return opId;
}

export function completeDeleteTransaction(id: string, opId: string): void {
    const tx = inFlightDeleteTransactions.get(id);
    if (!tx || tx.opId !== opId) {
        return;
    }
    inFlightDeleteTransactions.delete(id);
}

export function rollbackDeleteTransaction(id: string, opId: string): { snapshot: DownloadTask; index: number } | null {
    const tx = inFlightDeleteTransactions.get(id);
    if (!tx || tx.opId !== opId) {
        return null;
    }

    inFlightDeleteTransactions.delete(id);
    return {
        snapshot: tx.snapshot,
        index: tx.index
    };
}

export function listDeletingTaskIds(): string[] {
    const now = Date.now();
    const staleThreshold = pendingLockTimeout * 2;
    const deletingIds: string[] = [];

    for (const [id, tx] of inFlightDeleteTransactions.entries()) {
        if (now - tx.startedAt > staleThreshold) {
            inFlightDeleteTransactions.delete(id);
            continue;
        }
        deletingIds.push(id);
    }

    return deletingIds;
}
