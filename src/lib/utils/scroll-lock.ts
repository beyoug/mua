/**
 * scroll-lock.ts - 滚动锁定工具
 * 用于对话框/浮层打开时锁定背景滚动
 */

/**
 * 锁定滚动
 */
export function lockScroll(): void {
    if (typeof document === 'undefined') return;

    document.body.classList.add('no-scroll');
    document.documentElement.classList.add('no-scroll');
}

/**
 * 解锁滚动
 */
export function unlockScroll(): void {
    if (typeof document === 'undefined') return;

    document.body.classList.remove('no-scroll');
    document.documentElement.classList.remove('no-scroll');
}

/**
 * Svelte effect 滚动锁定 Hook
 * @param isOpen 是否打开（控制锁定状态）
 * 
 * 使用示例:
 * ```ts
 * $effect(() => {
 *   return createScrollLockEffect(open);
 * });
 * ```
 */
export function createScrollLockEffect(isOpen: boolean): (() => void) | undefined {
    if (isOpen) {
        lockScroll();
    } else {
        unlockScroll();
    }

    // 返回清理函数
    return () => {
        unlockScroll();
    };
}
