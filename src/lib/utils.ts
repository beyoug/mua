/**
 * utils.ts - 通用工具函数
 * 注：格式化函数已迁移至 utils/formatters.ts
 */

import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

/**
 * 合并 CSS 类名（Tailwind 类名智能合并）
 */
export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}
