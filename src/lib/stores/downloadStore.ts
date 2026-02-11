/**
 * downloadStore.ts - 集中式下载任务状态管理
 *
 * 职责：
 * 1. 管理下载任务数组
 * 2. 提供任务 CRUD 操作方法
 * 3. 导出筛选后的任务列表 (active/completed/all)
 * 4. 导出统计信息 (速度、任务数量)
 */

import { writable, derived, get, type Readable } from 'svelte/store';
import type { DownloadTask, DownloadConfig, DownloadStats, DownloadState } from '$lib/types/download';
import {
    isActiveTask,
    isCompletedTask,
    isDownloadingTask,
    isWaitingTask,
    isPausedTask,
} from '$lib/utils/downloadStates';
import {
    formatSpeed,
    formatAddedAt,
    extractFilenameFromUrl
} from '$lib/utils/formatters';
import {
    getTasks,
    addDownloadTasks as addDownloadTasksCmd,
    pauseTask as pauseTaskCmd,
    resumeTask as resumeTaskCmd,
    cancelTaskCmd,
    removeTaskRecord,
    pauseAllTasks,
    resumeAllTasks,
    removeTasksCmd,
    cancelTasksCmd
} from '$lib/api/cmd';



// ============ 内部状态 ============

// 所有下载任务（主数据源）
const { subscribe: subscribeTasks, set: setTasks, update: updateTasks } = writable<DownloadTask[]>([]);

// 乐观更新锁：记录正在进行操作的任务 ID（附带超时时间戳）
const pendingStateChanges = new Map<string, number>();

// 乐观锁超时时间（ms）
const PENDING_LOCK_TIMEOUT = 5000;

/** 添加乐观锁（自带超时安全网） */
function addPendingLock(id: string) {
    pendingStateChanges.set(id, Date.now());
}

/** 检查锁是否有效（过期自动清除） */
function hasPendingLock(id: string): boolean {
    const timestamp = pendingStateChanges.get(id);
    if (timestamp === undefined) return false;
    if (Date.now() - timestamp > PENDING_LOCK_TIMEOUT) {
        pendingStateChanges.delete(id);
        return false;
    }
    return true;
}

/** 浅比较两个任务数组是否有实际变化（只比较高频变化字段） */
function tasksChanged(prev: DownloadTask[], next: DownloadTask[]): boolean {
    if (prev.length !== next.length) return true;
    for (let i = 0; i < prev.length; i++) {
        const p = prev[i], n = next[i];
        if (
            p.id !== n.id ||
            p.state !== n.state ||
            p.progress !== n.progress ||
            p.speed !== n.speed ||
            p.completed !== n.completed ||
            p.total !== n.total ||
            p.remainingSecs !== n.remainingSecs
        ) return true;
    }
    return false;
}

// 后端推送的事件监听器
import { listen } from '@tauri-apps/api/event';
import type { UnlistenFn } from '@tauri-apps/api/event';

let unlistenFn: UnlistenFn | null = null;

async function setupEventListener() {
    if (unlistenFn) return;

    // 初始获取，以便立即填充数据（在第一个事件之前）
    try {
        const rawTasks = await getTasks();
        handleTasksUpdate(rawTasks);
    } catch (e) {
        console.error("初始任务获取失败:", e);
    }

    try {
        unlistenFn = await listen<any[]>('tasks-update', (event) => {
            const rawTasks = event.payload;
            handleTasksUpdate(rawTasks);
        });
    } catch (e) {
        console.error("设置事件监听器失败:", e);
    }
}

// 统一的处理逻辑
function handleTasksUpdate(backendTasks: DownloadTask[]) {
    const currentTasks = get({ subscribe: subscribeTasks });

    // 后端是单一事实来源（Single Source of Truth）
    // 仅对被锁定的任务保留本地状态（乐观更新）
    const nextTasks = backendTasks.map(task => {
        if (hasPendingLock(task.id)) {
            const existing = currentTasks.find(t => t.id === task.id);
            if (existing) {
                pendingStateChanges.delete(task.id);
                return { ...task, state: existing.state };
            }
        }
        return task;
    });

    // 浅比较：数据未变化时跳过更新，避免无效重渲染
    if (!tasksChanged(currentTasks, nextTasks)) return;
    setTasks(nextTasks);
}

// 自动开始监听
if (typeof window !== 'undefined') {
    setupEventListener();
}


// ============ 排序辅助 ============

/** 状态排序分数：活跃任务排在前面 */
function stateScore(state: DownloadState): number {
    if (state === 'active' || state === 'waiting' || state === 'paused') return 1;
    return 0;
}

/** 按状态分数降序 → 添加时间降序排列 */
function sortTasks(tasks: DownloadTask[]): DownloadTask[] {
    return tasks.slice().sort((a, b) => {
        const sa = stateScore(a.state), sb = stateScore(b.state);
        if (sa !== sb) return sb - sa;
        return b.addedAt.localeCompare(a.addedAt);
    });
}


// ============ 导出的 Derived Stores ============

/**
 * 活跃任务列表（进行中、等待中、已暂停）
 */
export const activeTasks: Readable<DownloadTask[]> = derived(
    { subscribe: subscribeTasks },
    ($tasks) => sortTasks($tasks.filter(task => isActiveTask(task.state)))
);

/**
 * 已完成任务列表
 */
export const completeTasks: Readable<DownloadTask[]> = derived(
    { subscribe: subscribeTasks },
    ($tasks) => $tasks
        .filter(task => isCompletedTask(task.state))
        .sort((a, b) => b.addedAt.localeCompare(a.addedAt))
);

/**
 * 所有任务列表（历史记录）
 */
export const allTasks: Readable<DownloadTask[]> = derived(
    { subscribe: subscribeTasks },
    ($tasks) => sortTasks($tasks)
);

/**
 * 下载统计信息
 */
export const downloadStats: Readable<DownloadStats> = derived(
    { subscribe: subscribeTasks },
    ($tasks) => {
        const activeDownloads = $tasks.filter(d =>
            isDownloadingTask(d.state) || isWaitingTask(d.state)
        );
        const completedDownloads = $tasks.filter(d => isCompletedTask(d.state));

        const totalSpeedBytes = activeDownloads
            .map(d => d.speed || 0)
            .reduce((a, b) => a + b, 0);

        return {
            totalSpeed: formatSpeed(totalSpeedBytes),
            totalSpeedBytes,
            activeCount: activeDownloads.length,
            completeCount: completedDownloads.length
        };
    }
);

// ============ 任务操作方法 ============

/**
 * 添加下载任务（统一入口，支持单个/批量）
 */
export async function addDownloadTasks(configOrConfigs: DownloadConfig | DownloadConfig[]): Promise<void> {
    const configs = Array.isArray(configOrConfigs) ? configOrConfigs : [configOrConfigs];
    try {
        const results = await addDownloadTasksCmd(configs);

        updateTasks(tasks => {
            const newTasks: DownloadTask[] = [];

            for (let i = 0; i < configs.length; i++) {
                const gid = results[i];
                if (!gid) continue;

                // 防止重复添加
                if (tasks.some(t => t.id === gid) || newTasks.some(t => t.id === gid)) continue;

                const config = configs[i];
                const primaryUrl = config.urls[0] || '';

                const newTask: DownloadTask = {
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
                    headers: config.headers ? config.headers.split(';').map(h => h.trim()).filter(h => h !== '') : []
                };
                newTasks.push(newTask);
            }
            return [...tasks, ...newTasks];
        });
    } catch (e) {
        console.error('添加任务失败:', e);
        throw e;
    }
}

/**
 * 暂停任务
 */
export async function pauseTask(id: string): Promise<void> {
    // 保存原始状态用于回滚
    let originalState: DownloadState | undefined;

    try {
        // 锁定状态以避免抖动
        addPendingLock(id);

        // 获取并保存原始状态
        updateTasks(tasks => {
            const task = tasks.find(t => t.id === id);
            if (task) {
                originalState = task.state;
            }
            // 乐观更新
            return tasks.map(t =>
                t.id === id ? { ...t, state: 'paused' as DownloadState, speed: 0, completedAt: new Date().toISOString() } : t
            );
        });

        await pauseTaskCmd(id);
    } catch (e) {
        console.error(`暂停任务 ${id} 失败:`, e);
        pendingStateChanges.delete(id);

        // 回滚状态
        if (originalState) {
            updateTasks(tasks => tasks.map(t =>
                t.id === id ? { ...t, state: originalState! } : t
            ));
        }
    }
}

/**
 * 恢复任务
 * 对于 'paused' 任务：调用 aria2.unpause
 * 对于 'removed'/'error' 任务：重新添加任务 (Retry)
 */
export async function resumeTask(id: string): Promise<void> {
    try {
        // 锁定状态以防止 UI 闪烁
        addPendingLock(id);

        const newGid = await resumeTaskCmd(id);

        if (newGid && newGid !== id) {
            // 发生了智能恢复：ID 已更改。
            updateTasks(tasks => {
                const oldTask = tasks.find(t => t.id === id);
                if (!oldTask) return tasks;

                const newTask: DownloadTask = {
                    ...oldTask,
                    id: newGid,
                    state: 'waiting',
                    progress: 0,
                    speed: 0,
                    remainingSecs: 0,
                };

                return tasks.map(t => t.id === id ? newTask : t);
            });

            addPendingLock(newGid);
            pendingStateChanges.delete(id);

        } else {
            // 标准恢复
            updateTasks(tasks => tasks.map(t =>
                t.id === id ? { ...t, state: 'active' as DownloadState, completedAt: null } : t
            ));
        }

    } catch (e) {
        console.error(`恢复任务 ${id} 失败:`, e);
        pendingStateChanges.delete(id);
    }
}


/**
 * 取消任务（软删除/移除下载）
 */
export async function cancelTask(id: string): Promise<void> {
    // 保存原始状态用于回滚
    let originalState: DownloadState | undefined;

    try {
        // 锁定状态以避免抖动
        addPendingLock(id);

        // 获取原始状态并乐观更新
        updateTasks(tasks => {
            const task = tasks.find(t => t.id === id);
            if (task) {
                originalState = task.state;
            }
            return tasks.map(t =>
                t.id === id ? { ...t, state: 'removed' as DownloadState, completedAt: new Date().toISOString() } : t
            );
        });

        await cancelTaskCmd(id);
    } catch (e) {
        console.error(`Failed to cancel task ${id}:`, e);
        pendingStateChanges.delete(id);

        // 回滚状态
        if (originalState) {
            updateTasks(tasks => tasks.map(t =>
                t.id === id ? { ...t, state: originalState! } : t
            ));
        }
    }
}

/**
 * 删除任务（硬删除）
 */
export function removeTask(id: string, deleteFile: boolean = false): void {
    updateTasks(tasks => {
        const taskToDelete = tasks.find(t => t.id === id);
        if (!taskToDelete) return tasks;

        if (isActiveTask(taskToDelete.state)) {
            addPendingLock(id);
        }

        removeTaskRecord(id, deleteFile)
            .catch(e => {
                console.error("删除任务失败:", e);
                pendingStateChanges.delete(id);
            });

        return tasks.filter(t => t.id !== id);
    });
}

/**
 * 批量删除任务
 */
export async function removeTasks(ids: Set<string>, deleteFile: boolean = false): Promise<void> {
    const idArray = Array.from(ids);
    try {
        await removeTasksCmd(idArray, deleteFile);
        updateTasks(tasks => tasks.filter(t => !ids.has(t.id)));
    } catch (e) {
        console.error("Batch remove failed", e);
    }
}

/**
 * 批量取消任务
 */
export async function cancelTasks(ids: Set<string>): Promise<void> {
    const idArray = Array.from(ids);
    ids.forEach(id => addPendingLock(id));

    try {
        await cancelTasksCmd(idArray);
        updateTasks(tasks => tasks.map(task =>
            ids.has(task.id) && isActiveTask(task.state)
                ? { ...task, state: 'removed' as DownloadState }
                : task
        ));
    } catch (e) {
        console.error("Batch cancel failed", e);
        ids.forEach(id => pendingStateChanges.delete(id));
    }
}

/**
 * 全局暂停所有下载中的任务
 */
export async function pauseAll(): Promise<void> {
    const snapshot = get({ subscribe: subscribeTasks });
    const affectedIds: string[] = [];

    try {
        updateTasks(tasks => {
            tasks.forEach(t => {
                if (isDownloadingTask(t.state) || isWaitingTask(t.state)) {
                    addPendingLock(t.id);
                    affectedIds.push(t.id);
                }
            });
            const now = new Date().toISOString();
            return tasks.map(t =>
                (t.state === 'active' || t.state === 'waiting')
                    ? { ...t, state: 'paused' as DownloadState, speed: 0, completedAt: now }
                    : t
            );
        });

        await pauseAllTasks();
    } catch (e) {
        console.error("Pause all failed", e);
        affectedIds.forEach(id => pendingStateChanges.delete(id));
        setTasks(snapshot);
    }
}

/**
 * 全局恢复所有暂停的任务
 */
export async function resumeAll(): Promise<void> {
    const snapshot = get({ subscribe: subscribeTasks });
    const affectedIds: string[] = [];

    try {
        updateTasks(tasks => {
            tasks.forEach(t => {
                if (isPausedTask(t.state)) {
                    addPendingLock(t.id);
                    affectedIds.push(t.id);
                }
            });
            return tasks.map(t =>
                isPausedTask(t.state)
                    ? { ...t, state: 'waiting' as DownloadState, completedAt: null }
                    : t
            );
        });

        await resumeAllTasks();
    } catch (e) {
        console.error("Resume all failed", e);
        affectedIds.forEach(id => pendingStateChanges.delete(id));
        setTasks(snapshot);
    }
}

/**
 * 判断是否有正在下载的任务
 */
export function hasDownloadingTasks(tasks: DownloadTask[]): boolean {
    return tasks.some(t => isDownloadingTask(t.state) || isWaitingTask(t.state));
}

/**
 * 判断是否有暂停的任务
 */
export function hasPausedTasks(tasks: DownloadTask[]): boolean {
    return tasks.some(t => isPausedTask(t.state));
}
