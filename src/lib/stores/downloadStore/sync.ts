import { listen } from '@tauri-apps/api/event';
import type { UnlistenFn } from '@tauri-apps/api/event';
import type { DownloadTask } from '$lib/types/download';
import { getTasks as getTasksCmd } from '$lib/api/cmd';
import {
    clearPendingLock,
    hasPendingLock,
    listDeletingTaskIds,
    setTasks,
    snapshotTasks
} from './state';
import { createLogger } from '$lib/utils/logger';

const logger = createLogger('DownloadStoreSync');

let unlistenFn: UnlistenFn | null = null;
let initialized = false;

function tasksChanged(prev: DownloadTask[], next: DownloadTask[]): boolean {
    if (prev.length !== next.length) return true;

    const sameHeaders = (a?: string[], b?: string[]) => {
        if (a === b) return true;
        if (!a || !b) return !a && !b;
        if (a.length !== b.length) return false;
        for (let i = 0; i < a.length; i++) {
            if (a[i] !== b[i]) return false;
        }
        return true;
    };

    for (let i = 0; i < prev.length; i++) {
        const p = prev[i];
        const n = next[i];
        if (
            p.id !== n.id ||
            p.filename !== n.filename ||
            p.url !== n.url ||
            p.state !== n.state ||
            p.progress !== n.progress ||
            p.speed !== n.speed ||
            p.completed !== n.completed ||
            p.total !== n.total ||
            p.remainingSecs !== n.remainingSecs ||
            p.savePath !== n.savePath ||
            p.errorMessage !== n.errorMessage ||
            p.userAgent !== n.userAgent ||
            p.referer !== n.referer ||
            p.proxy !== n.proxy ||
            p.maxDownloadLimit !== n.maxDownloadLimit ||
            p.addedAt !== n.addedAt ||
            p.completedAt !== n.completedAt ||
            !sameHeaders(p.headers, n.headers)
        ) {
            return true;
        }
    }

    return false;
}

function handleTasksUpdate(backendTasks: DownloadTask[]) {
    const currentTasks = snapshotTasks();
    const currentTaskMap = new Map(currentTasks.map((task) => [task.id, task]));
    const deletingTaskIds = new Set(listDeletingTaskIds());

    const nextTasks = backendTasks
        .filter((task) => !deletingTaskIds.has(task.id))
        .map((task) => {
        if (hasPendingLock(task.id)) {
            const existing = currentTaskMap.get(task.id);
            if (existing) {
                clearPendingLock(task.id);
                return { ...task, state: existing.state };
            }
        }
        return task;
    });

    if (!tasksChanged(currentTasks, nextTasks)) return;
    setTasks(nextTasks);
}

export async function initializeTaskSync() {
    if (initialized || unlistenFn) return;
    initialized = true;

    try {
        const rawTasks = await getTasksCmd();
        handleTasksUpdate(rawTasks);
    } catch (e) {
        logger.error('Initial task fetch failed', { error: e });
    }

    try {
        unlistenFn = await listen<DownloadTask[]>('tasks-update', (event) => {
            handleTasksUpdate(event.payload);
        });
    } catch (e) {
        logger.error('Failed to register tasks-update listener', { error: e });
        initialized = false;
    }
}
