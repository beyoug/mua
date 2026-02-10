/**
 * download.ts - 下载任务相关类型定义
 */

/**
 * 下载任务状态
 */
export type DownloadState =
	| 'active'
	| 'paused'
	| 'complete'
	| 'error'
	| 'waiting'
	| 'removed'
	| 'missing';

/**
 * 速度信息 - 分离数值与单位，便于 UI 直接渲染
 */
export interface SpeedInfo {
	value: string;
	unit: string;
}

/**
 * 下载任务接口
 */
export interface DownloadTask {
	id: string;
	filename: string;
	url: string;
	progress: number;
	speed: SpeedInfo;
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
	completedAt?: string | null;
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
	totalSpeed: SpeedInfo;
	totalSpeedBytes: number;
	activeCount: number;
	completeCount: number;
}

/**
 * Aria2 版本信息
 */
export interface Aria2VersionInfo {
	version: string;
	is_custom: boolean;
	path: string;
	custom_binary_exists: boolean;
	custom_binary_version?: string;
}
