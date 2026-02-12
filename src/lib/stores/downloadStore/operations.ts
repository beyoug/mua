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
    beginDeleteTransaction,
    clearPendingLock,
    completeDeleteTransaction,
    hasDeleteTransaction,
    rollbackDeleteTransaction,
    setTasks,
    snapshotTasks,
    updateTasks
} from './state';
import { createLogger } from '$lib/utils/logger';
import { getErrorMessage } from '$lib/utils/errors';

const logger = createLogger('DownloadStoreOps');

export async function addDownloadTasks(
    configOrConfigs: DownloadConfig | DownloadConfig[]
): Promise<void> {
    const configs = Array.isArray(configOrConfigs) ? configOrConfigs : [configOrConfigs];
    try {
        const results = await addDownloadTasksCmd(configs);
        const validGids = results.filter((gid): gid is string => Boolean(gid));
        if (validGids.length === 0) {
            throw new Error('未能创建任何下载任务');
        }

        if (validGids.length < configs.length) {
            logger.warn('Some tasks failed to be created', {
                requested: configs.length,
                created: validGids.length
            });
        }

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
        throw new Error(getErrorMessage(e, '添加任务失败，请检查 Aria2 服务状态'));
    }
}

export async function pauseTask(id: string): Promise<void> {
    let originalTask: DownloadTask | undefined;

    try {
        addPendingLock(id);

        updateTasks((tasks) => {
            const task = tasks.find((t) => t.id === id);
            if (task) originalTask = { ...task };
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

        if (originalTask) {
            const rollbackTask = originalTask;
            updateTasks((tasks) =>
                tasks.map((t) => (t.id === id ? { ...rollbackTask } : t))
            );
        }

        throw new Error(getErrorMessage(e, '暂停任务失败'));
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
        throw new Error(getErrorMessage(e, '恢复任务失败'));
    }
}

export async function cancelTask(id: string): Promise<void> {
    let originalTask: DownloadTask | undefined;

    try {
        addPendingLock(id);

        updateTasks((tasks) => {
            const task = tasks.find((t) => t.id === id);
            if (task) originalTask = { ...task };
            return tasks.map((t) =>
                t.id === id ? { ...t, state: 'removed' as DownloadState, completedAt: new Date().toISOString() } : t
            );
        });

        await cancelTaskCmd(id);
    } catch (e) {
        logger.error('Failed to cancel task', { taskId: id, error: e });
        clearPendingLock(id);

        if (originalTask) {
            const rollbackTask = originalTask;
            updateTasks((tasks) =>
                tasks.map((t) => (t.id === id ? { ...rollbackTask } : t))
            );
        }

        throw new Error(getErrorMessage(e, '取消任务失败'));
    }
}

export async function removeTask(id: string, deleteFile: boolean = false): Promise<void> {
    if (hasDeleteTransaction(id)) {
        return;
    }

    const currentTasks = snapshotTasks();
    const index = currentTasks.findIndex((t) => t.id === id);
    if (index === -1) {
        return;
    }

    const taskToDelete = currentTasks[index];
    const opId = beginDeleteTransaction(taskToDelete, index);
    const wasActiveTask = isActiveTask(taskToDelete.state);
    if (wasActiveTask) {
        addPendingLock(id);
    }

    updateTasks((tasks) => tasks.filter((t) => t.id !== id));

    try {
        await removeTaskRecord(id, deleteFile);
        completeDeleteTransaction(id, opId);
    } catch (e) {
        logger.error('Failed to remove task record', { taskId: id, deleteFile, error: e });
        const rollback = rollbackDeleteTransaction(id, opId);
        if (rollback) {
            updateTasks((tasks) => {
                if (tasks.some((t) => t.id === id)) {
                    return tasks;
                }

                const next = tasks.slice();
                next.splice(Math.min(rollback.index, next.length), 0, rollback.snapshot);
                return next;
            });
        }
        throw new Error(getErrorMessage(e, '删除任务记录失败'));
    } finally {
        if (wasActiveTask) {
            clearPendingLock(id);
        }
    }
}

export async function removeTasks(ids: Set<string>, deleteFile: boolean = false): Promise<void> {
    const idArray = Array.from(ids);

    const currentTasks = snapshotTasks();
    const rollbackOrder: { id: string; opId: string }[] = [];
    for (let index = 0; index < currentTasks.length; index++) {
        const task = currentTasks[index];
        if (!ids.has(task.id)) {
            continue;
        }

        if (hasDeleteTransaction(task.id)) {
            continue;
        }

        const opId = beginDeleteTransaction(task, index);
        rollbackOrder.push({ id: task.id, opId });
    }

    if (rollbackOrder.length > 0) {
        updateTasks((tasks) => tasks.filter((t) => !ids.has(t.id)));
    }

    try {
        const result = await removeTasksCmd(idArray, deleteFile);
        const succeeded = new Set(result.succeededGids);
        const failedCount = result.failedGids.length;

        rollbackOrder.forEach(({ id, opId }) => {
            if (succeeded.has(id)) {
                completeDeleteTransaction(id, opId);
                return;
            }

            const rollback = rollbackDeleteTransaction(id, opId);
            if (!rollback) {
                return;
            }

            updateTasks((tasks) => {
                if (tasks.some((t) => t.id === rollback.snapshot.id)) {
                    return tasks;
                }

                const next = tasks.slice();
                next.splice(Math.min(rollback.index, next.length), 0, rollback.snapshot);
                return next;
            });
        });

        if (failedCount > 0) {
            throw new Error(`批量删除部分失败（成功 ${result.succeededGids.length}/${result.requested}）`);
        }
    } catch (e) {
        logger.error('Failed to remove tasks batch', { taskIds: idArray, deleteFile, error: e });
        if (rollbackOrder.length > 0) {
            const rollbackItems = rollbackOrder
                .map(({ id, opId }) => rollbackDeleteTransaction(id, opId))
                .filter((item): item is { snapshot: DownloadTask; index: number } => item !== null)
                .sort((a, b) => a.index - b.index);

            if (rollbackItems.length > 0) {
                updateTasks((tasks) => {
                    const next = tasks.slice();
                    for (const item of rollbackItems) {
                        if (next.some((t) => t.id === item.snapshot.id)) {
                            continue;
                        }
                        next.splice(Math.min(item.index, next.length), 0, item.snapshot);
                    }
                    return next;
                });
            }
        }
        throw new Error(getErrorMessage(e, '批量删除任务失败'));
    }
}

export async function cancelTasks(ids: Set<string>): Promise<void> {
    const idArray = Array.from(ids);
    ids.forEach((id) => addPendingLock(id));

    try {
        const result = await cancelTasksCmd(idArray);
        const succeeded = new Set(result.succeededGids);

        updateTasks((tasks) =>
            tasks.map((task) =>
                succeeded.has(task.id) && isActiveTask(task.state)
                    ? { ...task, state: 'removed' as DownloadState }
                    : task
            )
        );

        idArray.forEach((id) => clearPendingLock(id));

        if (result.failedGids.length > 0) {
            throw new Error(`批量取消部分失败（成功 ${result.succeededGids.length}/${result.requested}）`);
        }
    } catch (e) {
        logger.error('Failed to cancel tasks batch', { taskIds: idArray, error: e });
        ids.forEach((id) => clearPendingLock(id));
        throw new Error(getErrorMessage(e, '批量取消任务失败'));
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
        throw new Error(getErrorMessage(e, '全部暂停失败'));
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
        throw new Error(getErrorMessage(e, '全部恢复失败'));
    }
}
