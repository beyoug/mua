import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";
import { SIZE_UNITS } from "./config/constants";

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

/**
 * 格式化文件大小
 * @param bytes 字节数
 * @param decimals 小数位数
 */
export function formatFileSize(bytes: number | string, decimals = 2): string {
	const size = typeof bytes === 'string' ? parseInt(bytes, 10) : bytes;
	if (size === 0) return '0 B';

	const k = 1024;
	const dm = decimals < 0 ? 0 : decimals;
	const i = Math.floor(Math.log(size) / Math.log(k));

	return `${parseFloat((size / Math.pow(k, i)).toFixed(dm))} ${SIZE_UNITS[i]}`;
}

/**
 * 格式化下载速度
 * @param bytesPerSecond 每秒字节数
 */
export function formatSpeed(bytesPerSecond: number | string): string {
	return `${formatFileSize(bytesPerSecond)}/s`;
}

/**
 * 格式化进度百分比
 */
export function formatProgress(completed: string, total: string): number {
	const c = parseInt(completed, 10);
	const t = parseInt(total, 10);
	if (t === 0) return 0;
	return Math.round((c / t) * 100);
}
