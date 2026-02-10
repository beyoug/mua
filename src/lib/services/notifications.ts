/**
 * notifications.ts - 系统通知服务
 * 
 * 职责：监听后端事件并发送系统通知
 * 独立于 Store，可在 App 根组件初始化
 */

import { listen } from '@tauri-apps/api/event';
import type { UnlistenFn } from '@tauri-apps/api/event';
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';
import type { DownloadTask } from '$lib/types/download';

let unlistenFn: UnlistenFn | null = null;

/**
 * 初始化通知监听器
 * 应在 App 根组件调用（如 +layout.svelte）
 */
export async function initNotifications(): Promise<void> {
    if (unlistenFn) return; // 已初始化

    try {
        unlistenFn = await listen<DownloadTask>('task-complete', async (event) => {
            const task = event.payload;
            await showCompletionNotification(task);
        });
    } catch (e) {
        console.error('Failed to setup notification listener:', e);
    }
}

/**
 * 清理通知监听器
 */
export function cleanupNotifications(): void {
    if (unlistenFn) {
        unlistenFn();
        unlistenFn = null;
    }
}

/**
 * 显示下载完成通知
 */
async function showCompletionNotification(task: DownloadTask): Promise<void> {
    try {
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
    } catch (e) {
        console.error('Failed to send notification:', e);
    }
}
