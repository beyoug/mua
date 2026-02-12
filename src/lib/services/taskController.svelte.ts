import {
    removeTask,
    removeTasks,
    cancelTask,
    cancelTasks,
    addDownloadTasks,
    pauseTask,
    pauseAll,
    resumeAll,
    resumeTask,
    isActiveTask,
    isCompletedTask,
    type DownloadTask,
    type DownloadConfig
} from '$lib';
import * as cmd from '$lib/api/cmd';
import { createLogger } from '$lib/utils/logger';
import { getErrorMessage } from '$lib/utils/errors';

const logger = createLogger('TaskController');

export type NavType = 'active' | 'complete' | 'history';

export class TaskController {
    // Navigation State
    activeNav = $state<NavType>('active');

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



    handleNavChange(nav: NavType) {
        this.activeNav = nav;
        this.exitSelectionMode();
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
    handleTrashClick(currentTasks: DownloadTask[]) {
        // 1. 进入选择模式
        if (!this.isSelectionMode) {
            this.isSelectionMode = true;
            this.selectedIds = new Set();
            return;
        }

        // 2. 如果未选中任何项，则全选
        if (this.selectedIds.size === 0) {
            const next = new Set(this.selectedIds);
            currentTasks.forEach(d => next.add(d.id));
            this.selectedIds = next;
            return;
        }

        // 3. 执行逻辑分流
        // 进行中页面：默认直接移动到历史记录（软删除），不需要弹窗
        if (this.activeNav === 'active') {
            void this.performClear(false);
            return;
        }

        // 历史/已完成页面：
        // 检查选中任务中是否包含"已完成"的任务
        // 如果选中的全是 失败/取消/缺失/进行中 等非完成状态，则直接删除不弹窗
        const selectedTasks = currentTasks.filter(t => this.selectedIds.has(t.id));
        const hasCompletedTask = selectedTasks.some(t => isCompletedTask(t.state));

        if (!hasCompletedTask) {
            // 全是非完成任务 -> 直接删除（含文件）
            void this.performClear(true);
            return;
        }

        // 包含已完成任务 -> 弹窗确认
        const count = this.selectedIds.size;
        let title = '';
        let description = '';

        if (this.activeNav === 'history') {
            title = '删除记录';
            description = `确定要永久删除这 ${count} 条任务记录吗？`;
        } else {
            title = '清空记录';
            description = `确定要清空这 ${count} 条已完成的任务记录吗？`;
        }

        this.clearDialogProps = {
            title,
            description,
            confirmText: '确定'
        };
        this.showClearDialog = true;
    }

    // 执行清理（批量或单项）
    async performClear(deleteFile: boolean) {
        try {
            if (this.itemToDelete) {
                // 单项删除
                await removeTask(this.itemToDelete, deleteFile);
                this.itemToDelete = null;
            } else {
                // 批量删除
                if (this.activeNav === 'active') {
                    // 进行中页面：软删除（取消）
                    await cancelTasks(this.selectedIds);
                } else {
                    // 历史/已完成页面：硬删除
                    await removeTasks(this.selectedIds, deleteFile);
                }
                this.exitSelectionMode();
            }

            this.showClearDialog = false;
        } catch (e) {
            logger.error('Failed to clear tasks', {
                nav: this.activeNav,
                taskId: this.itemToDelete,
                selectedCount: this.selectedIds.size,
                deleteFile,
                error: e
            });
        }
    }

    async handleCancelTask(task: DownloadTask) {
        try {
            if (isActiveTask(task.state)) {
                // 活跃任务（下载/等待/暂停）：软删除（仅取消并保留在历史），无需确认
                await cancelTask(task.id);
            } else if (isCompletedTask(task.state)) {
                // 已完成任务：物理删除记录，需要确认
                this.itemToDelete = task.id;
                this.clearDialogProps = {
                    title: '删除任务',
                    description: '确定要删除这条任务记录吗？',
                    confirmText: '删除'
                };
                this.showClearDialog = true;
            } else {
                // 失败/取消/缺失 等未完成状态：直接物理删除且清理文件，无需确认
                await removeTask(task.id, true);
            }
        } catch (e) {
            logger.error('Failed to cancel or delete task', { taskId: task.id, state: task.state, error: e });
        }
    }

    async handleOpenFolder(id: string) {
        try {
            await cmd.showTaskInFolder(id);
        } catch (e) {
            logger.error('Failed to open task folder', { taskId: id, error: e });
        }
    }

    /**
     * 添加下载任务并自动跳转至进行中
     */
    async handleAddTask(config: DownloadConfig | DownloadConfig[]) {
        try {
            await addDownloadTasks(config);
            this.handleNavChange('active');
        } catch (e) {
            logger.error('Failed to add task', { error: e });
            throw new Error(getErrorMessage(e, '添加任务失败'));
        }
    }

    async handlePauseTask(id: string) {
        try {
            await pauseTask(id);
        } catch (e) {
            logger.error('Failed to pause task', { taskId: id, error: e });
        }
    }

    async handlePauseAll() {
        try {
            await pauseAll();
        } catch (e) {
            logger.error('Failed to pause all tasks', { error: e });
        }
    }

    async handleResumeAll() {
        try {
            await resumeAll();
        } catch (e) {
            logger.error('Failed to resume all tasks', { error: e });
        }
    }

    /**
     * 恢复/重新下载任务并自动跳转至进行中
     */
    async handleResumeTask(id: string) {
        try {
            await resumeTask(id);
            this.handleNavChange('active');
        } catch (e) {
            logger.error('Failed to resume task', { taskId: id, error: e });
            throw new Error(getErrorMessage(e, '恢复任务失败'));
        }
    }
}
