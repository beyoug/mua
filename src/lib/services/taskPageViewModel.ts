import type { DownloadTask } from '$lib/types/download';
import type { NavType } from '$lib/services/taskController.svelte';

export interface EmptyStateText {
    title: string;
    hint: string;
}

const EMPTY_STATE_MAP: Record<NavType, EmptyStateText> = {
    active: {
        title: '暂无进行中的任务',
        hint: '点击左侧「添加任务」按钮开始下载'
    },
    complete: {
        title: '暂无已完成的任务',
        hint: '完成的下载任务会显示在这里'
    },
    history: {
        title: '暂无历史记录',
        hint: '所有下载任务的历史会显示在这里'
    }
};

const PAGE_TITLE_MAP: Record<NavType, string> = {
    active: '进行中',
    complete: '已完成',
    history: '历史记录'
};

export function getFilteredTasksByNav(
    nav: NavType,
    active: DownloadTask[],
    complete: DownloadTask[],
    all: DownloadTask[]
): DownloadTask[] {
    if (nav === 'active') {
        return active;
    }

    if (nav === 'complete') {
        return complete;
    }

    return all;
}

export function getPageTitleByNav(nav: NavType): string {
    return PAGE_TITLE_MAP[nav];
}

export function getEmptyStateTextByNav(nav: NavType): EmptyStateText {
    return EMPTY_STATE_MAP[nav];
}
