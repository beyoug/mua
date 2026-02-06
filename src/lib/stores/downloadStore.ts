/**
 * downloadStore.ts - 集中式下载任务状态管理
 * 
 * 职责：
 * 1. 管理下载任务数组
 * 2. 提供任务 CRUD 操作方法
 * 3. 导出筛选后的任务列表 (active/completed/all)
 * 4. 导出统计信息 (速度、任务数量)
 * 5. 实现任务自动跳转逻辑
 */

import { writable, derived, get, type Readable } from 'svelte/store';
import type { DownloadTask, DownloadConfig, DownloadStats, DownloadState } from '$lib/types/download';
import {
    isActiveTask,
    isCompletedTask,
} from '$lib/utils/downloadStates';
import {
    formatSpeed,
    formatAddedAt,
    extractFilenameFromUrl
} from '$lib/utils/formatters';
import {
    getTasks,
    addDownloadTask as addDownloadTaskCmd,
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

// 状态变更锁：ID -> Timestamp
// 用于在此时间内忽略后端的旧状态，防止 UI 闪烁
const STATE_LOCK_TIMEOUT_MS = 400;
const pendingStateChanges = new Map<string, number>();

// Event Listener for Backend Push
// 替换轮询：监听后端 'tasks-update' 事件
import { listen } from '@tauri-apps/api/event';
import type { UnlistenFn } from '@tauri-apps/api/event';
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';

let unlistenFn: UnlistenFn | null = null;
let unlistenNotificationFn: UnlistenFn | null = null;

async function setupEventListener() {
    if (unlistenFn && unlistenNotificationFn) return;

    // Initial fetch to populate data immediately (before first event)
    try {
        const rawTasks = await getTasks();
        handleTasksUpdate(rawTasks);
    } catch (e) {
        console.error("Initial task fetch failed:", e);
    }

    try {
        unlistenFn = await listen<any[]>('tasks-update', (event) => {
            const rawTasks = event.payload;
            handleTasksUpdate(rawTasks);
        });

        // Listen for completion notifications
        unlistenNotificationFn = await listen<DownloadTask>('task-completed', async (event) => {
            const task = event.payload;

            // Check and request notification permission
            let permissionGranted = await isPermissionGranted();
            if (!permissionGranted) {
                const permission = await requestPermission();
                permissionGranted = permission === 'granted';
            }

            if (permissionGranted) {
                sendNotification({
                    title: '下载完成',
                    body: `${task.filename} 已下载完成`,
                });
            }
        });
    } catch (e) {
        console.error("Failed to setup event listener:", e);
    }
}

// 统一的处理逻辑
function handleTasksUpdate(backendTasks: DownloadTask[]) {
    updateTasks(currentTasks => {
        // Backend is the Single Source of Truth.
        // We only apply local overrides if there's a pending state change lock (to prevent UI jitter).

        // Check if any state locks are active - if so, preserve current order
        const hasActiveLocks = Array.from(pendingStateChanges.entries()).some(
            ([_, lockTime]) => Date.now() - lockTime < STATE_LOCK_TIMEOUT_MS
        );

        // Clean up expired locks
        const now = Date.now();
        for (const [id, lockTime] of pendingStateChanges.entries()) {
            if (now - lockTime >= STATE_LOCK_TIMEOUT_MS) {
                pendingStateChanges.delete(id);
            }
        }

        // Create backend task map for quick lookup
        const backendMap = new Map<string, DownloadTask>();
        for (const task of backendTasks) {
            backendMap.set(task.id, task);
        }

        if (hasActiveLocks && currentTasks.length > 0) {
            // Preserve current order, only update data
            const result: DownloadTask[] = [];
            const processedIds = new Set<string>();

            // Update existing tasks in their current order
            for (const currentTask of currentTasks) {
                const backendTask = backendMap.get(currentTask.id);
                if (backendTask) {
                    // Check if this specific task is locked
                    if (pendingStateChanges.has(currentTask.id)) {
                        // Keep local state and order
                        result.push(currentTask);
                    } else {
                        // Use backend data but keep position
                        result.push(backendTask);
                    }
                    processedIds.add(currentTask.id);
                }
                // If task not in backend anymore, skip it (removed)
            }

            // Add any new tasks from backend (at end)
            for (const backendTask of backendTasks) {
                if (!processedIds.has(backendTask.id)) {
                    result.push(backendTask);
                }
            }

            return result;
        }

        // No active locks - use backend order directly
        return backendTasks.map(task => {
            // Still check individual locks for state preservation
            if (pendingStateChanges.has(task.id)) {
                const existing = currentTasks.find(t => t.id === task.id);
                if (existing) {
                    return existing;
                }
            }
            return task;
        });
    });
}

// 自动开始监听
if (typeof window !== 'undefined') {
    setupEventListener();
}


// ============ 导出的 Derived Stores ============

/**
 * 活跃任务列表（进行中、等待中、已暂停）
 * Backend already sorts tasks by Score (Desc) -> Time (Desc)
 */
export const activeTasks: Readable<DownloadTask[]> = derived(
    { subscribe: subscribeTasks },
    ($tasks) => $tasks
        .filter(task => isActiveTask(task.state))
);

/**
 * 已完成任务列表
 */
export const completedTasks: Readable<DownloadTask[]> = derived(
    { subscribe: subscribeTasks },
    ($tasks) => $tasks
        .filter(task => isCompletedTask(task.state))
);

/**
 * 所有任务列表（历史记录）
 */
export const allTasks: Readable<DownloadTask[]> = derived(
    { subscribe: subscribeTasks },
    ($tasks) => [...$tasks]
);

/**
 * 下载统计信息
 */
export const downloadStats: Readable<DownloadStats> = derived(
    { subscribe: subscribeTasks },
    ($tasks) => {
        const activeDownloads = $tasks.filter(d =>
            ['downloading', 'waiting'].includes(d.state)
        );
        const completedDownloads = $tasks.filter(d => isCompletedTask(d.state));

        // 计算总速度 (Using raw u64 from backend)
        const totalSpeedBytes = activeDownloads
            .map(d => d.speed_u64 || 0)
            .reduce((a, b) => a + b, 0);

        return {
            totalSpeed: formatSpeed(totalSpeedBytes),
            totalSpeedBytes,
            activeCount: activeDownloads.length,
            completedCount: completedDownloads.length
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
                speed: '0.00|B/s',
                speed_u64: 0,
                downloaded: '0 B',
                downloaded_u64: 0,
                total: '?',
                total_u64: 0,
                remaining: '',
                state: 'waiting',
                addedAt: formatAddedAt(),
                savePath: config.savePath
            };

            return [...tasks, newTask];
        });
    } catch (e) {
        console.error('Failed to add task:', e);
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
        pendingStateChanges.set(id, Date.now());

        // 获取并保存原始状态
        updateTasks(tasks => {
            const task = tasks.find(t => t.id === id);
            if (task) {
                originalState = task.state;
            }
            // 乐观更新
            return tasks.map(t =>
                t.id === id ? { ...t, state: 'paused', speed: '0.00|B/s' } : t
            );
        });

        await pauseTaskCmd(id);
    } catch (e) {
        console.error(`Failed to pause task ${id}:`, e);
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
 * 对于 'cancelled'/'error' 任务：重新添加任务 (Retry)
 */
export async function resumeTask(id: string): Promise<void> {
    try {
        // 锁定状态以防止 UI 闪烁
        pendingStateChanges.set(id, Date.now());

        // Call backend resume - now handles Smart Logic (auto-resubmit if needed)
        // returns new GID if smart resume happened, or "OK" (gid) if unpaused.
        const newGid = await resumeTaskCmd(id);

        if (newGid && newGid !== id) {
            // Smart resume happened: ID changed.
            // Instead of just removing old ID, we should OPTIMISTICALLY replace it with new ID
            // to prevent UI flickering (task disappearing then reappearing).
            updateTasks(tasks => {
                const oldTask = tasks.find(t => t.id === id);
                if (!oldTask) return tasks; // Should not happen

                const newTask: DownloadTask = {
                    ...oldTask,
                    id: newGid,
                    state: 'waiting', // or downloading
                    progress: 0,
                    speed: '0.00|B/s',
                    speed_u64: 0,
                    remaining: '',
                    // Reset stats as it's a fresh download
                };

                // Replace old task with new task
                return tasks.map(t => t.id === id ? newTask : t);
            });

            // Lock the NEW ID to prevent jitter until backend catches up
            pendingStateChanges.set(newGid, Date.now());
            // Also remove lock for old ID just in case
            pendingStateChanges.delete(id);

        } else {
            // standard resume
            // 乐观更新 - 假设变回 downloading
            updateTasks(tasks => tasks.map(t =>
                t.id === id ? { ...t, state: 'downloading' } : t
            ));
        }

    } catch (e) {
        console.error(`Failed to resume task ${id}:`, e);
        // Error handling if backend Smart Resume also fails
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
        pendingStateChanges.set(id, Date.now());

        // 获取原始状态并乐观更新
        updateTasks(tasks => {
            const task = tasks.find(t => t.id === id);
            if (task) {
                originalState = task.state;
            }
            return tasks.map(t =>
                t.id === id ? { ...t, state: 'cancelled' } : t
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

        // Backend (remove_task_record -> remove_task_inner) now handles:
        // 1. Determining if task is active.
        // 2. If active: cancel -> purge.
        // 3. If inactive: purge.
        // 4. File deletion logic.
        // So we can blindly call it.

        // Lock UI if it was active to prevent jitter
        if (isActiveTask(taskToDelete.state)) {
            pendingStateChanges.set(id, Date.now());
        }

        removeTaskRecord(id, deleteFile)
            .catch(e => {
                console.error("Remove task failed:", e);
                // Unlock on error
                pendingStateChanges.delete(id);
            });

        // Optimistically remove from local list
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
    const now = Date.now();
    ids.forEach(id => pendingStateChanges.set(id, now));

    try {
        await cancelTasksCmd(idArray);
        // 乐观更新
        updateTasks(tasks => tasks.map(task =>
            ids.has(task.id) && isActiveTask(task.state)
                ? { ...task, state: 'cancelled' }
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
    try {
        // 获取所有需要暂停的任务 ID 并锁定状态
        const now = Date.now();
        updateTasks(tasks => {
            tasks.forEach(t => {
                if (t.state === 'downloading' || t.state === 'waiting') {
                    pendingStateChanges.set(t.id, now);
                }
            });
            // 乐观更新：所有 active -> paused
            return tasks.map(t =>
                (t.state === 'downloading' || t.state === 'waiting')
                    ? { ...t, state: 'paused', speed: '0.00|B/s' }
                    : t
            );
        });

        await pauseAllTasks();
    } catch (e) {
        console.error("Pause all failed", e);
    }
}

/**
 * 全局恢复所有暂停的任务
 */
export async function resumeAll(): Promise<void> {
    try {
        // 获取所有需要恢复的任务 ID 并锁定状态
        const now = Date.now();
        updateTasks(tasks => {
            tasks.forEach(t => {
                if (t.state === 'paused') {
                    pendingStateChanges.set(t.id, now);
                }
            });
            // 乐观更新：所有 paused -> waiting
            return tasks.map(t =>
                t.state === 'paused'
                    ? { ...t, state: 'waiting' }
                    : t
            );
        });

        await resumeAllTasks();
    } catch (e) {
        console.error("Resume all failed", e);
    }
}

/**
 * 判断是否有正在下载的任务
 */
export function hasDownloadingTasks(tasks: DownloadTask[]): boolean {
    return tasks.some(t => t.state === 'downloading' || t.state === 'waiting');
}

/**
 * 判断是否有暂停的任务
 */
export function hasPausedTasks(tasks: DownloadTask[]): boolean {
    return tasks.some(t => t.state === 'paused');
}
