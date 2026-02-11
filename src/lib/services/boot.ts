import { listen } from '@tauri-apps/api/event';
import { message } from '@tauri-apps/plugin-dialog';
import { loadAppSettings } from '$lib/stores/settings';
import { initNotifications, cleanupNotifications } from '$lib/services/notifications';
import { createLogger } from '$lib/utils/logger';

const logger = createLogger('Boot');

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

    try {
        // 1. 环境治理：禁用系统级右键菜单（原子化控制）
        document.addEventListener('contextmenu', (e) => e.preventDefault());

        // 2. 状态映射：加载关键应用配置
        await loadAppSettings();

        // 3. 服务激活：初始化通知系统
        await initNotifications();

        // 4. 事件订阅：链路监控
        const unlistenSidecar = await listen<Aria2SidecarErrorPayload>('aria2-sidecar-error', async (event) => {
            const payload = event.payload;
            logger.error('Received sidecar error event', { payload });
            await message(
                `Aria2 Service Error: ${payload.message ?? 'Unknown error'}\n\nCode: ${payload.code ?? '-'}\nSignal: ${payload.signal ?? '-'}\n\nLog:\n${payload.stderr ?? ''}`,
                {
                    title: 'Aria2 Sidecar Error',
                    kind: 'error'
                }
            );
        });

        // 返回销毁函数块
        return () => {
            unlistenSidecar();
            cleanupNotifications();
        };
    } catch (e) {
        logger.error('Critical failure during frontend initialization', { error: e });
        throw e;
    }
}
