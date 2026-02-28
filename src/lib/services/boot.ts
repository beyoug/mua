import { listen } from '@tauri-apps/api/event';
import { loadAppSettings } from '$lib/services/settings';
import { downloadService } from '$lib/services/download';
import { initNotifications, cleanupNotifications } from '$lib/services/notifications';
import { createLogger } from '$lib/utils/logger';
import { EVENT_ARIA2_SIDECAR_ERROR } from '$lib/api/events';
import { showFeedback } from '$lib/services/feedback';

const logger = createLogger('Boot');
let bootCleanup: (() => void) | null = null;
let bootInFlight: Promise<() => void> | null = null;

interface Aria2SidecarErrorPayload {
    message?: string;
    code?: number | string;
    signal?: number | string;
    stderr?: string;
}

/**
 * 应用前端启动编排器
 * 官方术语：前端生命周期治理服务
 */
export async function bootApp() {
    if (bootCleanup) {
        return bootCleanup;
    }

    if (bootInFlight) {
        return bootInFlight;
    }

    bootInFlight = (async () => {

    try {
        const preventContextMenu = (e: MouseEvent) => {
            // 允许输入框/文本域使用系统右键菜单（复制/粘贴）
            const target = e.target as HTMLElement;
            if (
                target instanceof HTMLInputElement ||
                target instanceof HTMLTextAreaElement ||
                target.isContentEditable
            ) {
                return;
            }
            e.preventDefault();
        };
        document.addEventListener('contextmenu', preventContextMenu);

        // 2. 状态映射：加载关键应用配置
        await loadAppSettings();

        // 3. 启动下载同步服务
        await downloadService.initializeSync();

        // 4. 服务激活：初始化通知系统
        await initNotifications();

        // 5. 事件订阅：链路监控
        const unlistenSidecar = await listen<Aria2SidecarErrorPayload>(EVENT_ARIA2_SIDECAR_ERROR, async (event) => {
            const payload = event.payload;
            logger.error('Received sidecar error event', { payload });
            await showFeedback(
                `Aria2 Service Error: ${payload.message ?? 'Unknown error'}\n\nCode: ${payload.code ?? '-'}\nSignal: ${payload.signal ?? '-'}\n\nLog:\n${payload.stderr ?? ''}`,
                {
                    title: 'Aria2 Sidecar Error',
                    kind: 'error'
                }
            );
        });

        // 返回销毁函数块
        const cleanup = () => {
            unlistenSidecar();
            cleanupNotifications();
            document.removeEventListener('contextmenu', preventContextMenu);
        };
        bootCleanup = cleanup;
        return cleanup;
    } catch (e) {
        logger.error('Critical failure during frontend initialization', { error: e });
        throw e;
    } finally {
        bootInFlight = null;
    }
    })();

    return bootInFlight;
}

export async function recoverAppFromRuntimeFailure(): Promise<void> {
    try {
        await loadAppSettings();
        await downloadService.initializeSync();
    } catch (e) {
        logger.error('Recover app failed', { error: e });
        throw e;
    }
}

export function shutdownBootServices(): void {
    if (bootCleanup) {
        bootCleanup();
        bootCleanup = null;
    }
}
