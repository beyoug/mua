export {
    activeTasks,
    allTasks,
    completeTasks,
    downloadStats,
    hasDownloadingTasks,
    hasPausedTasks
} from './downloadStore/selectors';

export {
    addDownloadTasks,
    cancelTask,
    cancelTasks,
    pauseAll,
    pauseTask,
    removeTask,
    removeTasks,
    resumeAll,
    resumeTask
} from './downloadStore/operations';

import { initializeTaskSync } from './downloadStore/sync';

if (typeof window !== 'undefined') {
    initializeTaskSync();
}
