import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { get } from 'svelte/store';
import { appSettings, updateAppSettings } from '$lib/stores/settings';
import { isMagnetUrl, isValidDownloadUrl } from '$lib';
import type { DownloadConfig } from '$lib/types/download';
import type UaSelector from '../UaSelector.svelte';
import {
    buildDownloadConfigs,
    hasMixedLinks,
    normalizeUrls,
    validateInputUrls,
    type AdvancedSettingsState
} from './utils';
import { createLogger } from '$lib/utils/logger';

const logger = createLogger('AddTaskDialog');

interface Params {
    onClose: () => void;
    onSubmit: (config: DownloadConfig | DownloadConfig[]) => void | Promise<void>;
    onTorrentSelect: (path: string) => void;
}

export function useAddTaskDialog(params: Params) {
    const { onClose, onSubmit, onTorrentSelect } = params;

    let urls = $state('');
    let savePath = $state(get(appSettings).defaultSavePath || '~/Downloads');
    let filename = $state('');

    let showAdvanced = $state(false);
    let advancedSnapshot = $state<AdvancedSettingsState | null>(null);
    let selectedUaValue = $state('');
    let customUserAgent = $state('');
    let referer = $state('');
    let headers = $state('');
    let proxy = $state('');
    let maxDownloadLimitValue = $state('');
    let maxDownloadLimitUnit = $state('M');

    let validationError = $state('');
    let validationTimer: ReturnType<typeof setTimeout> | null = null;
    let isSubmitting = $state(false);
    let isSelectingFile = $state(false);

    // 监听设置变更，如果 defaultSavePath 改变且当前 savePath 是旧的默认值（或占位符），则自动更新
    $effect(() => {
        const unsubscribe = appSettings.subscribe(settings => {
            // 如果当前 savePath 是占位符 "..."，或者等于旧的 defaultSavePath (假设用户没改过)，则跟随更新
            // 简单起见，如果 savePath 为 "..." 或空，且新的 defaultSavePath 有效，则覆盖
            if ((savePath === '...' || !savePath) && settings.defaultSavePath && settings.defaultSavePath !== '...') {
                savePath = settings.defaultSavePath;
            }
        });
        return unsubscribe;
    });

    const effectiveUserAgent = $derived(
        selectedUaValue === 'custom' ? customUserAgent : selectedUaValue
    );
    const urlValidationError = $derived(validateInputUrls(urls, isValidDownloadUrl));
    const canSubmitNormal = $derived(!urlValidationError && urls.trim().length > 0);
    const isCustomUaInvalid = $derived(
        selectedUaValue === 'custom' && !customUserAgent.trim()
    );
    const hasMixedLinkTypes = $derived.by(() => hasMixedLinks(urls, isMagnetUrl));
    const canUseAdvanced = $derived(!hasMixedLinkTypes);

    function getAdvancedState(): AdvancedSettingsState {
        return {
            selectedUaValue,
            customUserAgent,
            referer,
            headers,
            proxy,
            maxDownloadLimitValue,
            maxDownloadLimitUnit
        };
    }

    function resetForm() {
        urls = '';
        savePath = get(appSettings).defaultSavePath || '~/Downloads';
        filename = '';

        selectedUaValue = '';
        customUserAgent = '';
        referer = '';
        headers = '';
        proxy = '';
        maxDownloadLimitValue = '';
        maxDownloadLimitUnit = 'M';

        validationError = '';
        isSubmitting = false;
        isSelectingFile = false;
        showAdvanced = false;
        advancedSnapshot = null;
    }

    async function saveUaHistoryIfNeeded(uaSelectorRef: UaSelector | undefined) {
        if (!effectiveUserAgent || !canUseAdvanced || !uaSelectorRef) return;
        if (uaSelectorRef.isBuiltinUa(effectiveUserAgent)) return;

        let history = [...(get(appSettings).uaHistory || [])];
        history = [effectiveUserAgent, ...history.filter((ua) => ua !== effectiveUserAgent)];
        if (history.length > 10) history = history.slice(0, 10);
        await updateAppSettings({ uaHistory: history });
    }

    async function handleSubmit(uaSelectorRef: UaSelector | undefined) {
        if (isSubmitting) return;

        const urlError = validateInputUrls(urls, isValidDownloadUrl);
        if (urlError || !urls.trim()) {
            validationError = urlError || '请输入下载链接';
            return;
        }

        isSubmitting = true;

        try {
            const configs: DownloadConfig[] = buildDownloadConfigs({
                urls,
                filename,
                savePath,
                canUseAdvanced,
                advanced: getAdvancedState(),
                effectiveUserAgent
            });

            if (configs.length > 0) {
                await onSubmit(configs);
            }

            await saveUaHistoryIfNeeded(uaSelectorRef);
            resetForm();
            onClose();
        } catch (e) {
            logger.error('Failed to add task', { error: e });
            validationError = typeof e === 'string' ? e : '添加任务失败，请检查 Aria2 服务是否正常';
        } finally {
            isSubmitting = false;
        }
    }

    async function selectFolder() {
        try {
            const selected = await openDialog({
                directory: true,
                multiple: false,
                title: '选择下载目录'
            });
            if (typeof selected === 'string') savePath = selected;
        } catch (e) {
            logger.error('Failed to select download directory', { error: e });
        }
    }

    async function selectTorrentFile() {
        if (isSelectingFile) return;
        isSelectingFile = true;
        try {
            const selected = await openDialog({
                multiple: false,
                filters: [{ name: 'Torrent Files', extensions: ['torrent'] }],
                title: '选择种子文件'
            });

            if (typeof selected === 'string') {
                onTorrentSelect(selected);
            }
        } catch (e) {
            logger.error('Failed to select torrent file', { error: e });
        } finally {
            isSelectingFile = false;
        }
    }

    function openAdvanced() {
        advancedSnapshot = getAdvancedState();
        showAdvanced = true;
    }

    function completeAdvanced() {
        showAdvanced = false;
        advancedSnapshot = null;
    }

    function handleBack() {
        if (advancedSnapshot) {
            selectedUaValue = advancedSnapshot.selectedUaValue;
            customUserAgent = advancedSnapshot.customUserAgent;
            referer = advancedSnapshot.referer;
            headers = advancedSnapshot.headers;
            proxy = advancedSnapshot.proxy;
            maxDownloadLimitValue = advancedSnapshot.maxDownloadLimitValue;
            maxDownloadLimitUnit = advancedSnapshot.maxDownloadLimitUnit;
            advancedSnapshot = null;
        }
        showAdvanced = false;
    }

    function handleUrlBlur() {
        if (validationTimer) {
            clearTimeout(validationTimer);
            validationTimer = null;
        }

        if (urls) {
            urls = normalizeUrls(urls);
        }

        validationError = urls.trim() ? validateInputUrls(urls, isValidDownloadUrl) : '';
    }

    function handleUrlInput() {
        if (validationTimer) clearTimeout(validationTimer);
        validationTimer = setTimeout(() => {
            validationError = urls.trim() ? validateInputUrls(urls, isValidDownloadUrl) : '';
        }, 500);
    }

    $effect(() => {
        return () => {
            if (validationTimer) clearTimeout(validationTimer);
        };
    });

    return {
        get urls() {
            return urls;
        },
        setUrls(value: string) {
            urls = value;
        },
        get savePath() {
            return savePath;
        },
        get filename() {
            return filename;
        },
        setFilename(value: string) {
            filename = value;
        },
        get validationError() {
            return validationError;
        },
        get isSelectingFile() {
            return isSelectingFile;
        },
        get showAdvanced() {
            return showAdvanced;
        },
        get selectedUaValue() {
            return selectedUaValue;
        },
        setSelectedUaValue(value: string) {
            selectedUaValue = value;
        },
        get customUserAgent() {
            return customUserAgent;
        },
        setCustomUserAgent(value: string) {
            customUserAgent = value;
        },
        get referer() {
            return referer;
        },
        setReferer(value: string) {
            referer = value;
        },
        get headers() {
            return headers;
        },
        setHeaders(value: string) {
            headers = value;
        },
        get proxy() {
            return proxy;
        },
        setProxy(value: string) {
            proxy = value;
        },
        get maxDownloadLimitValue() {
            return maxDownloadLimitValue;
        },
        setMaxDownloadLimitValue(value: string) {
            maxDownloadLimitValue = value;
        },
        get maxDownloadLimitUnit() {
            return maxDownloadLimitUnit;
        },
        setMaxDownloadLimitUnit(value: string) {
            maxDownloadLimitUnit = value;
        },
        get canUseAdvanced() {
            return canUseAdvanced;
        },
        get canSubmitNormal() {
            return canSubmitNormal;
        },
        get isSubmitting() {
            return isSubmitting;
        },
        get isCustomUaInvalid() {
            return isCustomUaInvalid;
        },
        handleSubmit,
        handleBack,
        handleUrlBlur,
        handleUrlInput,
        openAdvanced,
        completeAdvanced,
        selectFolder,
        selectTorrentFile
    };
}
