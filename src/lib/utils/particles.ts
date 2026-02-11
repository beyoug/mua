/**
 * particles.ts - 粒子效果计算工具
 */

/**
 * 根据下载速度计算粒子发射速率
 * @param speedMbps 下载速度 (MB/s)
 * @returns 每秒发射的粒子数
 */
export function getEmitRate(speedMbps: number): number {
    if (speedMbps <= 0) return 0;
    return Math.min(4 + speedMbps * 0.4, 40);
}

/**
 * 根据下载速度计算粒子速度倍数
 * @param speedMbps 下载速度 (MB/s)
 * @returns 速度倍数 (1.0 - 1.8)
 */
export function getSpeedMultiplier(speedMbps: number): number {
    if (speedMbps <= 0) return 1;
    return 1 + Math.min(speedMbps / 150, 0.8);
}
