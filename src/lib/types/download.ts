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
	url?: string;
	progress: number;
	speed?: string;
	downloaded?: string;
	total?: string;
	remaining?: string;
	state: DownloadState;
	addedAt?: string;
}

/**
 * 下载配置接口 - 用于创建新任务
 */
export interface DownloadConfig {
	urls: string[];
	savePath: string;
	filename: string;
	userAgent: string;
	referer: string;
	headers: string;
	proxy: string;
	maxDownloadLimit: string;
}

/**
 * 下载统计信息接口
 */
export interface DownloadStats {
	totalSpeed: string;
	totalSpeedBytes: number;
	activeCount: number;
	completedCount: number;
}

/**
 * 任务操作接口
 */
export interface TaskOperations {
	pause: (id: string) => void;
	resume: (id: string) => void;
	cancel: (id: string) => void;
	remove: (id: string) => void;
}
