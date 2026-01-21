/**
 * downloadSpeed.ts - 全局下载速度 Store
 * 用于在 Sidebar 统计和 ParticleBackground 之间共享下载速度
 */
import { writable } from 'svelte/store';

// 当前总下载速度 (bytes/s)
export const totalDownloadSpeed = writable(0);
