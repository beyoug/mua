import {
    removeTask,
    removeTasks,
    cancelTask,
    cancelTasks,
    type DownloadTask
} from '$lib';
import * as cmd from '$lib/api/cmd';

export type NavType = 'active' | 'completed' | 'history';

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
            this.performClear(false);
            return;
        }

        // 其他页面（已完成/历史）：需要弹窗确认 + 删除文件选项
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
    performClear(deleteFile: boolean) {
        if (this.itemToDelete) {
            // 单项删除
            removeTask(this.itemToDelete, deleteFile);
            this.itemToDelete = null;
        } else {
            // 批量删除
            if (this.activeNav === 'active') {
                // 进行中页面：软删除（取消）
                cancelTasks(this.selectedIds);
            } else {
                // 历史/已完成页面：硬删除
                removeTasks(this.selectedIds, deleteFile);
            }
            this.exitSelectionMode();
        }

        this.showClearDialog = false;
    }

    handleCancelTask(id: string) {
        if (this.activeNav === 'active') {
            // 进行中：软删除（取消），无需确认
            cancelTask(id);
        } else {
            // 历史记录：需要确认是否删除文件
            this.itemToDelete = id;
            this.clearDialogProps = {
                title: '删除任务',
                description: '确定要删除这条任务记录吗？',
                confirmText: '删除'
            };
            this.showClearDialog = true;
        }
    }

    async handleOpenFolder(id: string) {
        try {
            await cmd.showTaskInFolder(id);
        } catch (e) {
            console.error('Failed to open folder', e);
        }
    }
}
