/**
 * notifications.ts - 系统通知服务
 *
 * 职责：监听任务状态变化并发送系统通知
 * 通过订阅 store 检测任务完成，不依赖后端事件
 */

import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';
import { listen } from '@tauri-apps/api/event';
import type { UnlistenFn } from '@tauri-apps/api/event';
import { createLogger } from '$lib/utils/logger';
import { EVENT_TASK_COMPLETED } from '$lib/api/events';

const logger = createLogger('Notifications');

let unsubscribeFn: UnlistenFn | null = null;
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

    unsubscribeFn = await listen<{ id: string; filename: string }>(EVENT_TASK_COMPLETED, (event) => {
        const payload = event.payload;
        showCompletionNotification(payload.id, payload.filename);
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
    permissionStatus = 'unknown';
    permissionPromptAttempted = false;
    permissionCheckInFlight = null;
}

/**
 * 显示下载完成通知
 */
async function showCompletionNotification(taskId: string, filename: string): Promise<void> {
    try {
        const permissionGranted = await ensureNotificationPermission();

        if (permissionGranted) {
            sendNotification({
                title: '下载完成',
                body: `${filename} 已下载完成`,
            });
        }
    } catch (e) {
        logger.error('Failed to send completion notification', { taskId, error: e });
    }
}
