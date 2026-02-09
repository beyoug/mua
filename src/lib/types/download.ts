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
	| 'cancelled'
	| 'missing';

/**
 * 下载任务接口
 */
export interface DownloadTask {
	id: string;
	filename: string;
	url: string;
	progress: number;
	speed: string;
	speed_u64: number;
	downloaded: string;
	downloaded_u64: number;
	total: string;
	total_u64: number;
	remaining: string;
	state: DownloadState;
	addedAt: string;
	savePath: string;
	errorMessage?: string;
	userAgent?: string;
	referer?: string;
	proxy?: string;
	headers?: string[];
	maxDownloadLimit?: string;
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


