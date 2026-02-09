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

// 乐观更新锁：记录正在进行操作的任务 ID
// 后端推送时会自动清除锁
const pendingStateChanges = new Set<string>();

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
    updateTasks(currentTasks => {
        // 后端是单一事实来源（Single Source of Truth）
        // 仅对被锁定的任务保留本地状态（乐观更新）
        return backendTasks.map(task => {
            if (pendingStateChanges.has(task.id)) {
                const existing = currentTasks.find(t => t.id === task.id);
                if (existing) {
                    // 后端已推送，清除锁
                    pendingStateChanges.delete(task.id);
                    // 保留本地状态一次，下次推送使用后端数据
                    return { ...task, state: existing.state };
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

        // 计算总速度（使用来自后端的原始 u64 值）
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
 * 暂停任务
 */
export async function pauseTask(id: string): Promise<void> {
    // 保存原始状态用于回滚
    let originalState: DownloadState | undefined;

    try {
        // 锁定状态以避免抖动
        pendingStateChanges.add(id);

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
 * 对于 'cancelled'/'error' 任务：重新添加任务 (Retry)
 */
export async function resumeTask(id: string): Promise<void> {
    try {
        // 锁定状态以防止 UI 闪烁
        pendingStateChanges.add(id);

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
                    speed: '0.00|B/s',
                    speed_u64: 0,
                    remaining: '',
                    // 重置统计信息，因为它是一个全新的下载
                };

                // 用新任务替换旧任务
                return tasks.map(t => t.id === id ? newTask : t);
            });

            // 锁定新 ID 以防止抖动，直到后端赶上
            pendingStateChanges.add(newGid);
            // 同时删除旧 ID 的锁定，以防万一
            pendingStateChanges.delete(id);

        } else {
            // 标准恢复
            // 乐观更新 - 假设变回 downloading
            updateTasks(tasks => tasks.map(t =>
                t.id === id ? { ...t, state: 'downloading' } : t
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
        pendingStateChanges.add(id);

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

        // 后端 (remove_task_record -> remove_task_inner) 现在处理：
        // 1. 确定任务是否活跃。
        // 2. 如果活跃：取消 -> 清理。
        // 3. 如果不活跃：清理。
        // 4. 文件删除逻辑。
        // 所以我们可以直接调用。

        // 如果任务是活跃的，锁定 UI 以防止抖动
        if (isActiveTask(taskToDelete.state)) {
            pendingStateChanges.add(id);
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
    ids.forEach(id => pendingStateChanges.add(id));

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
        updateTasks(tasks => {
            tasks.forEach(t => {
                if (t.state === 'downloading' || t.state === 'waiting') {
                    pendingStateChanges.add(t.id);
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
        updateTasks(tasks => {
            tasks.forEach(t => {
                if (t.state === 'paused') {
                    pendingStateChanges.add(t.id);
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
