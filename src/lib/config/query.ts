import { QueryClient } from '@tanstack/svelte-query';

/**
 * TanStack Query 全局配置
 * 针对 aria2 实时状态轮询优化
 */
export const queryClient = new QueryClient({
    defaultOptions: {
        queries: {
            // 窗口获得焦点时自动 refetch
            refetchOnWindowFocus: true,
            // 数据过期时间 (ms)
            staleTime: 1000,
            // 失败重试次数
            retry: 2,
            // 重试延迟
            retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000)
        },
        mutations: {
            retry: 1
        }
    }
});
