<script lang="ts">
import { onMount, onDestroy } from 'svelte';
import { totalDownloadSpeed } from '$lib/stores/downloadSpeed';

interface Particle {
  x: number;
  y: number;
  size: number;
  vx: number;
  vy: number;
  opacity: number;
  life: number;
  maxLife: number;
}

let canvas: HTMLCanvasElement;
let ctx: CanvasRenderingContext2D;
let particles: Particle[] = [];
let animationId: number;
let lastTime = 0;
let emitAccumulator = 0;
let currentSpeedMbps = 0;

// 粒子池 - 避免频繁创建对象
const POOL_SIZE = 1000;
const particlePool: Particle[] = [];
let poolIndex = 0;

// 初始化粒子池
function initPool() {
  for (let i = 0; i < POOL_SIZE; i++) {
    particlePool.push({
      x: 0, y: 0, size: 0, vx: 0, vy: 0,
      opacity: 0, life: 0, maxLife: 0
    });
  }
}

// 订阅下载速度
const unsubscribeSpeed = totalDownloadSpeed.subscribe((speed) => {
  currentSpeedMbps = speed / (1024 * 1024);
});

function getEmitRate(): number {
  if (currentSpeedMbps <= 0) return 0;
  return Math.min(6 + currentSpeedMbps * 0.5, 56);
}

function getSpeedMultiplier(): number {
  if (currentSpeedMbps <= 0) return 1;
  return 1 + Math.min(currentSpeedMbps / 100, 1);
}

function getGlowOpacity(base: number): number {
  return base + Math.min(currentSpeedMbps / 100, 1) * 0.25;
}

// 从池中获取粒子
function getParticleFromPool(): Particle | null {
  if (particles.length >= POOL_SIZE) return null;
  
  const particle = particlePool[poolIndex];
  poolIndex = (poolIndex + 1) % POOL_SIZE;
  
  const isScattered = Math.random() < 0.15;
  const baseAngle = Math.PI / 4;
  const angleVariation = (Math.random() - 0.5) * 0.6;
  const angle = baseAngle + angleVariation;
  const speedMultiplier = getSpeedMultiplier();
  const speed = (30 + Math.random() * 40) * speedMultiplier;
  
  // 从左下角发射粒子，对应侧边栏底部的网速统计位置
  particle.x = (canvas.width * 0.05) + Math.random() * (canvas.width * 0.1);
  particle.y = (canvas.height * 0.85) + Math.random() * (canvas.height * 0.1);
  particle.size = 2 + Math.random() * 4;
  particle.vx = Math.cos(angle) * speed * (isScattered ? 0.6 : 1);
  particle.vy = -Math.sin(angle) * speed * (isScattered ? 0.6 : 1);
  particle.opacity = 0;
  particle.life = 0;
  particle.maxLife = 8 + Math.random() * 6;
  
  return particle;
}

// 更新粒子状态
function updateParticles(deltaTime: number) {
  const dt = deltaTime / 1000;
  
  // 释放新粒子
  const emitRate = getEmitRate();
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
    
    // 渐入渐出
    if (lifeRatio < 0.1) {
      p.opacity = lifeRatio / 0.1 * 0.5;
    } else if (lifeRatio > 0.8) {
      p.opacity = (1 - lifeRatio) / 0.2 * 0.5;
    } else {
      p.opacity = 0.5;
    }
    
    // 更新位置
    p.x += p.vx * dt;
    p.y += p.vy * dt;
    
    // 移除死亡或越界的粒子
    if (p.life >= p.maxLife || p.x > canvas.width * 1.1 || p.y < -10) {
      particles.splice(i, 1);
    }
  }
}

// 渲染粒子
function renderParticles() {
  if (!ctx || !canvas) return;
  
  // 清空画布
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  
  // 获取当前主题颜色
  const accentPrimary = getComputedStyle(document.documentElement)
    .getPropertyValue('--accent-primary').trim();
  const accentGlow = getComputedStyle(document.documentElement)
    .getPropertyValue('--accent-glow').trim();
  
  // 绘制粒子
  for (const p of particles) {
    ctx.save();
    
    ctx.globalAlpha = p.opacity;
    
    // 绘制光晕
    const gradient = ctx.createRadialGradient(p.x, p.y, 0, p.x, p.y, p.size * 2);
    gradient.addColorStop(0, accentPrimary);
    gradient.addColorStop(0.5, accentGlow);
    gradient.addColorStop(1, 'transparent');
    
    ctx.fillStyle = gradient;
    ctx.beginPath();
    ctx.arc(p.x, p.y, p.size * 2, 0, Math.PI * 2);
    ctx.fill();
    
    // 绘制核心
    ctx.fillStyle = accentPrimary;
    ctx.beginPath();
    ctx.arc(p.x, p.y, p.size / 2, 0, Math.PI * 2);
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
  
  initPool();
  animationId = requestAnimationFrame(animate);
});

onDestroy(() => {
  cancelAnimationFrame(animationId);
  window.removeEventListener('resize', handleResize);
  unsubscribeSpeed();
});
</script>

<div class="particle-container">
  <!-- 背景光晕 -->
  <div class="glow glow-1" style="opacity: {getGlowOpacity(0.15)}"></div>
  <div class="glow glow-2" style="opacity: {getGlowOpacity(0.1)}"></div>
  
  <!-- Canvas 粒子层 -->
  <canvas bind:this={canvas} class="particle-canvas"></canvas>
</div>

<style>
.particle-container {
  position: fixed;
  inset: 0;
  overflow: hidden;
  pointer-events: none;
  z-index: 0; /* 作为背景层，在所有内容下方 */
}

.particle-canvas {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
}

/* 背景光晕 */
.glow {
  position: absolute;
  border-radius: 50%;
  filter: blur(100px);
  transition: opacity 0.5s ease;
}

.glow-1 {
  width: 500px;
  height: 500px;
  background: var(--accent-primary);
  bottom: -150px;
  left: -100px;
  animation: pulse 8s ease-in-out infinite;
}

.glow-2 {
  width: 400px;
  height: 400px;
  background: var(--accent-secondary);
  top: -100px;
  right: 100px;
  animation: pulse 10s ease-in-out infinite reverse;
}

@keyframes pulse {
  0%, 100% {
    transform: scale(1);
  }
  50% {
    transform: scale(1.1);
  }
}
</style>
