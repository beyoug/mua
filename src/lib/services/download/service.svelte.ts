import { listen } from '@tauri-apps/api/event';
import type { UnlistenFn } from '@tauri-apps/api/event';
import { get, writable, type Updater } from 'svelte/store';
import type { DownloadConfig, DownloadState, DownloadTask } from '$lib/types/download';
import { extractFilenameFromUrl, formatAddedAt } from '$lib/utils/formatters';
import { createLogger } from '$lib/utils/logger';
import {
    addDownloadTasks as addDownloadTasksCmd,
    cancelTask as cancelTaskCmd,
    cancelTasks as cancelTasksCmd,
    getTasks as getTasksCmd,
    pauseAllTasks,
    pauseTask as pauseTaskCmd,
    removeTaskRecord,
    removeTasks as removeTasksCmd,
    showTaskInFolder as showTaskInFolderCmd,
    resumeAllTasks,
    resumeTask as resumeTaskCmd
} from '$lib/api/download';
import { EVENT_TASKS_DELTA } from '$lib/api/events';
import { isActiveTask, isDownloadingTask, isPausedTask, isWaitingTask } from '$lib/utils/downloadStates';
import { sortTasks } from '$lib/services/download/sort';
import type { BulkTrashPlan, SingleTaskRemovalPlan, TaskViewNav } from '$lib/services/download/types';

type TaskDeltaChange =
    | { op: 'upsert'; task: DownloadTask }
    | { op: 'remove'; id: string };

type TaskDeltaEvent =
    | { type: 'snapshot'; revision: number; tasks: DownloadTask[] }
    | { type: 'delta'; fromRevision: number; toRevision: number; seq: number; changes: TaskDeltaChange[] };

function sameHeaders(a?: string[], b?: string[]): boolean {
    if (a === b) return true;
    if (!a || !b) return !a && !b;
    if (a.length !== b.length) return false;
    for (let i = 0; i < a.length; i++) {
        if (a[i] !== b[i]) return false;
    }
    return true;
}

function sameTask(a: DownloadTask, b: DownloadTask): boolean {
    return (
        a.id === b.id &&
        a.filename === b.filename &&
        a.url === b.url &&
        a.state === b.state &&
        a.progress === b.progress &&
        a.speed === b.speed &&
        a.completed === b.completed &&
        a.total === b.total &&
        a.remainingSecs === b.remainingSecs &&
        a.savePath === b.savePath &&
        a.errorMessage === b.errorMessage &&
        a.userAgent === b.userAgent &&
        a.referer === b.referer &&
        a.proxy === b.proxy &&
        a.maxDownloadLimit === b.maxDownloadLimit &&
        a.addedAt === b.addedAt &&
        a.completedAt === b.completedAt &&
        sameHeaders(a.headers, b.headers)
    );
}

class DownloadService {
    private readonly logger = createLogger('DownloadService');
    private readonly tasksStore = writable<DownloadTask[]>([]);

    private readonly pendingStateChanges = new Map<string, number>();
    private readonly pendingLockTimeout = 5000;

    private initialized = false;
    private unlistenTasksDelta: UnlistenFn | null = null;

    private revision = 0;
    private lastSeq = 0;

    readonly subscribeTasks = this.tasksStore.subscribe;

    snapshotTasks(): DownloadTask[] {
        return get(this.tasksStore);
    }

    setTasks(tasks: DownloadTask[]): void {
        const normalized = sortTasks(tasks);
        this.tasksStore.set(normalized);
    }

    updateTasks(updater: Updater<DownloadTask[]>): void {
        this.tasksStore.update((tasks) => {
            const next = updater(tasks);
            return sortTasks(next);
        });
    }

    addPendingLock(id: string): void {
        this.pendingStateChanges.set(id, Date.now());
    }

    clearPendingLock(id: string): void {
        this.pendingStateChanges.delete(id);
    }

    hasPendingLock(id: string): boolean {
        const timestamp = this.pendingStateChanges.get(id);
        if (timestamp === undefined) return false;
        if (Date.now() - timestamp > this.pendingLockTimeout) {
            this.pendingStateChanges.delete(id);
            return false;
        }
        return true;
    }

    private applySnapshot(tasks: DownloadTask[]): void {
        const current = this.snapshotTasks();
        const currentById = new Map(current.map((task) => [task.id, task]));
        const next: DownloadTask[] = [];

        for (const rawTask of tasks) {
            const existing = currentById.get(rawTask.id);
            if (existing && this.hasPendingLock(rawTask.id)) {
                this.clearPendingLock(rawTask.id);
                next.push({ ...rawTask, state: existing.state });
            } else {
                next.push(rawTask);
            }
        }

        const sortedNext = sortTasks(next);
        if (sortedNext.length !== current.length) {
            this.tasksStore.set(sortedNext);
            return;
        }

        for (let i = 0; i < sortedNext.length; i++) {
            if (!sameTask(sortedNext[i], current[i])) {
                this.tasksStore.set(sortedNext);
                return;
            }
        }
    }

    private applyDelta(event: Extract<TaskDeltaEvent, { type: 'delta' }>): void {
        this.tasksStore.update((tasks) => {
            const byId = new Map(tasks.map((task) => [task.id, task]));

            for (const change of event.changes) {
                if (change.op === 'remove') {
                    byId.delete(change.id);
                    this.clearPendingLock(change.id);
                    continue;
                }

                const incoming = change.task;
                const existing = byId.get(incoming.id);
                if (existing && this.hasPendingLock(incoming.id)) {
                    this.clearPendingLock(incoming.id);
                    byId.set(incoming.id, { ...incoming, state: existing.state });
                } else {
                    byId.set(incoming.id, incoming);
                }
            }

            return sortTasks(Array.from(byId.values()));
        });
    }

    private async forceResync(): Promise<void> {
        const tasks = await getTasksCmd();
        this.applySnapshot(tasks);
    }

    async initializeSync(): Promise<void> {
        if (this.initialized || this.unlistenTasksDelta) return;
        this.initialized = true;

        try {
            const rawTasks = await getTasksCmd();
            this.applySnapshot(rawTasks);
        } catch (e) {
            this.logger.error('Initial task fetch failed', { error: e });
        }

        try {
            this.unlistenTasksDelta = await listen<TaskDeltaEvent>(EVENT_TASKS_DELTA, async (event) => {
                const payload = event.payload;
                if (payload.type === 'snapshot') {
                    this.revision = payload.revision;
                    this.lastSeq = 0;
                    this.applySnapshot(payload.tasks);
                    return;
                }

                if (
                    payload.fromRevision !== this.revision ||
                    (this.lastSeq !== 0 && payload.seq !== this.lastSeq + 1)
                ) {
                    this.logger.warn('Delta sequence/revision mismatch, forcing full resync', {
                        expectedRevision: this.revision,
                        incomingFromRevision: payload.fromRevision,
                        expectedSeq: this.lastSeq + 1,
                        incomingSeq: payload.seq
                    });
                    try {
                        await this.forceResync();
                        this.revision = payload.toRevision;
                        this.lastSeq = payload.seq;
                    } catch (e) {
                        this.logger.error('Resync after delta mismatch failed', { error: e });
                    }
                    return;
                }

                this.applyDelta(payload);
                this.revision = payload.toRevision;
                this.lastSeq = payload.seq;
            });
        } catch (e) {
            this.logger.error('Failed to register tasks-delta listener', { error: e });
            this.initialized = false;
        }
    }

    async addDownloadTasks(configOrConfigs: DownloadConfig | DownloadConfig[]): Promise<void> {
        const configs = Array.isArray(configOrConfigs) ? configOrConfigs : [configOrConfigs];

        try {
            const results = await addDownloadTasksCmd(configs);
            this.updateTasks((tasks) => {
                const byId = new Set(tasks.map((task) => task.id));
                const newTasks: DownloadTask[] = [];

                for (let i = 0; i < configs.length; i++) {
                    const gid = results[i];
                    if (!gid || byId.has(gid)) continue;

                    const config = configs[i];
                    const primaryUrl = config.urls[0] || '';
                    byId.add(gid);

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
                        errorMessage: '',
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
            this.logger.error('Failed to add tasks', { error: e });
            throw e;
        }
    }

    async pauseTask(id: string): Promise<void> {
        let originalState: DownloadState | undefined;

        try {
            this.addPendingLock(id);
            this.updateTasks((tasks) =>
                tasks.map((task) => {
                    if (task.id !== id) return task;
                    originalState = task.state;
                    return {
                        ...task,
                        state: 'paused',
                        speed: 0,
                        completedAt: new Date().toISOString()
                    };
                })
            );
            await pauseTaskCmd(id);
        } catch (e) {
            this.logger.error('Failed to pause task', { taskId: id, error: e });
            this.clearPendingLock(id);

            if (originalState) {
                this.updateTasks((tasks) =>
                    tasks.map((task) => (task.id === id ? { ...task, state: originalState as DownloadState } : task))
                );
            }
        }
    }

    async resumeTask(id: string): Promise<void> {
        try {
            this.addPendingLock(id);
            const newGid = await resumeTaskCmd(id);

            if (newGid && newGid !== id) {
                this.updateTasks((tasks) =>
                    tasks.map((task) =>
                        task.id === id
                            ? {
                                ...task,
                                id: newGid,
                                state: 'waiting',
                                progress: 0,
                                speed: 0,
                                remainingSecs: 0
                            }
                            : task
                    )
                );
                this.addPendingLock(newGid);
                this.clearPendingLock(id);
                return;
            }

            this.updateTasks((tasks) =>
                tasks.map((task) =>
                    task.id === id ? { ...task, state: 'active', completedAt: null } : task
                )
            );
        } catch (e) {
            this.logger.error('Failed to resume task', { taskId: id, error: e });
            this.clearPendingLock(id);
        }
    }

    async cancelTask(id: string): Promise<void> {
        let originalState: DownloadState | undefined;

        try {
            this.addPendingLock(id);
            this.updateTasks((tasks) =>
                tasks.map((task) => {
                    if (task.id !== id) return task;
                    originalState = task.state;
                    return { ...task, state: 'removed', completedAt: new Date().toISOString() };
                })
            );
            await cancelTaskCmd(id);
        } catch (e) {
            this.logger.error('Failed to cancel task', { taskId: id, error: e });
            this.clearPendingLock(id);

            if (originalState) {
                this.updateTasks((tasks) =>
                    tasks.map((task) => (task.id === id ? { ...task, state: originalState as DownloadState } : task))
                );
            }
        }
    }

    async openTaskFolder(id: string): Promise<void> {
        await showTaskInFolderCmd(id);
    }

    planBulkTrash(
        nav: TaskViewNav,
        isSelectionMode: boolean,
        selectedIds: Set<string>,
        currentTasks: DownloadTask[]
    ): BulkTrashPlan {
        if (!isSelectionMode) {
            return { action: 'enter_selection' };
        }

        if (selectedIds.size === 0) {
            return { action: 'select_all' };
        }

        if (nav === 'active') {
            return { action: 'execute', deleteFile: false };
        }

        const hasCompletedTask = currentTasks
            .filter((task) => selectedIds.has(task.id))
            .some((task) => task.state === 'complete');

        if (!hasCompletedTask) {
            return { action: 'execute', deleteFile: true };
        }

        const count = selectedIds.size;
        if (nav === 'history') {
            return {
                action: 'confirm',
                dialog: {
                    title: '删除记录',
                    description: `确定要永久删除这 ${count} 条任务记录吗？`,
                    confirmText: '确定'
                }
            };
        }

        return {
            action: 'confirm',
            dialog: {
                title: '清空记录',
                description: `确定要清空这 ${count} 条已完成的任务记录吗？`,
                confirmText: '确定'
            }
        };
    }

    planSingleTaskRemoval(task: DownloadTask): SingleTaskRemovalPlan {
        if (isActiveTask(task.state)) {
            return { action: 'cancel' };
        }

        if (task.state === 'complete') {
            return {
                action: 'confirm_remove',
                dialog: {
                    title: '删除任务',
                    description: '确定要删除这条任务记录吗？',
                    confirmText: '删除'
                }
            };
        }

        return { action: 'remove', deleteFile: true };
    }

    async executeClear(
        nav: TaskViewNav,
        selectedIds: Set<string>,
        itemToDelete: string | null,
        deleteFile: boolean
    ): Promise<void> {
        if (itemToDelete) {
            this.removeTask(itemToDelete, deleteFile);
            return;
        }

        if (nav === 'active') {
            await this.cancelTasks(selectedIds);
            return;
        }

        await this.removeTasks(selectedIds, deleteFile);
    }

    removeTask(id: string, deleteFile: boolean = false): void {
        this.updateTasks((tasks) => {
            const existing = tasks.find((task) => task.id === id);
            if (!existing) return tasks;

            if (isActiveTask(existing.state)) {
                this.addPendingLock(id);
            }

            removeTaskRecord(id, deleteFile).catch((e) => {
                this.logger.error('Failed to remove task record', { taskId: id, deleteFile, error: e });
                this.clearPendingLock(id);
                this.forceResync().catch((resyncError) => {
                    this.logger.error('Failed to resync after remove task record failure', {
                        taskId: id,
                        error: resyncError
                    });
                });
            });

            return tasks.filter((task) => task.id !== id);
        });
    }

    async removeTasks(ids: Set<string>, deleteFile: boolean = false): Promise<void> {
        const idArray = Array.from(ids);
        try {
            await removeTasksCmd(idArray, deleteFile);
            this.updateTasks((tasks) => tasks.filter((task) => !ids.has(task.id)));
        } catch (e) {
            this.logger.error('Failed to remove tasks batch', { taskIds: idArray, deleteFile, error: e });
        }
    }

    async cancelTasks(ids: Set<string>): Promise<void> {
        const idArray = Array.from(ids);
        ids.forEach((id) => {
            this.addPendingLock(id);
        });

        try {
            await cancelTasksCmd(idArray);
            this.updateTasks((tasks) =>
                tasks.map((task) =>
                    ids.has(task.id) && isActiveTask(task.state) ? { ...task, state: 'removed' } : task
                )
            );
        } catch (e) {
            this.logger.error('Failed to cancel tasks batch', { taskIds: idArray, error: e });
            ids.forEach((id) => {
                this.clearPendingLock(id);
            });
        }
    }

    async pauseAll(): Promise<void> {
        const snapshot = this.snapshotTasks();
        const affectedIds: string[] = [];

        try {
            this.updateTasks((tasks) => {
                const now = new Date().toISOString();
                return tasks.map((task) => {
                    if (isDownloadingTask(task.state) || isWaitingTask(task.state)) {
                        this.addPendingLock(task.id);
                        affectedIds.push(task.id);
                        return { ...task, state: 'paused', speed: 0, completedAt: now };
                    }
                    return task;
                });
            });

            await pauseAllTasks();
        } catch (e) {
            this.logger.error('Failed to pause all tasks', { taskIds: affectedIds, error: e });
            affectedIds.forEach((id) => {
                this.clearPendingLock(id);
            });
            this.setTasks(snapshot);
        }
    }

    async resumeAll(): Promise<void> {
        const snapshot = this.snapshotTasks();
        const affectedIds: string[] = [];

        try {
            this.updateTasks((tasks) =>
                tasks.map((task) => {
                    if (isPausedTask(task.state)) {
                        this.addPendingLock(task.id);
                        affectedIds.push(task.id);
                        return { ...task, state: 'waiting', completedAt: null };
                    }
                    return task;
                })
            );
            await resumeAllTasks();
        } catch (e) {
            this.logger.error('Failed to resume all tasks', { taskIds: affectedIds, error: e });
            affectedIds.forEach((id) => {
                this.clearPendingLock(id);
            });
            this.setTasks(snapshot);
        }
    }
}

export const downloadService = new DownloadService();
