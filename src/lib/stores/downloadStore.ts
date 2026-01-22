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

// ============ 内部状态 ============

// 所有下载任务（主数据源）
const { subscribe: subscribeTasks, set: setTasks, update: updateTasks } = writable<DownloadTask[]>([
    // 初始 Mock 数据
    {
        id: '1',
        filename: 'macOS-Tahoe-26.0.dmg',
        progress: 75,
        speed: '12.5 MB/s',
        downloaded: '3.2 GB',
        total: '4.3 GB',
        remaining: '1:28',
        state: 'downloading',
        addedAt: '2024-05-20 14:30'
    },
    {
        id: '2',
        filename: 'Xcode_16.2.xip',
        progress: 45,
        speed: '8.3 MB/s',
        downloaded: '2.1 GB',
        total: '4.7 GB',
        remaining: '5:12',
        state: 'downloading',
        addedAt: '2024-05-20 15:10'
    },
    {
        id: '3',
        filename: 'SF-Pro-Fonts.pkg',
        progress: 100,
        downloaded: '156 MB',
        total: '156 MB',
        state: 'completed',
        addedAt: '2024-05-19 09:20'
    },
    {
        id: '4',
        filename: 'node-v22.0.0.pkg',
        progress: 30,
        downloaded: '24 MB',
        total: '80 MB',
        state: 'paused',
        addedAt: '2024-05-18 18:45'
    },
    {
        id: '5',
        filename: 'docker-desktop.dmg',
        progress: 0,
        state: 'waiting',
        addedAt: '2024-05-21 10:00'
    }
]);

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

/**
 * 添加下载任务
 */
export function addDownloadTask(config: DownloadConfig): void {
    updateTasks(tasks => {
        const newTasks: DownloadTask[] = config.urls.map(url => ({
            id: crypto.randomUUID(),
            filename: config.filename || extractFilenameFromUrl(url),
            progress: 0,
            state: 'waiting',
            addedAt: formatAddedAt()
        }));

        return [...tasks, ...newTasks];
    });

    // TODO: 将 config 传递给 aria2 API
    console.log('Download config:', config);
}

/**
 * 暂停任务
 */
export function pauseTask(id: string): void {
    updateTasks(tasks => {
        const task = tasks.find(t => t.id === id);
        if (task && task.state === 'downloading') {
            task.state = 'paused';
        }
        return tasks;
    });
}

/**
 * 恢复任务
 */
export function resumeTask(id: string): void {
    updateTasks(tasks => {
        const task = tasks.find(t => t.id === id);
        if (task && (task.state === 'paused' || task.state === 'cancelled' || task.state === 'waiting')) {
            task.state = 'downloading';
        }
        return tasks;
    });
}

/**
 * 取消任务（软删除）
 */
export function cancelTask(id: string): void {
    updateTasks(tasks => {
        const task = tasks.find(t => t.id === id);
        if (task && isActiveTask(task.state)) {
            task.state = 'cancelled';
        }
        return tasks;
    });
}

/**
 * 删除任务（硬删除）
 */
export function removeTask(id: string, deleteFile: boolean = false): void {
    updateTasks(tasks => {
        const index = tasks.findIndex(t => t.id === id);
        if (index === -1) return tasks;

        const task = tasks[index];
        if (isRemovableTask(task.state)) {
            tasks.splice(index, 1);

            if (deleteFile) {
                // TODO: 调用 Tauri API 删除文件
                console.log(`Delete file for task: ${id}`);
            }
        }

        return tasks;
    });
}

/**
 * 批量删除任务
 */
export function removeTasks(ids: Set<string>, deleteFile: boolean = false): void {
    updateTasks(tasks => {
        return tasks.filter(task => {
            if (ids.has(task.id) && isRemovableTask(task.state)) {
                if (deleteFile) {
                    // TODO: 调用 Tauri API 删除文件
                    console.log(`Delete file for task: ${task.id}`);
                }
                return false; // 过滤掉
            }
            return true; // 保留
        });
    });
}

/**
 * 批量取消任务
 */
export function cancelTasks(ids: Set<string>): void {
    updateTasks(tasks => {
        tasks.forEach(task => {
            if (ids.has(task.id) && isActiveTask(task.state)) {
                task.state = 'cancelled';
            }
        });
        return tasks;
    });
}

/**
 * 全局暂停所有下载中的任务
 */
export function pauseAll(): void {
    updateTasks(tasks => {
        tasks.forEach(task => {
            if (task.state === 'downloading') {
                task.state = 'paused';
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
        tasks.forEach(task => {
            if (task.state === 'paused') {
                task.state = 'downloading';
            }
        });
        return tasks;
    });
}

/**
 * 判断是否有正在下载的任务
 */
export function hasDownloadingTasks(tasks: DownloadTask[]): boolean {
    return tasks.some(t => t.state === 'downloading');
}

/**
 * 判断是否有暂停的任务
 */
export function hasPausedTasks(tasks: DownloadTask[]): boolean {
    return tasks.some(t => t.state === 'paused');
}
