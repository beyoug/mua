interface DragDropWatchdog {
    stop: () => void;
    touch: () => void;
}

export function createDragDropWatchdog(
    onTimeout: () => void,
    timeoutMs: number = 300,
    tickMs: number = 100
): DragDropWatchdog {
    let lastDragTime = 0;
    let timer: ReturnType<typeof setInterval> | null = null;

    const stop = () => {
        if (!timer) return;
        clearInterval(timer);
        timer = null;
    };

    const start = () => {
        if (timer) return;
        timer = setInterval(() => {
            if (Date.now() - lastDragTime > timeoutMs) {
                onTimeout();
            }
        }, tickMs);
    };

    const touch = () => {
        lastDragTime = Date.now();
        start();
    };

    return {
        stop,
        touch
    };
}
