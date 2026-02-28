import { onMount } from "svelte";
import type { TaskController } from "$lib/services/download";
import { registerDragDropHandlers } from "$lib/services/dragDropEvents";
import { createDragDropWatchdog } from "$lib/services/dragDropWatchdog";
import { parseTorrentFile } from "$lib/services/torrent";
import type { DownloadConfig } from "$lib/types/download";
import type { TorrentInfo } from "$lib/types/torrent";
import { createLogger } from "$lib/utils/logger";

interface TorrentDialogResult {
	torrentPath: string;
	selectedFiles?: string;
	trackers: string;
	savePath: string;
}

const logger = createLogger("HomePage");

export function useHomePageOrchestration(controller: TaskController) {
	let showAddDialog = $state(false);
	let showSettings = $state(false);
	let showTorrentConfig = $state(false);

	let isDragOver = $state(false);
	let pendingTorrentInfo = $state<TorrentInfo | null>(null);
	let pendingTorrentPath = $state("");
	let pendingParseError = $state("");
	let torrentParseRequestId = 0;

	onMount(() => {
		let unlistenDragDrop: (() => void) | null = null;
		const watchdog = createDragDropWatchdog(() => {
			isDragOver = false;
		});

		(async () => {
			try {
				unlistenDragDrop = await registerDragDropHandlers({
					onEnter: () => {
						isDragOver = true;
						watchdog.touch();
					},
					onOver: () => {
						watchdog.touch();
					},
					onDrop: (paths) => {
						isDragOver = false;
						watchdog.stop();
						if (paths.length > 0) {
							handleGlobalFileDrop(paths);
						}
					},
					onLeave: () => {
						isDragOver = false;
						watchdog.stop();
					},
				});
			} catch (e) {
				logger.error("Failed to register drag-drop handlers", {
					error: e,
				});
			}
		})();

		return () => {
			if (unlistenDragDrop) {
				unlistenDragDrop();
			}
			watchdog.stop();
		};
	});

	function openAddDialog() {
		showAddDialog = true;
	}

	function closeAddDialog() {
		showAddDialog = false;
	}

	function openSettings() {
		showSettings = true;
	}

	function closeSettings() {
		showSettings = false;
	}

	function openTorrentConfig(path: string) {
		const requestId = ++torrentParseRequestId;
		pendingTorrentPath = path;
		pendingTorrentInfo = null;
		pendingParseError = "";
		showTorrentConfig = true;
		showAddDialog = false;

		parseTorrentFile(path)
			.then((info) => {
				if (requestId !== torrentParseRequestId) return;
				if (info.files.length > 1000) {
					logger.warn("Large torrent file count", {
						fileCount: info.files.length,
						path,
					});
				}
				pendingTorrentInfo = info;
			})
			.catch((e) => {
				if (requestId !== torrentParseRequestId) return;
				logger.error("Failed to parse torrent", { path, error: e });
				pendingParseError =
					typeof e === "string" ? e : "种子解析失败，但仍可提交任务";
			});
	}

	function handleGlobalFileDrop(paths: string[]) {
		const torrentFile = paths.find((p) => p.toLowerCase().endsWith(".torrent"));
		if (torrentFile) {
			openTorrentConfig(torrentFile);
		}
	}

	async function handleTorrentConfirm(result: TorrentDialogResult) {
		const normalizedSelectFile = result.selectedFiles?.trim() || undefined;
		const normalizedTrackers = result.trackers.trim() || undefined;
		const config: DownloadConfig = {
			urls: [],
			savePath: result.savePath,
			filename: "",
			userAgent: "",
			referer: "",
			headers: "",
			proxy: "",
			maxDownloadLimit: "",
			torrentConfig: {
				path: result.torrentPath,
				selectFile: normalizedSelectFile,
				trackers: normalizedTrackers,
			},
		};

		try {
			await controller.addTasks(config);
			showTorrentConfig = false;
			pendingTorrentInfo = null;
			pendingTorrentPath = "";
			pendingParseError = "";
		} catch (e) {
			logger.error("Failed to add task from torrent confirm", {
				path: result.torrentPath,
				error: e,
			});
			pendingParseError = "任务添加失败，请检查 Aria2 服务是否正常";
		}
	}

	function handleTorrentCancel() {
		torrentParseRequestId += 1;
		showTorrentConfig = false;
		pendingTorrentInfo = null;
		pendingTorrentPath = "";
		pendingParseError = "";
	}

	async function handleAddTask(config: DownloadConfig | DownloadConfig[]) {
		await controller.addTasks(config);
		showAddDialog = false;
	}

	return {
		get showAddDialog() {
			return showAddDialog;
		},
		get showSettings() {
			return showSettings;
		},
		get showTorrentConfig() {
			return showTorrentConfig;
		},
		get isDragOver() {
			return isDragOver;
		},
		get pendingTorrentInfo() {
			return pendingTorrentInfo;
		},
		get pendingTorrentPath() {
			return pendingTorrentPath;
		},
		get pendingParseError() {
			return pendingParseError;
		},
		openAddDialog,
		closeAddDialog,
		openSettings,
		closeSettings,
		openTorrentConfig,
		handleTorrentConfirm,
		handleTorrentCancel,
		handleAddTask,
	};
}
