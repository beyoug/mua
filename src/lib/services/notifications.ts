/**
 * notifications.ts - 系统通知服务
 *
 * 职责：监听任务状态变化并发送系统通知
 * 通过订阅 store 检测任务完成，不依赖后端事件
 */

import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';
import type { DownloadTask } from '$lib/types/download';
import { allTasks } from '$lib/services/download';
import { createLogger } from '$lib/utils/logger';

const logger = createLogger('Notifications');

let unsubscribeFn: (() => void) | null = null;
let previousStates = new Map<string, string>();
let initialized = false;
let permissionStatus: 'unknown' | 'granted' | 'denied' = 'unknown';
let permissionPromptAttempted = false;
let permissionCheckInFlight: Promise<boolean> | null = null;

async function resolveNotificationPermission(): Promise<boolean> {
    if (permissionStatus === 'granted') {
        return true;
    }

    const alreadyGranted = await isPermissionGranted();
    if (alreadyGranted) {
        permissionStatus = 'granted';
        return true;
    }

    if (permissionPromptAttempted) {
        permissionStatus = 'denied';
        return false;
    }

    permissionPromptAttempted = true;
    const permission = await requestPermission();
    const granted = permission === 'granted';
    permissionStatus = granted ? 'granted' : 'denied';
    return granted;
}

async function ensureNotificationPermission(): Promise<boolean> {
    if (permissionStatus === 'granted') {
        return true;
    }

    if (permissionStatus === 'denied') {
        return false;
    }

    if (permissionCheckInFlight) {
        return permissionCheckInFlight;
    }

    permissionCheckInFlight = resolveNotificationPermission().finally(() => {
        permissionCheckInFlight = null;
    });
    return permissionCheckInFlight;
}

/**
 * 初始化通知监听器
 * 通过 store 订阅检测任务完成状态变化
 */
export async function initNotifications(): Promise<void> {
    if (unsubscribeFn) return;

    unsubscribeFn = allTasks.subscribe((tasks) => {
        if (!initialized) {
            // 首次订阅时，初始化状态快照（不触发通知）
            for (const task of tasks) {
                previousStates.set(task.id, task.state);
            }
            initialized = true;
            return;
        }

        // 检测状态变化：非完成 → 完成
        for (const task of tasks) {
            const prev = previousStates.get(task.id);
            if (task.state === 'complete' && prev && prev !== 'complete') {
                showCompletionNotification(task);
            }
            previousStates.set(task.id, task.state);
        }

        // 清理已移除的任务
        const currentIds = new Set(tasks.map(t => t.id));
        for (const id of previousStates.keys()) {
            if (!currentIds.has(id)) {
                previousStates.delete(id);
            }
        }
    });
}

/**
 * 清理通知监听器
 */
export function cleanupNotifications(): void {
    if (unsubscribeFn) {
        unsubscribeFn();
        unsubscribeFn = null;
    }
    previousStates.clear();
    initialized = false;
    permissionStatus = 'unknown';
    permissionPromptAttempted = false;
    permissionCheckInFlight = null;
}

/**
 * 显示下载完成通知
 */
async function showCompletionNotification(task: DownloadTask): Promise<void> {
    try {
        const permissionGranted = await ensureNotificationPermission();

        if (permissionGranted) {
            sendNotification({
                title: '下载完成',
                body: `${task.filename} 已下载完成`,
            });
        }
    } catch (e) {
        logger.error('Failed to send completion notification', { taskId: task.id, error: e });
    }
}
