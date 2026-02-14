import type { DownloadTask, DownloadConfig } from '$lib/types/download';
import { createLogger } from '$lib/utils/logger';
import { downloadService, type TaskViewNav } from './index';

const logger = createLogger('TaskController');

export class TaskController {
    // Navigation State
    activeNav = $state<TaskViewNav>('active');

    // Selection State
    isSelectionMode = $state(false);
    selectedIds = $state(new Set<string>());

    // Dialog State
    showClearDialog = $state(false);
    clearDialogProps = $state({
        title: '确认清空',
        description: '确定要清空这些任务吗？此操作无法撤销。',
        confirmText: '清空'
    });
    itemToDelete = $state<string | null>(null);

    // Context getters (to be provided by the consumer because they are derived from simple stores)
    // Alternatively, methods accept the current list / task state.



    setNav(nav: TaskViewNav) {
        this.activeNav = nav;
        this.exitSelectionMode();
    }

    private get navAsTaskView(): TaskViewNav {
        return this.activeNav;
    }

    toggleSelection(id: string) {
        const next = new Set(this.selectedIds);
        if (next.has(id)) {
            next.delete(id);
        } else {
            next.add(id);
        }
        this.selectedIds = next;
    }

    exitSelectionMode() {
        this.isSelectionMode = false;
        this.selectedIds = new Set();
    }

    // 垃圾桶按钮点击逻辑
    onTrashClick(currentTasks: DownloadTask[]) {
        const plan = downloadService.planBulkTrash(
            this.navAsTaskView,
            this.isSelectionMode,
            this.selectedIds,
            currentTasks
        );

        if (plan.action === 'enter_selection') {
            this.isSelectionMode = true;
            this.selectedIds = new Set();
            return;
        }

        if (plan.action === 'select_all') {
            const next = new Set(this.selectedIds);
            currentTasks.forEach((task) => next.add(task.id));
            this.selectedIds = next;
            return;
        }

        if (plan.action === 'execute') {
            this.confirmClear(plan.deleteFile);
            return;
        }

        this.clearDialogProps = plan.dialog;
        this.showClearDialog = true;
    }

    // 执行清理（批量或单项）
    async confirmClear(deleteFile: boolean) {
        await downloadService.executeClear(this.navAsTaskView, this.selectedIds, this.itemToDelete, deleteFile);

        if (this.itemToDelete) {
            this.itemToDelete = null;
        } else {
            this.exitSelectionMode();
        }

        this.showClearDialog = false;
    }

    async cancelOrDeleteTask(task: DownloadTask) {
        const plan = downloadService.planSingleTaskRemoval(task);

        if (plan.action === 'cancel') {
            await downloadService.cancelTask(task.id);
            return;
        }

        if (plan.action === 'remove') {
            downloadService.removeTask(task.id, plan.deleteFile);
            return;
        }

        if (plan.action === 'confirm_remove') {
            this.itemToDelete = task.id;
            this.clearDialogProps = plan.dialog;
            this.showClearDialog = true;
        }
    }

    async openTaskFolder(id: string) {
        try {
            await downloadService.openTaskFolder(id);
        } catch (e) {
            logger.error('Failed to open task folder', { taskId: id, error: e });
        }
    }

    /**
     * 添加下载任务并自动跳转至进行中
     */
    async addTasks(config: DownloadConfig | DownloadConfig[]) {
        await downloadService.addDownloadTasks(config);
        this.setNav('active');
    }

    /**
     * 恢复/重新下载任务并自动跳转至进行中
     */
    async resumeTask(id: string) {
        await downloadService.resumeTask(id);
        this.setNav('active');
    }

    async pauseTask(id: string) {
        await downloadService.pauseTask(id);
    }

    async pauseAll() {
        await downloadService.pauseAll();
    }

    async resumeAll() {
        await downloadService.resumeAll();
        this.setNav('active');
    }
}
