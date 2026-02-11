import type { DownloadConfig, DownloadState, DownloadTask } from '$lib/types/download';
import { extractFilenameFromUrl, formatAddedAt } from '$lib/utils/formatters';
import {
    isActiveTask,
    isDownloadingTask,
    isPausedTask,
    isWaitingTask
} from '$lib/utils/downloadStates';
import {
    addDownloadTasks as addDownloadTasksCmd,
    cancelTaskCmd,
    cancelTasksCmd,
    pauseAllTasks,
    pauseTask as pauseTaskCmd,
    removeTaskRecord,
    removeTasksCmd,
    resumeAllTasks,
    resumeTask as resumeTaskCmd
} from '$lib/api/cmd';
import {
    addPendingLock,
    clearPendingLock,
    setTasks,
    snapshotTasks,
    updateTasks
} from './state';
import { createLogger } from '$lib/utils/logger';

const logger = createLogger('DownloadStoreOps');

export async function addDownloadTasks(
    configOrConfigs: DownloadConfig | DownloadConfig[]
): Promise<void> {
    const configs = Array.isArray(configOrConfigs) ? configOrConfigs : [configOrConfigs];
    try {
        const results = await addDownloadTasksCmd(configs);

        updateTasks((tasks) => {
            const newTasks: DownloadTask[] = [];

            for (let i = 0; i < configs.length; i++) {
                const gid = results[i];
                if (!gid) continue;
                if (tasks.some((t) => t.id === gid) || newTasks.some((t) => t.id === gid)) continue;

                const config = configs[i];
                const primaryUrl = config.urls[0] || '';

                newTasks.push({
                    id: gid,
                    filename: config.filename || extractFilenameFromUrl(primaryUrl),
                    url: primaryUrl,
                    progress: 0,
                    speed: 0,
                    completed: 0,
                    total: 0,
                    remainingSecs: 0,
                    state: 'waiting',
                    addedAt: formatAddedAt(),
                    savePath: config.savePath || '',
                    userAgent: config.userAgent,
                    referer: config.referer,
                    proxy: config.proxy,
                    maxDownloadLimit: config.maxDownloadLimit,
                    headers: config.headers
                        ? config.headers
                              .split(';')
                              .map((h) => h.trim())
                              .filter((h) => h !== '')
                        : []
                });
            }

            return [...tasks, ...newTasks];
        });
    } catch (e) {
        logger.error('Failed to add tasks', { error: e });
        throw e;
    }
}

export async function pauseTask(id: string): Promise<void> {
    let originalState: DownloadState | undefined;

    try {
        addPendingLock(id);

        updateTasks((tasks) => {
            const task = tasks.find((t) => t.id === id);
            if (task) originalState = task.state;
            return tasks.map((t) =>
                t.id === id
                    ? { ...t, state: 'paused' as DownloadState, speed: 0, completedAt: new Date().toISOString() }
                    : t
            );
        });

        await pauseTaskCmd(id);
    } catch (e) {
        logger.error('Failed to pause task', { taskId: id, error: e });
        clearPendingLock(id);

        if (originalState) {
            updateTasks((tasks) =>
                tasks.map((t) => (t.id === id ? { ...t, state: originalState as DownloadState } : t))
            );
        }
    }
}

export async function resumeTask(id: string): Promise<void> {
    try {
        addPendingLock(id);

        const newGid = await resumeTaskCmd(id);
        if (newGid && newGid !== id) {
            updateTasks((tasks) => {
                const oldTask = tasks.find((t) => t.id === id);
                if (!oldTask) return tasks;

                const newTask: DownloadTask = {
                    ...oldTask,
                    id: newGid,
                    state: 'waiting',
                    progress: 0,
                    speed: 0,
                    remainingSecs: 0
                };

                return tasks.map((t) => (t.id === id ? newTask : t));
            });

            addPendingLock(newGid);
            clearPendingLock(id);
            return;
        }

        updateTasks((tasks) =>
            tasks.map((t) =>
                t.id === id ? { ...t, state: 'active' as DownloadState, completedAt: null } : t
            )
        );
    } catch (e) {
        logger.error('Failed to resume task', { taskId: id, error: e });
        clearPendingLock(id);
    }
}

export async function cancelTask(id: string): Promise<void> {
    let originalState: DownloadState | undefined;

    try {
        addPendingLock(id);

        updateTasks((tasks) => {
            const task = tasks.find((t) => t.id === id);
            if (task) originalState = task.state;
            return tasks.map((t) =>
                t.id === id ? { ...t, state: 'removed' as DownloadState, completedAt: new Date().toISOString() } : t
            );
        });

        await cancelTaskCmd(id);
    } catch (e) {
        logger.error('Failed to cancel task', { taskId: id, error: e });
        clearPendingLock(id);

        if (originalState) {
            updateTasks((tasks) =>
                tasks.map((t) => (t.id === id ? { ...t, state: originalState as DownloadState } : t))
            );
        }
    }
}

export function removeTask(id: string, deleteFile: boolean = false): void {
    updateTasks((tasks) => {
        const taskToDelete = tasks.find((t) => t.id === id);
        if (!taskToDelete) return tasks;

        if (isActiveTask(taskToDelete.state)) {
            addPendingLock(id);
        }

        removeTaskRecord(id, deleteFile).catch((e) => {
            logger.error('Failed to remove task record', { taskId: id, deleteFile, error: e });
            clearPendingLock(id);
        });

        return tasks.filter((t) => t.id !== id);
    });
}

export async function removeTasks(ids: Set<string>, deleteFile: boolean = false): Promise<void> {
    const idArray = Array.from(ids);
    try {
        await removeTasksCmd(idArray, deleteFile);
        updateTasks((tasks) => tasks.filter((t) => !ids.has(t.id)));
    } catch (e) {
        logger.error('Failed to remove tasks batch', { taskIds: idArray, deleteFile, error: e });
    }
}

export async function cancelTasks(ids: Set<string>): Promise<void> {
    const idArray = Array.from(ids);
    ids.forEach((id) => addPendingLock(id));

    try {
        await cancelTasksCmd(idArray);
        updateTasks((tasks) =>
            tasks.map((task) =>
                ids.has(task.id) && isActiveTask(task.state)
                    ? { ...task, state: 'removed' as DownloadState }
                    : task
            )
        );
    } catch (e) {
        logger.error('Failed to cancel tasks batch', { taskIds: idArray, error: e });
        ids.forEach((id) => clearPendingLock(id));
    }
}

export async function pauseAll(): Promise<void> {
    const snapshot = snapshotTasks();
    const affectedIds: string[] = [];

    try {
        updateTasks((tasks) => {
            tasks.forEach((t) => {
                if (isDownloadingTask(t.state) || isWaitingTask(t.state)) {
                    addPendingLock(t.id);
                    affectedIds.push(t.id);
                }
            });
            const now = new Date().toISOString();
            return tasks.map((t) =>
                t.state === 'active' || t.state === 'waiting'
                    ? { ...t, state: 'paused' as DownloadState, speed: 0, completedAt: now }
                    : t
            );
        });

        await pauseAllTasks();
    } catch (e) {
        logger.error('Failed to pause all tasks', { taskIds: affectedIds, error: e });
        affectedIds.forEach((id) => clearPendingLock(id));
        setTasks(snapshot);
    }
}

export async function resumeAll(): Promise<void> {
    const snapshot = snapshotTasks();
    const affectedIds: string[] = [];

    try {
        updateTasks((tasks) => {
            tasks.forEach((t) => {
                if (isPausedTask(t.state)) {
                    addPendingLock(t.id);
                    affectedIds.push(t.id);
                }
            });
            return tasks.map((t) =>
                isPausedTask(t.state)
                    ? { ...t, state: 'waiting' as DownloadState, completedAt: null }
                    : t
            );
        });

        await resumeAllTasks();
    } catch (e) {
        logger.error('Failed to resume all tasks', { taskIds: affectedIds, error: e });
        affectedIds.forEach((id) => clearPendingLock(id));
        setTasks(snapshot);
    }
}
