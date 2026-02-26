import { allTasks } from '$lib';
import { fromStore } from 'svelte/store';
import type { DownloadTask } from '$lib/types/download';

export function useTaskDetailsState() {
    const allTasksState = fromStore(allTasks);

    let detailsTaskId = $state<string | null>(null);

    const detailsTask = $derived(
        allTasksState.current.find((t) => t.id === detailsTaskId) || null
    );

    const showDetailsModal = $derived(detailsTask !== null);

    function openDetails(task: DownloadTask) {
        detailsTaskId = task.id;
    }

    function closeDetails() {
        detailsTaskId = null;
    }

    return {
        detailsTask,
        showDetailsModal,
        openDetails,
        closeDetails
    };
}
