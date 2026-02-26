import {
    activeTasks,
    completeTasks,
    allTasks,
    hasDownloadingTasks,
    hasPausedTasks,
    isRemovableTask
} from '$lib';
import { fromStore } from 'svelte/store';
import type { DownloadTask } from '$lib/types/download';
import type { TaskController } from '$lib/services/download';

interface EmptyStateText {
    title: string;
    hint: string;
}

export function useTaskViewState(controller: TaskController) {
    const activeTasksState = fromStore(activeTasks);
    const completeTasksState = fromStore(completeTasks);
    const allTasksState = fromStore(allTasks);

    const filteredTasks = $derived.by<DownloadTask[]>(() => {
        switch (controller.activeNav) {
            case 'active':
                return activeTasksState.current;
            case 'complete':
                return completeTasksState.current;
            case 'history':
                return allTasksState.current;
            default:
                return allTasksState.current;
        }
    });

    const pageTitle = $derived.by(() => {
        switch (controller.activeNav) {
            case 'active':
                return '进行中';
            case 'complete':
                return '已完成';
            case 'history':
                return '历史记录';
            default:
                return '历史记录';
        }
    });

    const emptyStateText = $derived.by<EmptyStateText>(() => {
        switch (controller.activeNav) {
            case 'active':
                return {
                    title: '暂无进行中的任务',
                    hint: '点击左侧「添加任务」按钮开始下载'
                };
            case 'complete':
                return {
                    title: '暂无已完成的任务',
                    hint: '完成的下载任务会显示在这里'
                };
            case 'history':
                return {
                    title: '暂无历史记录',
                    hint: '所有下载任务的历史会显示在这里'
                };
            default:
                return {
                    title: '暂无任务',
                    hint: '点击左侧「添加任务」按钮开始下载'
                };
        }
    });

    const hasDownloading = $derived(hasDownloadingTasks(filteredTasks));
    const hasPaused = $derived(hasPausedTasks(filteredTasks));
    const hasRemovable = $derived(filteredTasks.some((t) => isRemovableTask(t.state)));

    let prevActiveIds: string[] = [];
    $effect(() => {
        const currentIds = activeTasksState.current.map((d) => d.id);

        if (controller.activeNav === 'active' && prevActiveIds.length > 0 && currentIds.length === 0) {
            const allCompleted = prevActiveIds.every((id) => completeTasksState.current.some((t) => t.id === id));

            if (allCompleted) {
                controller.setNav('complete');
            }
        }
        prevActiveIds = currentIds;
    });

    return {
        filteredTasks,
        pageTitle,
        emptyStateText,
        hasDownloading,
        hasPaused,
        hasRemovable
    };
}
