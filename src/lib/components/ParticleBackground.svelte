<!--
  ParticleBackground.svelte
  粒子背景组件 - 根据下载速度动态从左下角释放粒子
-->
<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { totalDownloadSpeed } from '$lib/stores/downloadSpeed';

	interface Particle {
		id: number;
		x: number;
		y: number;
		size: number;
		vx: number;
		vy: number;
		opacity: number;
		life: number;
		maxLife: number;
	}

	let particles = $state<Particle[]>([]);
	let nextId = 0;
	let animationId: number;
	let lastTime = 0;
	let emitAccumulator = 0;
	let currentSpeedMbps = $state(0); // 当前速度 (MB/s)

	// 订阅下载速度 (bytes/s -> MB/s)
	const unsubscribeSpeed = totalDownloadSpeed.subscribe((speed) => {
		currentSpeedMbps = speed / (1024 * 1024);
	});

	// 根据速度计算每秒释放的粒子数 (每 MB/s 增加 0.5 个)
	function getEmitRate(): number {
		if (currentSpeedMbps <= 0) return 0;
		// 基础 6 个/秒 + 每 MB/s 增加 0.5 个，上限 56 个/秒
		const rate = Math.min(6 + currentSpeedMbps * 0.5, 56);
		return rate;
	}

	// 根据速度计算粒子飞行速度倍率 (0-100 MB/s -> 1x-2x)
	function getSpeedMultiplier(): number {
		if (currentSpeedMbps <= 0) return 1;
		return 1 + Math.min(currentSpeedMbps / 100, 1);
	}

	// 计算光晕透明度 (0-100 MB/s -> 0.1-0.4)
	function getGlowOpacity(base: number): number {
		return base + Math.min(currentSpeedMbps / 100, 1) * 0.25;
	}

	// 创建新粒子
	function createParticle(): Particle {
		const isScattered = Math.random() < 0.15;
		const baseAngle = Math.PI / 4; // 45度，向右上方
		const angleVariation = (Math.random() - 0.5) * 0.6; // ±30度偏移
		const angle = baseAngle + angleVariation;
		const speedMultiplier = getSpeedMultiplier();
		const speed = (30 + Math.random() * 40) * speedMultiplier; // 基础速度 × 倍率

		return {
			id: nextId++,
			x: 5 + Math.random() * 10, // 左下角区域 (5%-15% 宽度)
			y: 85 + Math.random() * 10, // 左下角区域 (85%-95% 高度)
			size: 2 + Math.random() * 4, // 2-6px
			vx: Math.cos(angle) * speed * (isScattered ? 0.6 : 1),
			vy: -Math.sin(angle) * speed * (isScattered ? 0.6 : 1),
			opacity: 0,
			life: 0,
			maxLife: 8 + Math.random() * 6 // 8-14秒生命周期
		};
	}

	// 更新粒子状态
	function updateParticles(deltaTime: number) {
		const dt = deltaTime / 1000; // 转换为秒

		// 根据速度释放新粒子
		const emitRate = getEmitRate();
		if (emitRate > 0) {
			emitAccumulator += emitRate * dt;
			while (emitAccumulator >= 1) {
				particles.push(createParticle());
				emitAccumulator -= 1;
			}
		}

		// 更新现有粒子
		particles = particles
			.map((p) => {
				const newLife = p.life + dt;
				const lifeRatio = newLife / p.maxLife;

				// 渐入渐出
				let opacity: number;
				if (lifeRatio < 0.1) {
					opacity = lifeRatio / 0.1 * 0.7;
				} else if (lifeRatio > 0.8) {
					opacity = (1 - lifeRatio) / 0.2 * 0.7;
				} else {
					opacity = 0.7;
				}

				return {
					...p,
					x: p.x + p.vx * dt,
					y: p.y + p.vy * dt,
					life: newLife,
					opacity
				};
			})
			.filter((p) => p.life < p.maxLife && p.x < 110 && p.y > -10);
	}

	// 动画循环
	function animate(currentTime: number) {
		if (lastTime === 0) lastTime = currentTime;
		const deltaTime = Math.min(currentTime - lastTime, 100); // 最大 100ms 避免跳帧
		lastTime = currentTime;

		updateParticles(deltaTime);
		animationId = requestAnimationFrame(animate);
	}

	onMount(() => {
		animationId = requestAnimationFrame(animate);
	});

	onDestroy(() => {
		cancelAnimationFrame(animationId);
		unsubscribeSpeed();
	});
</script>

<div class="particle-container" class:active={currentSpeedMbps > 0}>
	<!-- 背景光晕 -->
	<div class="glow glow-1" style="opacity: {getGlowOpacity(0.15)}"></div>
	<div class="glow glow-2" style="opacity: {getGlowOpacity(0.1)}"></div>
	
	{#each particles as particle (particle.id)}
		<div
			class="particle"
			style="
				width: {particle.size}px;
				height: {particle.size}px;
				left: {particle.x}%;
				top: {particle.y}%;
				opacity: {particle.opacity};
			"
		></div>
	{/each}
</div>

<style>
	.particle-container {
		position: fixed;
		inset: 0;
		overflow: hidden;
		pointer-events: none;
		z-index: 1;
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

	.particle {
		position: absolute;
		background: var(--accent-primary);
		border-radius: 50%;
		box-shadow: 0 0 8px var(--accent-primary), 0 0 16px var(--accent-glow);
		will-change: transform, opacity;
	}
</style>
