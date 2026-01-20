/**
 * aria2 RPC 方法封装
 * 提供类型安全的 aria2 API 调用
 */

import type { DownloadStatus, GlobalStat, VersionInfo } from './types';
import { getAria2Client } from './client';

/**
 * 获取 aria2 版本信息
 */
export async function getVersion(): Promise<VersionInfo> {
    const client = getAria2Client();
    return client.call<VersionInfo>('aria2.getVersion');
}

/**
 * 获取全局统计信息
 */
export async function getGlobalStat(): Promise<GlobalStat> {
    const client = getAria2Client();
    return client.call<GlobalStat>('aria2.getGlobalStat');
}

/**
 * 获取活动下载列表
 */
export async function tellActive(): Promise<DownloadStatus[]> {
    const client = getAria2Client();
    return client.call<DownloadStatus[]>('aria2.tellActive');
}

/**
 * 获取等待中的下载列表
 */
export async function tellWaiting(offset: number, num: number): Promise<DownloadStatus[]> {
    const client = getAria2Client();
    return client.call<DownloadStatus[]>('aria2.tellWaiting', [offset, num]);
}

/**
 * 获取已停止的下载列表
 */
export async function tellStopped(offset: number, num: number): Promise<DownloadStatus[]> {
    const client = getAria2Client();
    return client.call<DownloadStatus[]>('aria2.tellStopped', [offset, num]);
}

/**
 * 获取指定下载状态
 */
export async function tellStatus(gid: string): Promise<DownloadStatus> {
    const client = getAria2Client();
    return client.call<DownloadStatus>('aria2.tellStatus', [gid]);
}

/**
 * 添加 URI 下载
 */
export async function addUri(
    uris: string[],
    options?: Record<string, string>
): Promise<string> {
    const client = getAria2Client();
    return client.call<string>('aria2.addUri', [uris, options || {}]);
}

/**
 * 暂停下载
 */
export async function pause(gid: string): Promise<string> {
    const client = getAria2Client();
    return client.call<string>('aria2.pause', [gid]);
}

/**
 * 暂停所有下载
 */
export async function pauseAll(): Promise<'OK'> {
    const client = getAria2Client();
    return client.call<'OK'>('aria2.pauseAll');
}

/**
 * 恢复下载
 */
export async function unpause(gid: string): Promise<string> {
    const client = getAria2Client();
    return client.call<string>('aria2.unpause', [gid]);
}

/**
 * 恢复所有下载
 */
export async function unpauseAll(): Promise<'OK'> {
    const client = getAria2Client();
    return client.call<'OK'>('aria2.unpauseAll');
}

/**
 * 移除下载
 */
export async function remove(gid: string): Promise<string> {
    const client = getAria2Client();
    return client.call<string>('aria2.remove', [gid]);
}

/**
 * 强制移除下载
 */
export async function forceRemove(gid: string): Promise<string> {
    const client = getAria2Client();
    return client.call<string>('aria2.forceRemove', [gid]);
}
