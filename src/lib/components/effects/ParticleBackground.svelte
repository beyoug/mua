<!--
  ParticleBackground.svelte
  背景粒子效果 - 响应下载速度动态调整
  优化：更分散的发射区域、有机曲线运动、主题感知颜色
-->
<script lang="ts">
import { onMount, onDestroy } from 'svelte';
import { downloadStats } from '$lib/stores/downloadStore';
import { getEmitRate, getSpeedMultiplier } from '$lib/utils/particles';

interface Particle {
  x: number;
  y: number;
  size: number;
  vx: number;
  vy: number;
  opacity: number;
  life: number;
  maxLife: number;
  // 新增：用于曲线运动的参数
  wavePhase: number;
  waveAmplitude: number;
}

let canvas: HTMLCanvasElement;
let ctx: CanvasRenderingContext2D;
let particles: Particle[] = [];
let animationId: number;
let lastTime = 0;
let emitAccumulator = 0;
let currentSpeedMbps = 0;

// 缓存主题颜色，避免每帧读取 DOM
let cachedColors = { primary: '#ffffff', glow: 'rgba(255,255,255,0.2)' };
let colorCacheTime = 0;
const COLOR_CACHE_DURATION = 500; // 每 500ms 更新一次颜色缓存

// 粒子池 - 避免频繁创建对象
const POOL_SIZE = 800;
const particlePool: Particle[] = [];
let poolIndex = 0;

// 初始化粒子池
function initPool() {
  for (let i = 0; i < POOL_SIZE; i++) {
    particlePool.push({
      x: 0, y: 0, size: 0, vx: 0, vy: 0,
      opacity: 0, life: 0, maxLife: 0,
      wavePhase: 0, waveAmplitude: 0
    });
  }
}

// 订阅下载速度（直接从 downloadStats 获取，无需中间 Store）
const unsubscribeSpeed = downloadStats.subscribe((stats) => {
  currentSpeedMbps = stats.totalSpeedBytes / (1024 * 1024);
});

// 获取粒子颜色（主题感知）
function updateColorCache() {
  const now = performance.now();
  if (now - colorCacheTime < COLOR_CACHE_DURATION) return;
  colorCacheTime = now;
  
  const style = getComputedStyle(document.documentElement);
  const isMinimalTheme = document.documentElement.classList.contains('theme-minimal');
  const isLightMode = document.documentElement.classList.contains('light');
  
  if (isMinimalTheme) {
    // 极简模式：使用柔和的中性灰色
    cachedColors = isLightMode 
      ? { primary: 'rgba(120, 120, 130, 1)', glow: 'rgba(120, 120, 130, 0.25)' }
      : { primary: 'rgba(160, 160, 170, 1)', glow: 'rgba(160, 160, 170, 0.2)' };
  } else {
    cachedColors = {
      primary: style.getPropertyValue('--accent-primary').trim() || '#3B82F6',
      glow: style.getPropertyValue('--accent-glow').trim() || 'rgba(59, 130, 246, 0.4)'
    };
  }
}

// 从池中获取粒子
function getParticleFromPool(): Particle | null {
  if (particles.length >= POOL_SIZE) return null;
  
  const particle = particlePool[poolIndex];
  poolIndex = (poolIndex + 1) % POOL_SIZE;
  
  // 更分散的发射角度
  const baseAngle = Math.PI / 4 + (Math.random() - 0.5) * 0.4;
  const angleVariation = (Math.random() - 0.5) * 0.7;
  const angle = baseAngle + angleVariation;
  const speedMultiplier = getSpeedMultiplier(currentSpeedMbps);
  const speed = (25 + Math.random() * 35) * speedMultiplier;
  
  // 更宽的发射区域 - 从左下角扩展
  particle.x = Math.random() * (canvas.width * 0.3);
  particle.y = (canvas.height * 0.7) + Math.random() * (canvas.height * 0.3);
  
  // 粒子大小：使用幂函数让小粒子更多
  particle.size = 1.2 + Math.pow(Math.random(), 1.5) * 4.5;
  
  particle.vx = Math.cos(angle) * speed;
  particle.vy = -Math.sin(angle) * speed;
  particle.opacity = 0;
  particle.life = 0;
  particle.maxLife = 10 + Math.random() * 8;
  
  // 曲线运动参数
  particle.wavePhase = Math.random() * Math.PI * 2;
  particle.waveAmplitude = 0.2 + Math.random() * 0.4;
  
  return particle;
}

// 更新粒子状态
function updateParticles(deltaTime: number) {
  const dt = deltaTime / 1000;
  
  // 释放新粒子
  const emitRate = getEmitRate(currentSpeedMbps);
  if (emitRate > 0) {
    emitAccumulator += emitRate * dt;
    while (emitAccumulator >= 1) {
      const particle = getParticleFromPool();
      if (particle) particles.push(particle);
      emitAccumulator -= 1;
    }
  }
  
  // 更新每个粒子
  for (let i = particles.length - 1; i >= 0; i--) {
    const p = particles[i];
    p.life += dt;
    const lifeRatio = p.life / p.maxLife;
    
    // 更平滑的渐入渐出曲线
    if (lifeRatio < 0.15) {
      p.opacity = (lifeRatio / 0.15) * 0.3;
    } else if (lifeRatio > 0.75) {
      p.opacity = ((1 - lifeRatio) / 0.25) * 0.3;
    } else {
      p.opacity = 0.3;
    }
    
    // 更新位置 - 添加柔和的曲线运动
    p.x += p.vx * dt;
    p.y += p.vy * dt;
    
    // 微弱的横向震荡，让轨迹更有机
    p.x += Math.sin(p.life * 0.6 + p.wavePhase) * p.waveAmplitude;
    
    // 移除死亡或越界的粒子
    if (p.life >= p.maxLife || p.x > canvas.width * 1.1 || p.y < -20) {
      particles.splice(i, 1);
    }
  }
}

// 渲染粒子
function renderParticles() {
  if (!ctx || !canvas) return;
  
  // 更新颜色缓存
  updateColorCache();
  
  // 清空画布
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  
  // 绘制粒子
  for (const p of particles) {
    ctx.save();
    ctx.globalAlpha = p.opacity;
    
    // 绘制光晕
    const gradient = ctx.createRadialGradient(p.x, p.y, 0, p.x, p.y, p.size * 2.5);
    gradient.addColorStop(0, cachedColors.primary);
    gradient.addColorStop(0.4, cachedColors.glow);
    gradient.addColorStop(1, 'transparent');
    
    ctx.fillStyle = gradient;
    ctx.beginPath();
    ctx.arc(p.x, p.y, p.size * 2.5, 0, Math.PI * 2);
    ctx.fill();
    
    // 绘制核心 - 更小更亮
    ctx.globalAlpha = p.opacity * 1.5;
    ctx.fillStyle = cachedColors.primary;
    ctx.beginPath();
    ctx.arc(p.x, p.y, p.size * 0.4, 0, Math.PI * 2);
    ctx.fill();
    
    ctx.restore();
  }
}

// 动画循环
function animate(currentTime: number) {
  if (lastTime === 0) lastTime = currentTime;
  const deltaTime = Math.min(currentTime - lastTime, 100);
  lastTime = currentTime;
  
  updateParticles(deltaTime);
  renderParticles();
  
  animationId = requestAnimationFrame(animate);
}

// 处理窗口缩放
function handleResize() {
  if (!canvas) return;
  canvas.width = window.innerWidth;
  canvas.height = window.innerHeight;
}

onMount(() => {
  ctx = canvas.getContext('2d', { 
    alpha: true,
    desynchronized: true
  })!;
  

  handleResize();
  window.addEventListener('resize', handleResize);
  document.addEventListener('visibilitychange', handleVisibilityChange);
  
  initPool();
  updateColorCache();
  startAnimation();
});

function startAnimation() {
    if (!animationId) {
        lastTime = performance.now();
        animate(lastTime);
    }
}

function stopAnimation() {
    if (animationId) {
        cancelAnimationFrame(animationId);
        animationId = 0;
    }
}

function handleVisibilityChange() {
    if (document.hidden) {
        stopAnimation();
    } else {
        startAnimation();
    }
}

onDestroy(() => {
  stopAnimation();
  window.removeEventListener('resize', handleResize);
  document.removeEventListener('visibilitychange', handleVisibilityChange);
  unsubscribeSpeed();
});
</script>

<div class="particle-container">
  <!-- 背景光晕 - 仅彩色主题显示 -->
  <div class="glow glow-1"></div>
  <div class="glow glow-2"></div>
  
  <!-- Canvas 粒子层 -->
  <canvas bind:this={canvas} class="particle-canvas"></canvas>
</div>

<style>
.particle-container {
  position: fixed;
  inset: 0;
  overflow: hidden;
  pointer-events: none;
  z-index: 0;
}

.particle-canvas {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
}

/* 背景光晕 - 更柔和 */
.glow {
  position: absolute;
  border-radius: 50%;
  filter: blur(120px);
  opacity: 0.1;
  transition: opacity 0.8s ease;
}

.glow-1 {
  width: 400px;
  height: 400px;
  background: var(--accent-primary);
  bottom: -120px;
  left: -80px;
  animation: pulse 10s ease-in-out infinite;
}

.glow-2 {
  width: 350px;
  height: 350px;
  background: var(--accent-secondary);
  top: -80px;
  right: 80px;
  opacity: 0.06;
  animation: pulse 12s ease-in-out infinite reverse;
}

@keyframes pulse {
  0%, 100% {
    transform: scale(1);
    opacity: 0.1;
  }
  50% {
    transform: scale(1.08);
    opacity: 0.12;
  }
}

/* 极简主题：隐藏背景光晕 */
:global(html.theme-minimal) .glow {
  display: none;
}
</style>
