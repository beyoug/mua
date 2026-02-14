import { invoke } from '@tauri-apps/api/core';
import type { DownloadConfig, DownloadTask } from '$lib/types/download';

export async function getTasks(): Promise<DownloadTask[]> {
    return invoke<DownloadTask[]>('get_tasks');
}

export async function addDownloadTasks(configs: DownloadConfig[]): Promise<(string | null)[]> {
    return invoke<(string | null)[]>('add_download_tasks', { configs });
}

export async function pauseTask(gid: string): Promise<void> {
    await invoke<void>('pause_task', { gid });
}

export async function resumeTask(gid: string): Promise<string> {
    return invoke<string>('resume_task', { gid });
}

export async function cancelTask(gid: string): Promise<void> {
    await invoke<void>('cancel_task', { gid });
}

export async function removeTaskRecord(gid: string, deleteFile: boolean): Promise<void> {
    await invoke<void>('remove_task_record', { gid, deleteFile });
}

export async function pauseAllTasks(): Promise<void> {
    await invoke('pause_all_tasks');
}

export async function resumeAllTasks(): Promise<void> {
    await invoke('resume_all_tasks');
}

export async function removeTasks(gids: string[], deleteFile: boolean): Promise<void> {
    await invoke('remove_tasks', { gids, deleteFile });
}

export async function cancelTasks(gids: string[]): Promise<void> {
    await invoke('cancel_tasks', { gids });
}

export async function showTaskInFolder(gid: string): Promise<void> {
    return invoke('show_task_in_folder', { gid });
}
