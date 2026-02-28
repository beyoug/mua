/**
 * download.ts - 下载任务相关类型定义
 */

/**
 * 下载任务状态
 */
export type DownloadState =
	| "active"
	| "paused"
	| "complete"
	| "error"
	| "waiting"
	| "removed"
	| "missing";

/**
 * 速度信息 - 分离数值与单位，便于 UI 直接渲染
 */
export interface SpeedInfo {
	value: string;
	unit: string;
}

/**
 * 下载任务接口 - 只包含原始数值，格式化由前端负责
 */
export interface DownloadTask {
	id: string;
	filename: string;
	url: string;
	progress: number;
	speed: number; // 原始 bytes/s
	completed: number; // 原始 bytes
	total: number; // 原始 bytes
	remainingSecs: number; // 原始秒数
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
	torrentConfig?: {
		path: string;
		selectFile?: string;
		trackers?: string;
	};
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
	custom_binary_trusted: boolean;
	custom_binary_hash_match: boolean;
	custom_binary_security_status:
		| "missing"
		| "untrusted"
		| "hash_mismatch"
		| "trusted";
}
