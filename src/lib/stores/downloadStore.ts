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

import { writable, derived, type Readable } from 'svelte/store';
import type { DownloadTask, DownloadConfig, DownloadStats, DownloadState } from '$lib/types/download';
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
import { invoke } from '@tauri-apps/api/core';

// Backend Type Definition
interface Aria2Task {
    gid: string;
    status: string; // active, waiting, paused, error, complete, removed
    totalLength: string;
    completedLength: string;
    downloadSpeed: string;
    uploadLength: string;
    uploadSpeed: string;
    errorCode: string | null;
    errorMessage: string | null;
    dir: string;
    files: Aria2File[];
}
// ...
// ...
// Update Tray Speed logic moved to poll function

interface Aria2File {
    index: string;
    path: string;
    length: string;
    completedLength: string;
    selected: string;
    uris: Aria2Uri[];
}

interface Aria2Uri {
    uri: string;
    status: string;
}

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
            const rawTasks = await invoke<Aria2Task[]>('get_tasks');
            updateTasks(currentTasks => {
                // Merge strategy:
                // 1. We prioritize backend data.
                // 2. We try to preserve 'addedAt' and 'filename' (if user renamed it) from local store if possible.
                //    But actually, filename in Aria2 might be updated after download starts (metadata).

                const merged: DownloadTask[] = rawTasks.map(rt => {
                    const existing = currentTasks.find(t => t.id === rt.gid);

                    // Calculate basic stats
                    const total = parseInt(rt.totalLength, 10);
                    const completed = parseInt(rt.completedLength, 10);
                    const speed = parseInt(rt.downloadSpeed, 10);
                    let progress = 0;
                    if (total > 0) {
                        progress = (completed / total) * 100;
                    }

                    // Remaining time calculation
                    let remaining = '';
                    if (speed > 0 && total > completed) {
                        const seconds = (total - completed) / speed;
                        if (seconds > 3600) remaining = `${(seconds / 3600).toFixed(1)}h`;
                        else if (seconds > 60) remaining = `${(seconds / 60).toFixed(1)}m`;
                        else remaining = `${Math.ceil(seconds)}s`;
                    }

                    // Filename
                    // If rt.files[0].path is non-empty, use basename.
                    // Otherwise use existing or fallback to url.
                    let filename = existing?.filename || 'Unknown';
                    if (rt.files && rt.files.length > 0 && rt.files[0].path) {
                        // Extract basename
                        const path = rt.files[0].path;
                        // Handle both unix and windows paths just in case, though we are on mac
                        const parts = path.split(/[/\\]/);
                        if (parts.length > 0 && parts[parts.length - 1]) {
                            filename = parts[parts.length - 1];
                        }
                    }

                    // State determination with locking logic
                    // If we have a pending local change for this task, verify if we should ignore backend entirely
                    if (pendingStateChanges.has(rt.gid)) {
                        const lockTime = pendingStateChanges.get(rt.gid)!;
                        if (Date.now() - lockTime < 1000) {
                            // Lock active: Return existing local state to prevent jitter/reversion
                            if (existing) {
                                return existing;
                            }
                        } else {
                            // Lock expired
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

                // Keep cancelled/removed tasks? 
                // Aria2 'removed' status is transient. If we want history, we might need to keep them if they are missing from backend 
                // AND were previously in 'completed' or 'cancelled' state in local store.
                // For 'active'/'waiting'/'paused' tasks that disappear, they are likely removed.

                // For specific requirements: "History"
                // Aria2 tellStopped returns stopped (completed/error) tasks.
                // So merged list should contain everything Aria2 knows about.
                // If a task was manually removed from UI (and thus Aria2), it's gone.
                // If we want to keep history of removed tasks locally, we need separate logic.
                // For now, let's sync with Aria2.

                return merged;
            });

            // Update Tray Speed (Bidirectional)
            let totalDownload = 0;
            let totalUpload = 0;

            rawTasks.forEach(t => {
                const dl = parseInt(t.downloadSpeed, 10);
                // t.uploadSpeed might be undefined if backend struct not yet synced (or if json missing)
                // Use safe parsing
                const ul = t.uploadSpeed ? parseInt(t.uploadSpeed, 10) : 0;

                if (!isNaN(dl)) totalDownload += dl;
                if (!isNaN(ul)) totalUpload += ul;
            });

            // Format: "↓ 2.5 MB/s  ↑ 10 KB/s"
            const dlStr = formatSpeed(totalDownload);
            const ulStr = formatSpeed(totalUpload);
            // Use unicode arrows for compactness
            const trayStr = `↓ ${dlStr}  ↑ ${ulStr}`;

            await invoke('update_tray_speed', { speed: trayStr });

        } catch (e) {
            console.error('Failed to sync tasks:', e);
        }

        if (isPolling) {
            pollingInterval = setTimeout(poll, 1000) as unknown as number;
        }
    };

    poll();
}

function stopPolling() {
    isPolling = false;
    if (pollingInterval) {
        clearTimeout(pollingInterval);
        pollingInterval = null;
    }
}

// Auto start polling
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

// Helper for formatBytes if not imported
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
        const gid = await invoke<string>('add_download_task', {
            urls: config.urls,
            savePath: config.savePath,
            filename: config.filename,
            userAgent: config.userAgent,
            referer: config.referer,
            headers: config.headers,
            proxy: config.proxy,
            maxDownloadLimit: config.maxDownloadLimit
        });

        // Optimistic UI update or just wait for next poll
        // Let's add a placeholder to feel responsive
        updateTasks(tasks => {
            const newTasks: DownloadTask[] = config.urls.map(url => ({
                id: gid, // Use the GID returned!
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
        // Lock state to avoid jitter
        pendingStateChanges.set(id, Date.now());

        await invoke('pause_task', { gid: id });
        updateTasks(tasks => tasks.map(t =>
            t.id === id ? { ...t, state: 'paused', speed: '0 B/s' } : t
        ));
    } catch (e) {
        console.error(`Failed to pause task ${id}:`, e);
        pendingStateChanges.delete(id); // Unlock on error
    }
}

/**
 * 恢复任务
 */
export async function resumeTask(id: string): Promise<void> {
    try {
        // Lock state
        pendingStateChanges.set(id, Date.now());

        await invoke('resume_task', { gid: id });
        // Optimistic update
        updateTasks(tasks => tasks.map(t =>
            t.id === id ? { ...t, state: 'downloading' } : t
        ));
    } catch (e) {
        console.error(`Failed to resume task ${id}:`, e);
        pendingStateChanges.delete(id);
    }
}

/**
 * 取消任务（软删除/移除下载）
 */
export async function cancelTask(id: string): Promise<void> {
    try {
        await invoke('cancel_task', { gid: id });
        // For cancel, aria2 changes status to removed or error. Polling will pick it up.
        // But we can optimistically mark it as cancelled or remove it from active list if desired.
        // UI expects 'cancelled' state for soft delete in 'active' view usually?
        // Actually, 'cancel_task' calls 'aria2.remove'. 
        // We'll let polling handle the state change or update locally.
        updateTasks(tasks => tasks.map(t =>
            t.id === id ? { ...t, state: 'cancelled' } : t
        ));
    } catch (e) {
        console.error(`Failed to cancel task ${id}:`, e);
    }
}

/**
 * 删除任务（硬删除）
 */
export function removeTask(id: string, deleteFile: boolean = false): void {
    updateTasks(tasks => {
        const taskToDelete = tasks.find(t => t.id === id);
        if (!taskToDelete) return tasks;

        if (isRemovableTask(taskToDelete.state)) {
            let filepath: string | null = null;
            if (taskToDelete.savePath && taskToDelete.filename) {
                // Determine separator based on assumption (or just use / for mac)
                // We'll trust the backend/Aria2 path format usually.
                // Simple concat for Mac/Linux
                if (taskToDelete.savePath.endsWith('/')) {
                    filepath = taskToDelete.savePath + taskToDelete.filename;
                } else {
                    filepath = taskToDelete.savePath + '/' + taskToDelete.filename;
                }
            }

            if (deleteFile) {
                // console.log(`[DEBUG] Attempting to delete file...`);
            }

            invoke('remove_task_record', {
                gid: id,
                deleteFile: deleteFile,
                filepath: filepath
            }).catch(e => console.error("remove failed", e));

            return tasks.filter(t => t.id !== id);
        }
        return tasks;
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
    // We should iterate active tasks and pause them
    // Or call a batch method. iterating is easiest for now.
    activeTasks.subscribe(tasks => {
        tasks.forEach(t => {
            if (t.state === 'downloading' || t.state === 'waiting') {
                pauseTask(t.id);
            }
        });
    })(); // Subscribe returns unsubscribe, but we just want the current value once? 
    // Actually using `get(activeTasks)` is better but we are inside the module.
    // simpler:
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
