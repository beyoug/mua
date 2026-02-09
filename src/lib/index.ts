/**
 * lib 模块统一导出
 */

// Utils
export { cn } from './utils/cn';
export { lockScroll, unlockScroll, createScrollLockEffect } from './utils/scroll-lock';
export { formatSpeed, formatAddedAt, extractFilenameFromUrl } from './utils/formatters';
export { getEmitRate, getEstimatedParticles, getSpeedMultiplier } from './utils/particles';
export { isValidDownloadUrl, validateUrl, validateDownloadConfig } from './utils/validators';
export {
    DownloadStateGroups,
    isActiveTask,
    isCompletedTask,
    isRemovableTask,
    isPausedTask,
    isDownloadingTask,
    isWaitingTask,
    isErrorTask,
    isCancelledTask,
    isMissingTask,
    isResumableTask,
    isTerminalTask
} from './utils/downloadStates';

// Stores
export {
    activeTasks,
    completedTasks,
    allTasks,
    downloadStats,
    addDownloadTask,
    pauseTask,
    resumeTask,
    cancelTask,
    removeTask,
    removeTasks,
    cancelTasks,
    pauseAll,
    resumeAll,
    hasDownloadingTasks,
    hasPausedTasks
} from './stores/downloadStore';

// Types
export type { DownloadTask, DownloadConfig, DownloadState, DownloadStats } from './types/download';
