/**
 * cmd.ts - Tauri Command API 统一入口
 * 集中管理所有与后端的通信，提供类型安全
 */

import { invoke } from '@tauri-apps/api/core';
import type { DownloadConfig } from '$lib/types/download';
import type { DownloadTask } from '$lib/types/download';

// === Command Wrappers ===

/**
 * 获取任务列表
 */
export async function getTasks(): Promise<DownloadTask[]> {
    return invoke<DownloadTask[]>('get_tasks');
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
export async function removeTaskRecord(gid: string, deleteFile: boolean): Promise<string> {
    return invoke<string>('remove_task_record', { gid, deleteFile });
}

/**
 * 暂停所有任务
 */
export async function pauseAllTasks(): Promise<void> {
    await invoke('pause_all_tasks');
}

/**
 * 恢复所有任务
 */
export async function resumeAllTasks(): Promise<void> {
    await invoke('resume_all_tasks');
}

/**
 * 批量移除任务
 */
export async function removeTasksCmd(gids: string[], deleteFile: boolean): Promise<void> {
    await invoke('remove_tasks', { gids, deleteFile });
}

/**
 * 批量取消任务
 */
export async function cancelTasksCmd(gids: string[]): Promise<void> {
    await invoke('cancel_tasks', { gids });
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
 * 在文件夹中显示任务文件
 */
export async function showTaskInFolder(gid: string): Promise<void> {
    return invoke('show_task_in_folder', { gid });
}
