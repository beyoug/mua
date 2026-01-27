/**
 * 下载状态相关的常量和工具函数
 */
import type { DownloadState } from '$lib/types/download';

/**
 * 下载状态分组
 */
export const DownloadStateGroups = {
    /** 活跃任务：进行中、等待中、已暂停 */
    ACTIVE: ['downloading', 'waiting', 'paused'] as const,
    /** 已完成任务 */
    COMPLETED: ['completed'] as const,
    /** 可恢复任务：已暂停、已取消 */
    RESUMABLE: ['paused', 'cancelled'] as const,
    /** 可删除任务：已完成、已取消、错误 */
    REMOVABLE: ['completed', 'cancelled', 'error'] as const,
} as const;

/**
 * 判断是否为活跃任务
 */
export function isActiveTask(state: DownloadState): boolean {
    return (DownloadStateGroups.ACTIVE as readonly string[]).includes(state);
}

/**
 * 判断是否为已完成任务
 */
export function isCompletedTask(state: DownloadState): boolean {
    return state === 'completed';
}

/**
 * 判断是否可删除
 */
export function isRemovableTask(state: DownloadState): boolean {
    return (DownloadStateGroups.REMOVABLE as readonly string[]).includes(state);
}

