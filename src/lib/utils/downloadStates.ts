/**
 * 下载状态相关的常量和工具函数
 */
import type { DownloadState } from '$lib/types/download';

/**
 * 下载状态分组
 */
export const DownloadStateGroups = {
    /** 活跃任务：进行中、等待中、已暂停 */
    ACTIVE: ['active', 'waiting', 'paused'] as const,
    /** 已完成任务 */
    COMPLETED: ['complete'] as const,
    /** 可恢复任务：已暂停、已移除 */
    RESUMABLE: ['paused', 'removed'] as const,
    /** 可删除任务：已完成、已移除、错误 */
    REMOVABLE: ['complete', 'removed', 'error'] as const,
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
    return state === 'complete';
}

/**
 * 判断是否可删除
 */
export function isRemovableTask(state: DownloadState): boolean {
    return (DownloadStateGroups.REMOVABLE as readonly string[]).includes(state);
}

/**
 * 判断是否为已暂停任务
 */
export function isPausedTask(state: DownloadState): boolean {
    return state === 'paused';
}

/**
 * 判断是否为下载中任务
 */
export function isDownloadingTask(state: DownloadState): boolean {
    return state === 'active';
}

/**
 * 判断是否为等待中任务
 */
export function isWaitingTask(state: DownloadState): boolean {
    return state === 'waiting';
}

/**
 * 判断是否为错误任务
 */
export function isErrorTask(state: DownloadState): boolean {
    return state === 'error';
}

/**
 * 判断是否为已移除任务
 */
export function isCancelledTask(state: DownloadState): boolean {
    return state === 'removed';
}

/**
 * 判断是否为文件缺失任务
 */
export function isMissingTask(state: DownloadState): boolean {
    return state === 'missing';
}

/**
 * 判断是否为可恢复任务（暂停、取消、错误）
 */
export function isResumableTask(state: DownloadState): boolean {
    return (DownloadStateGroups.RESUMABLE as readonly string[]).includes(state) || state === 'error';
}

/**
 * 判断是否为终态任务（已完成、已移除、错误、缺失）
 */
export function isTerminalTask(state: DownloadState): boolean {
    return ['complete', 'removed', 'error', 'missing'].includes(state);
}
