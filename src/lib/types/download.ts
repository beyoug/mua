/**
 * download.ts - 下载任务相关类型定义
 */

/**
 * 下载任务状态
 */
export type DownloadState = 
	| 'downloading' 
	| 'paused' 
	| 'completed' 
	| 'error' 
	| 'waiting' 
	| 'cancelled';

/**
 * 下载任务接口
 */
export interface DownloadTask {
	id: string;
	filename: string;
	progress: number;
	speed?: string;
	downloaded?: string;
	total?: string;
	remaining?: string;
	state: DownloadState;
	addedAt?: string;
}
