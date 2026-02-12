import { parseTorrent, type TorrentInfo } from '$lib/api/cmd';
import type { DownloadConfig } from '$lib/types/download';
import { getErrorMessage } from '$lib/utils/errors';
import { createLogger } from '$lib/utils/logger';

interface TorrentDialogResult {
    torrentPath: string;
    selectedFiles?: string;
    trackers: string;
    savePath: string;
}

const logger = createLogger('TorrentDialogController');

interface Params {
    onAddTask: (config: DownloadConfig) => Promise<void>;
}

export class TorrentDialogController {
    showConfig = $state(false);
    pendingInfo = $state<TorrentInfo | null>(null);
    pendingPath = $state('');
    pendingParseError = $state('');

    private parseRequestId = 0;
    private onAddTask: Params['onAddTask'];

    constructor(params: Params) {
        this.onAddTask = params.onAddTask;
    }

    open(path: string) {
        const requestId = ++this.parseRequestId;

        this.pendingPath = path;
        this.pendingInfo = null;
        this.pendingParseError = '';
        this.showConfig = true;

        parseTorrent(path)
            .then((info) => {
                if (requestId !== this.parseRequestId) {
                    return;
                }

                if (info.files.length > 1000) {
                    logger.warn('Large torrent file count', { fileCount: info.files.length, path });
                }

                this.pendingInfo = info;
            })
            .catch((e) => {
                if (requestId !== this.parseRequestId) {
                    return;
                }

                logger.error('Failed to parse torrent', { path, error: e });
                this.pendingParseError = getErrorMessage(e, '种子解析失败，但仍可提交任务');
            });
    }

    openFromDrop(paths: string[]): boolean {
        const torrentFile = paths.find((path) => path.toLowerCase().endsWith('.torrent'));
        if (torrentFile) {
            this.open(torrentFile);
            return true;
        }

        return false;
    }

    async confirm(result: TorrentDialogResult): Promise<void> {
        const normalizedSelectFile = result.selectedFiles?.trim() || undefined;
        const normalizedTrackers = result.trackers.trim() || undefined;

        const config: DownloadConfig = {
            urls: [],
            savePath: result.savePath,
            filename: '',
            userAgent: '',
            referer: '',
            headers: '',
            proxy: '',
            maxDownloadLimit: '',
            torrentConfig: {
                path: result.torrentPath,
                selectFile: normalizedSelectFile,
                trackers: normalizedTrackers
            }
        };

        try {
            await this.onAddTask(config);
            this.reset();
        } catch (e) {
            logger.error('Failed to add task from torrent confirm', {
                path: result.torrentPath,
                error: e
            });
            this.pendingParseError = getErrorMessage(e, '任务添加失败，请检查 Aria2 服务是否正常');
        }
    }

    cancel() {
        this.parseRequestId += 1;
        this.reset();
    }

    private reset() {
        this.showConfig = false;
        this.pendingInfo = null;
        this.pendingPath = '';
        this.pendingParseError = '';
    }
}
