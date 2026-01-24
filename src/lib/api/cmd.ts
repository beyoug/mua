/**
 * cmd.ts - Tauri Command API 统一入口
 * 集中管理所有与后端的通信，提供类型安全
 */

import { invoke } from '@tauri-apps/api/core';
import type { DownloadConfig } from '$lib/types/download';
import type { Aria2Task } from '$lib/types/aria2';

// === Command Wrappers ===

/**
 * 获取任务列表
 */
export async function getTasks(): Promise<Aria2Task[]> {
    return invoke<Aria2Task[]>('get_tasks');
}

/**
 * 添加下载任务
 */
export async function addDownloadTask(config: DownloadConfig): Promise<string> {
    return invoke<string>('add_download_task', {
        urls: config.urls,
        savePath: config.savePath,
        filename: config.filename,
        userAgent: config.userAgent,
        referer: config.referer,
        headers: config.headers,
        proxy: config.proxy,
        maxDownloadLimit: config.maxDownloadLimit
    });
}

/**
 * 暂停任务
 */
export async function pauseTask(gid: string): Promise<string> {
    return invoke<string>('pause_task', { gid });
}

/**
 * 恢复任务
 */
export async function resumeTask(gid: string): Promise<string> {
    return invoke<string>('resume_task', { gid });
}

/**
 * 取消任务
 */
export async function cancelTaskCmd(gid: string): Promise<string> {
    return invoke<string>('cancel_task', { gid });
}

/**
 * 移除任务记录（可选删除文件）
 */
export async function removeTaskRecord(gid: string, deleteFile: boolean, filepath: string | null): Promise<string> {
    return invoke<string>('remove_task_record', { gid, deleteFile, filepath });
}

/**
 * 获取 Aria2 配置文件路径
 */
export async function getAria2ConfigPath(): Promise<string> {
    return invoke<string>('get_aria2_config_path');
}

/**
 * 读取 Aria2 配置文件内容
 */
export async function readAria2Config(): Promise<string> {
    return invoke<string>('read_aria2_config');
}

/**
 * 导入 Aria2 配置文件
 */
export async function importAria2Config(path: string): Promise<string> {
    return invoke<string>('import_aria2_config', { path });
}

/**
 * 更新托盘图标速度
 */
export async function updateTrayIconSpeed(dlSpeed: number, ulSpeed: number): Promise<void> {
    // Backend expects u64, we pass number (JS uses double, tauri handles cast usually)
    // Safe to pass integer
    return invoke('update_tray_icon_with_speed', { dlSpeed, ulSpeed });
}
