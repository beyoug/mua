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
    addDownloadTask as addDownloadTaskCmd,
    addDownloadTasksCmd,
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
            p.downloaded !== n.downloaded ||
            p.total !== n.total ||
            p.remaining !== n.remaining
        ) return true;
    }
    return false;
}

// 后端推送的事件监听器
// 替换轮询：监听后端 'tasks-update' 事件
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


// ============ 导出的 Derived Stores ============

/**
 * 活跃任务列表（进行中、等待中、已暂停）
 * 后端已经按分数（降序）-> 时间（降序）对任务进行了排序
 */
export const activeTasks: Readable<DownloadTask[]> = derived(
    { subscribe: subscribeTasks },
    ($tasks) => $tasks
        .filter(task => isActiveTask(task.state))
);

/**
 * 已完成任务列表
 */
export const completeTasks: Readable<DownloadTask[]> = derived(
    { subscribe: subscribeTasks },
    ($tasks) => $tasks
        .filter(task => isCompletedTask(task.state))
);

/**
 * 所有任务列表（历史记录）
 */
export const allTasks: Readable<DownloadTask[]> = derived(
    { subscribe: subscribeTasks },
    ($tasks) => $tasks
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

        // 计算总速度（使用来自后端的原始 u64 值）
        const totalSpeedBytes = activeDownloads
            .map(d => d.speed_u64 || 0)
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
 * 添加下载任务
 */
export async function addDownloadTask(config: DownloadConfig): Promise<void> {
    try {
        const gid = await addDownloadTaskCmd(config);

        // 乐观 UI 更新
        updateTasks(tasks => {
            // 防止重复添加（如果后端同步已推送到 store）
            if (tasks.some(t => t.id === gid)) {
                return tasks;
            }

            // 只创建一个代表性任务 (Aria2 GID 对应一个 Task，即使有多个 Mirrors)
            const primaryUrl = config.urls[0] || '';
            const newTask: DownloadTask = {
                id: gid, // 使用返回的 GID！
                filename: config.filename || extractFilenameFromUrl(primaryUrl),
                url: primaryUrl,
                progress: 0,
                speed: { value: '0', unit: 'B/s' },
                speed_u64: 0,
                downloaded: '0 B',
                downloaded_u64: 0,
                total: '?',
                total_u64: 0,
                remaining: '',
                state: 'waiting',
                addedAt: formatAddedAt(),
                savePath: config.savePath || '',
                userAgent: config.userAgent,
                referer: config.referer,
                proxy: config.proxy,
                maxDownloadLimit: config.maxDownloadLimit,
                headers: config.headers ? config.headers.split(';').map(h => h.trim()).filter(h => h !== '') : []
            };

            return [...tasks, newTask];
        });
    } catch (e) {
        console.error('添加任务失败:', e);
        throw e;
    }
}

/**
 * 批量添加下载任务
 */
export async function addBatchDownloadTasks(configs: DownloadConfig[]): Promise<void> {
    try {
        const results = await addDownloadTasksCmd(configs);

        updateTasks(tasks => {
            const newTasks: DownloadTask[] = [];

            for (let i = 0; i < configs.length; i++) {
                const gid = results[i];
                if (!gid) continue;

                // Prevent duplicates
                if (tasks.some(t => t.id === gid) || newTasks.some(t => t.id === gid)) continue;

                const config = configs[i];
                const primaryUrl = config.urls[0] || '';

                const newTask: DownloadTask = {
                    id: gid,
                    filename: config.filename || extractFilenameFromUrl(primaryUrl),
                    url: primaryUrl,
                    progress: 0,
                    speed: { value: '0', unit: 'B/s' },
                    speed_u64: 0,
                    downloaded: '0 B',
                    downloaded_u64: 0,
                    total: '?',
                    total_u64: 0,
                    remaining: '',
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
        console.error('Batch add failed:', e);
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
                t.id === id ? { ...t, state: 'paused', speed: { value: '0', unit: 'B/s' }, completedAt: new Date().toISOString() } : t
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

        // 调用后端恢复 - 现在处理智能逻辑（如果需要则自动重新提交）
        // 如果发生了智能恢复，返回新的 GID；如果是取消暂停，返回 "OK" (gid)。
        const newGid = await resumeTaskCmd(id);

        if (newGid && newGid !== id) {
            // 发生了智能恢复：ID 已更改。
            // 不仅仅是移除旧 ID，我们应该乐观地将其替换为新 ID
            // 以防止 UI 闪烁（任务消失后又重新出现）。
            updateTasks(tasks => {
                const oldTask = tasks.find(t => t.id === id);
                if (!oldTask) return tasks; // 不应该发生

                const newTask: DownloadTask = {
                    ...oldTask,
                    id: newGid,
                    state: 'waiting', // 或 downloading
                    progress: 0,
                    speed: { value: '0', unit: 'B/s' },
                    speed_u64: 0,
                    remaining: '',
                    // 重置统计信息，因为它是一个全新的下载
                };

                // 用新任务替换旧任务
                return tasks.map(t => t.id === id ? newTask : t);
            });

            // 锁定新 ID 以防止抖动，直到后端赶上
            addPendingLock(newGid);
            // 同时删除旧 ID 的锁定，以防万一
            pendingStateChanges.delete(id);

        } else {
            // 标准恢复
            // 乐观更新 - 假设变回 active
            updateTasks(tasks => tasks.map(t =>
                t.id === id ? { ...t, state: 'active', completedAt: null } : t
            ));
        }

    } catch (e) {
        console.error(`恢复任务 ${id} 失败:`, e);
        // 如果后端智能恢复也失败，进行错误处理
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
        // 锁定状态以避免抖动 (防止轮询在 Aria2 处理完成前覆盖本地状态)
        addPendingLock(id);

        // 获取原始状态并乐观更新
        updateTasks(tasks => {
            const task = tasks.find(t => t.id === id);
            if (task) {
                originalState = task.state;
            }
            return tasks.map(t =>
                t.id === id ? { ...t, state: 'removed', completedAt: new Date().toISOString() } : t
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

        // 后端 (remove_task_record -> remove_task_inner) 现在处理：
        // 1. 确定任务是否活跃。
        // 2. 如果活跃：取消 -> 清理。
        // 3. 如果不活跃：清理。
        // 4. 文件删除逻辑。
        // 所以我们可以直接调用。

        // 如果任务是活跃的，锁定 UI 以防止抖动
        if (isActiveTask(taskToDelete.state)) {
            addPendingLock(id);
        }

        removeTaskRecord(id, deleteFile)
            .catch(e => {
                console.error("删除任务失败:", e);
                // 出错时解锁
                pendingStateChanges.delete(id);
            });

        // 乐观地从本地列表中移除
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
        // 乐观更新
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
    // 锁定状态
    ids.forEach(id => addPendingLock(id));

    try {
        await cancelTasksCmd(idArray);
        // 乐观更新
        updateTasks(tasks => tasks.map(task =>
            ids.has(task.id) && isActiveTask(task.state)
                ? { ...task, state: 'removed' }
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
    // 保存原始状态用于回滚
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
            // 乐观更新：所有 active -> paused
            const now = new Date().toISOString();
            return tasks.map(t =>
                (t.state === 'active' || t.state === 'waiting')
                    ? { ...t, state: 'paused', speed: { value: '0', unit: 'B/s' }, completedAt: now }
                    : t
            );
        });

        await pauseAllTasks();
    } catch (e) {
        console.error("Pause all failed", e);
        // 回滚到操作前状态
        affectedIds.forEach(id => pendingStateChanges.delete(id));
        setTasks(snapshot);
    }
}

/**
 * 全局恢复所有暂停的任务
 */
export async function resumeAll(): Promise<void> {
    // 保存原始状态用于回滚
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
            // 乐观更新：所有 paused -> waiting
            return tasks.map(t =>
                isPausedTask(t.state)
                    ? { ...t, state: 'waiting', completedAt: null }
                    : t
            );
        });

        await resumeAllTasks();
    } catch (e) {
        console.error("Resume all failed", e);
        // 回滚到操作前状态
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
