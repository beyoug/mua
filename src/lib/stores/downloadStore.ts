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
import { appSettings } from '$lib/stores/settings';
import {
    isActiveTask,
    isCompletedTask,
    isRemovableTask,
    getStateScore
} from '$lib/utils/downloadStates';
import {
    parseSpeedToBytes,
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
    updateTrayIconSpeed
} from '$lib/api/cmd';
import type { Aria2Task } from '$lib/types/aria2';

// ...
// ...
// 托盘速度更新逻辑已移至轮询函数

// Aria2File 和 Aria2Uri 现在是 Aria2Task 类型的一部分，已导入。
// 不需要本地定义。

function mapAria2Status(status: string): DownloadState {
    switch (status) {
        case 'active': return 'downloading';
        case 'waiting': return 'waiting';
        case 'paused': return 'paused';
        case 'complete': return 'completed';
        case 'error': return 'error';
        case 'removed': return 'cancelled';
        default: return 'waiting';
    }
}


// ============ 内部状态 ============

// 所有下载任务（主数据源）
const { subscribe: subscribeTasks, set: setTasks, update: updateTasks } = writable<DownloadTask[]>([]);

let pollingInterval: number | null = null;
let isPolling = false;

// 状态变更锁：ID -> Timestamp
// 用于在此时间内忽略后端的旧状态，防止 UI 闪烁
const pendingStateChanges = new Map<string, number>();

// 启动轮询
async function startPolling() {
    if (isPolling) return;
    isPolling = true;

    const poll = async () => {
        try {
            const rawTasks = await getTasks();
            updateTasks(currentTasks => {
                // 合并策略：
                // 1. 优先使用后端数据。
                // 2. 尝试保留本地存储中的 'addedAt' 和 'filename'（如果用户重命名了）。
                //    实际上，文件名在 Aria2 下载开始后可能会更新（元数据）。

                const merged: DownloadTask[] = rawTasks.map(rt => {
                    const existing = currentTasks.find(t => t.id === rt.gid);

                    // 计算基本统计信息
                    const total = parseInt(rt.totalLength, 10);
                    const completed = parseInt(rt.completedLength, 10);
                    const speed = parseInt(rt.downloadSpeed, 10);
                    let progress = 0;
                    if (total > 0) {
                        progress = (completed / total) * 100;
                    }

                    // 剩余时间计算
                    let remaining = '';
                    if (speed > 0 && total > completed) {
                        const seconds = (total - completed) / speed;
                        if (seconds > 3600) remaining = `${(seconds / 3600).toFixed(1)}h`;
                        else if (seconds > 60) remaining = `${(seconds / 60).toFixed(1)}m`;
                        else remaining = `${Math.ceil(seconds)}s`;
                    }

                    //文件名
                    // 如果 rt.files[0].path 非空，使用 basename。
                    // 否则使用现有的或回退到 url。
                    let filename = existing?.filename || 'Unknown';
                    if (rt.files && rt.files.length > 0 && rt.files[0].path) {
                        // 提取文件名
                        const path = rt.files[0].path;
                        // 处理 unix 和 windows 路径以防万一，虽然我们在 Mac 上
                        const parts = path.split(/[/\\]/);
                        if (parts.length > 0 && parts[parts.length - 1]) {
                            filename = parts[parts.length - 1];
                        }
                    }

                    // 带锁逻辑的状态判定
                    // 如果此任务有挂起的本地更改，验证是否应完全忽略后端
                    if (pendingStateChanges.has(rt.gid)) {
                        const lockTime = pendingStateChanges.get(rt.gid)!;
                        if (Date.now() - lockTime < 1000) {
                            // 锁激活：返回现有的本地状态以防止抖动/回退
                            if (existing) {
                                return existing;
                            }
                        } else {
                            // 锁过期
                            pendingStateChanges.delete(rt.gid);
                        }
                    }

                    return {
                        id: rt.gid,
                        filename: filename,
                        url: rt.files[0]?.uris[0]?.uri || '',
                        progress: progress,
                        speed: formatSpeed(speed),
                        downloaded: formatBytes(completed),
                        total: formatBytes(total),
                        remaining: remaining,
                        state: mapAria2Status(rt.status),
                        addedAt: existing?.addedAt || formatAddedAt(), // Fallback to now if new
                        savePath: rt.dir
                    }
                });

                // 保留已取消/已移除的任务？
                // Aria2 'removed' 状态是短暂的。如果我们需要历史记录，可能需要在它们从后端消失时保留它们
                // 并且之前在本地存储中处于 'completed' 或 'cancelled' 状态。
                // 对于消失的 'active'/'waiting'/'paused' 任务，它们可能已被移除。

                // 对于具体需求："历史记录"
                // Aria2 tellStopped 返回已停止（已完成/错误）的任务。
                // 因此合并列表应包含 Aria2 知道的所有内容。
                // 如果任务从 UI（以及 Aria2）手动移除，它就消失了。
                // 如果我们想在本地保留已移除任务的历史记录，我们需要单独的逻辑。
                // 目前，让我们与 Aria2 保持同步。

                // 提取后端任务ID集合
                const backendIds = new Set(rawTasks.map(t => t.gid));

                // 2. 识别并保留本地存在但后端已消失的任务
                // 这些通常是已被取消(Removed)的任务，我们需要将其保留为历史记录
                const retainedTasks: DownloadTask[] = [];
                currentTasks.forEach(localTask => {
                    if (!backendIds.has(localTask.id)) {
                        // 如果它之前是一个活跃任务(下载中/等待/暂停)，现在不见了 -> 视为被取消
                        if (['downloading', 'waiting', 'paused'].includes(localTask.state)) {
                            // 检查是否已有挂起的删除操作 Pending？
                            // 如果用户请求了 purge (removeTaskRecord)，那么它应该彻底消失。
                            // removeTask 函数会更新本地 store 移除它。
                            // 但在这里我们是基于上一帧的 store。
                            // 最简单的方式：保留它，标记为 cancelled。
                            retainedTasks.push({
                                ...localTask,
                                state: 'cancelled',
                                speed: '0 B/s',
                                remaining: ''
                            });
                        }
                        // 如果已经是 completed, error, cancelled，也应该保留作为历史
                        else if (['completed', 'error', 'cancelled'].includes(localTask.state)) {
                            retainedTasks.push(localTask);
                        }
                    }
                });

                // 3. 合并列表 (后端新状态 + 本地保留的历史)
                // 注意去重：backendIds 中的任务已经在 merged Map 中处理过？
                // rawTasks.map 生成了 merged 数组，包含了所有后端任务。
                // retainedTasks 包含所有后端缺失的任务。
                // 两者互斥，直接拼接即可。

                return [...merged, ...retainedTasks];
            });

            // 更新托盘速度（双向）
            let totalDownload = 0;
            let totalUpload = 0;

            rawTasks.forEach(t => {
                const dl = parseInt(t.downloadSpeed, 10);
                // t.uploadSpeed 可能未定义，如果后端结构尚未同步（或 json 缺失）
                // 使用安全解析
                const ul = t.uploadSpeed ? parseInt(t.uploadSpeed, 10) : 0;

                if (!isNaN(dl)) totalDownload += dl;
                if (!isNaN(ul)) totalUpload += ul;
            });

            // 更新托盘速度（动态图标）
            await updateTrayIconSpeed(totalDownload, totalUpload);

        } catch (e) {
            console.error('Failed to sync tasks:', e);
        }

        if (isPolling) {
            // 自适应轮询策略
            // 如果隐藏：5s，如果可见：1s
            const interval = (typeof document !== 'undefined' && document.visibilityState === 'hidden')
                ? 5000
                : 1000;
            pollingInterval = setTimeout(poll, interval) as unknown as number;
        }
    };

    poll();

    // 监听可见性变化以获得即时响应
    if (typeof document !== 'undefined') {
        document.addEventListener('visibilitychange', () => {
            if (document.visibilityState === 'visible' && isPolling) {
                if (pollingInterval) clearTimeout(pollingInterval);
                poll();
            }
        });
    }
}

function stopPolling() {
    isPolling = false;
    if (pollingInterval) {
        clearTimeout(pollingInterval);
        pollingInterval = null;
    }
}

// 自动开始轮询
if (typeof window !== 'undefined') {
    startPolling();
}


// ============ 导出的 Derived Stores ============

/**
 * 活跃任务列表（进行中、等待中、已暂停）
 */
export const activeTasks: Readable<DownloadTask[]> = derived(
    { subscribe: subscribeTasks },
    ($tasks) => $tasks
        .filter(task => isActiveTask(task.state))
        .sort(sortByStateAndTime)
);

/**
 * 已完成任务列表
 */
export const completedTasks: Readable<DownloadTask[]> = derived(
    { subscribe: subscribeTasks },
    ($tasks) => $tasks
        .filter(task => isCompletedTask(task.state))
        .sort(sortByStateAndTime)
);

/**
 * 所有任务列表（历史记录）
 */
export const allTasks: Readable<DownloadTask[]> = derived(
    { subscribe: subscribeTasks },
    ($tasks) => [...$tasks].sort(sortByStateAndTime)
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

        // 计算总速度
        const totalSpeedBytes = activeDownloads
            .map(d => parseSpeedToBytes(d.speed || ''))
            .reduce((a, b) => a + b, 0);

        return {
            totalSpeed: formatSpeed(totalSpeedBytes),
            totalSpeedBytes,
            activeCount: activeDownloads.length,
            completedCount: completedDownloads.length
        };
    }
);

// ============ 工具函数 ============

/**
 * 任务排序函数
 * 1. 按状态优先级排序（下载中 > 暂停 > 其他）
 * 2. 按添加时间倒序（最新的在前）
 */
function sortByStateAndTime(a: DownloadTask, b: DownloadTask): number {
    const scoreA = getStateScore(a.state);
    const scoreB = getStateScore(b.state);

    if (scoreA !== scoreB) {
        return scoreB - scoreA;
    }

    const timeA = a.addedAt || '';
    const timeB = b.addedAt || '';
    return timeB.localeCompare(timeA);
}

// ============ 任务操作方法 ============

// formatBytes 的辅助函数（如果未导入）
function formatBytes(bytes: number, decimals = 2) {
    if (!+bytes) return '0 B';
    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`;
}


/**
 * 添加下载任务
 */
export async function addDownloadTask(config: DownloadConfig): Promise<void> {
    try {
        const gid = await addDownloadTaskCmd(config);

        // 乐观 UI 更新或仅等待下一次轮询
        // 让我们添加一个占位符以感觉响应迅速
        updateTasks(tasks => {
            const newTasks: DownloadTask[] = config.urls.map(url => ({
                id: gid, // 使用返回的 GID！
                filename: config.filename || extractFilenameFromUrl(url),
                url: url,
                progress: 0,
                state: 'waiting',
                addedAt: formatAddedAt()
            }));

            return [...tasks, ...newTasks];
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
    try {
        // 锁定状态以避免抖动
        pendingStateChanges.set(id, Date.now());

        await pauseTaskCmd(id);
        updateTasks(tasks => tasks.map(t =>
            t.id === id ? { ...t, state: 'paused', speed: '0 B/s' } : t
        ));
    } catch (e) {
        console.error(`Failed to pause task ${id}:`, e);
        pendingStateChanges.delete(id); // 发生错误时解锁
    }
}

/**
 * 恢复任务
 * 对于 'paused' 任务：调用 aria2.unpause
 * 对于 'cancelled'/'error' 任务：重新添加任务 (Retry)
 */
export async function resumeTask(id: string): Promise<void> {
    try {
        // 1. 获取当前任务详情以进行决策
        // 此时我们只能从本地 store 获取状态，因为后端可能已经没有这个任务了
        let task: DownloadTask | undefined;
        const unsubscribe = subscribeTasks(tasks => {
            task = tasks.find(t => t.id === id);
        });
        unsubscribe();

        if (!task) {
            console.error(`Cannot resume task ${id}: Task not found in local store`);
            return;
        }

        // 锁定状态以防止 UI 闪烁
        pendingStateChanges.set(id, Date.now());

        if (task.state === 'cancelled' || task.state === 'error' || task.state === 'completed') {
            // 2. 如果任务已结束（被取消/出错/完成），Aria2 可能已遗忘 GID。
            // 我们必须重新添加任务。
            console.log(`Resuming (Restarting) task ${id} from state ${task.state}`);

            // 构建配置
            const config: DownloadConfig = {
                urls: [task.url || ''],
                filename: task.filename || '',
                savePath: task.savePath || '',
                userAgent: '',
                referer: '',
                headers: '',
                proxy: '',
                maxDownloadLimit: ''
            };

            await addDownloadTask(config);

            // 重新添加后，我们会得到一个新的 GID。
            // 旧的 'cancelled' 任务记录怎么办？
            // 1. 保留它作为历史？
            // 2. 删除它（替换为新的）？
            // 通常 "重试" 意味着替换。
            // 我们可以手动移除旧记录。
            removeTask(id, false);

        } else {
            // 3. 这里的状态通常是 'paused'，直接调用后端恢复
            await resumeTaskCmd(id);
            // 乐观更新
            updateTasks(tasks => tasks.map(t =>
                t.id === id ? { ...t, state: 'downloading' } : t
            ));
        }

    } catch (e) {
        console.error(`Failed to resume task ${id}:`, e);

        // 如果错误是 "GID check failed" 或类似，且任务在本地存在，则尝试 Retry
        // 简单起见，如果 unpause 失败，且不是网络错误，我们可以推断 GID 无效
        // 尝试 fallback 到重试逻辑
        const errStr = String(e);
        if (errStr.includes('is not found') || errStr.includes('GID')) {
            console.log(`Resume failed for ${id}, assuming GID lost. Retrying...`);
            let task: DownloadTask | undefined;
            const unsubscribe = subscribeTasks(tasks => {
                task = tasks.find(t => t.id === id);
            });
            unsubscribe();

            if (task) {
                // 复用上面的 Retry 逻辑
                // Copy-paste 逻辑提取? 暂时内联
                const config: DownloadConfig = {
                    urls: [task.url || ''],
                    filename: task.filename || '',
                    savePath: task.savePath || '',
                    userAgent: '',
                    referer: '',
                    headers: '',
                    proxy: '',
                    maxDownloadLimit: ''
                };
                await addDownloadTask(config).catch(err => console.error("Retry failed", err));
                removeTask(id, false);
                return;
            }
        }

        pendingStateChanges.delete(id);
    }
}


/**
 * 取消任务（软删除/移除下载）
 */
export async function cancelTask(id: string): Promise<void> {
    try {
        // 锁定状态以避免抖动 (防止轮询在 Aria2 处理完成前覆盖本地状态)
        pendingStateChanges.set(id, Date.now());

        await cancelTaskCmd(id);
        // 对于取消，aria2 将状态更改为已移除或错误。轮询将获取它。
        // 但我们可以乐观地将其标记为已取消或根据需要将其从活动列表中移除。
        // UI 通常期望在 'active' 视图中进行软删除的 'cancelled' 状态？
        // 实际上，'cancel_task' 调用 'aria2.remove'。
        // 我们将让轮询处理状态更改或在本地更新。
        updateTasks(tasks => tasks.map(t =>
            t.id === id ? { ...t, state: 'cancelled' } : t
        ));
    } catch (e) {
        console.error(`Failed to cancel task ${id}:`, e);
        pendingStateChanges.delete(id); // 解锁
    }
}

/**
 * 删除任务（硬删除）
 */
export function removeTask(id: string, deleteFile: boolean = false): void {
    updateTasks(tasks => {
        const taskToDelete = tasks.find(t => t.id === id);
        if (!taskToDelete) return tasks;

        // 计算文件路径（无论状态如何都需要，用于删除文件）
        let filepath: string | null = null;
        if (taskToDelete.savePath && taskToDelete.filename) {
            // 基于假设确定分隔符（或者在 Mac 上直接使用 /）
            if (taskToDelete.savePath.endsWith('/')) {
                filepath = taskToDelete.savePath + taskToDelete.filename;
            } else {
                filepath = taskToDelete.savePath + '/' + taskToDelete.filename;
            }
        }

        if (deleteFile) {
            // 我们不能在这里直接删除，因为逻辑太深。
            // 但我们的 removeTaskRecord 会处理。
        }

        if (isRemovableTask(taskToDelete.state)) {
            // 已完成/已取消/错误的任务 -> Purge (清空记录)
            removeTaskRecord(id, deleteFile, filepath)
                .catch(e => console.error("remove failed", e));
        } else {
            // 进行中的任务 -> Force Cancel (移除任务)
            // 锁定以防轮询再次读取到它 ('active')
            pendingStateChanges.set(id, Date.now());

            cancelTaskCmd(id).catch(e => {
                console.error("Cancel for remove failed", e);
                pendingStateChanges.delete(id);
            });

            // 如果需要删除文件，这可能比较棘手，因为任务还没停。
            // 最好的做法是尝试调用 removeTaskRecord，它内部也是尽力而为。
            // 但 aria2.purge 只能对 stopped 任务生效。
            // 我们可以仅依靠 cancelTaskCmd(aria2.remove) 来停止任务。
            // 对于文件删除：
            if (deleteFile && filepath) {
                // 延迟一点删除？或者现在尝试？
                // 用户希望“删除”，我们应该尽力清理。
                // 如果 aria2 还没释放句柄，可能会失败。
                // 我们尝试调用 removeTaskRecord (它包含文件删除逻辑)
                // 虽然 purge 会失败，但文件删除代码是独立的。
                removeTaskRecord(id, deleteFile, filepath)
                    .catch(e => console.warn("File deletion might fail as task is active", e));
            }
        }

        // 无论状态如何，都从本地列表移除
        return tasks.filter(t => t.id !== id);
    });
}

/**
 * 批量删除任务
 */
export function removeTasks(ids: Set<string>, deleteFile: boolean = false): void {
    ids.forEach(id => removeTask(id, deleteFile));
}

/**
 * 批量取消任务
 */
export function cancelTasks(ids: Set<string>): void {
    // 锁定所有涉及的任务
    const now = Date.now();
    ids.forEach(id => pendingStateChanges.set(id, now));

    // 调用后端取消（这里需要单独遍历调用，因为 aria2 没有 batch cancel）
    // 或者我们可以在 commands.rs 实现 batch cancel
    // 目前简单遍历
    ids.forEach(id => {
        cancelTaskCmd(id).catch(e => {
            console.error(`Failed to cancel task ${id}`, e);
            pendingStateChanges.delete(id);
        });
    });

    updateTasks(tasks => {
        return tasks.map(task =>
            ids.has(task.id) && isActiveTask(task.state)
                ? { ...task, state: 'cancelled' }
                : task
        );
    });
}

/**
 * 全局暂停所有下载中的任务
 */
export function pauseAll(): void {
    // 我们应该遍历活动任务并暂停它们
    // 或者调用批处理方法。目前遍历是最简单的。
    activeTasks.subscribe(tasks => {
        tasks.forEach(t => {
            if (t.state === 'downloading' || t.state === 'waiting') {
                pauseTask(t.id);
            }
        });
    })(); // Subscribe 返回 unsubscribe，但我们只想获取一次当前值？
    // 实际上使用 get(activeTasks) 更好，但我们在模块内部。
    // 更简单的方法：
    updateTasks(tasks => {
        tasks.forEach(t => {
            if (t.state === 'downloading' || t.state === 'waiting') {
                pauseTask(t.id);
            }
        });
        return tasks;
    });
}

/**
 * 全局恢复所有暂停的任务
 */
export function resumeAll(): void {
    updateTasks(tasks => {
        tasks.forEach(t => {
            if (t.state === 'paused') {
                resumeTask(t.id);
            }
        });
        return tasks;
    });
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
