/**
 * particles.ts - 粒子效果计算工具
 * 用于统一 SettingsPanel 和 ParticleBackground 的粒子计算逻辑
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
 * 估算当前粒子数量
 * @param speedMbps 下载速度 (MB/s)
 * @returns 预估的粒子数量（基于平均生命周期约14秒）
 */
export function getEstimatedParticles(speedMbps: number): number {
    return Math.round(getEmitRate(speedMbps) * 14);
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
