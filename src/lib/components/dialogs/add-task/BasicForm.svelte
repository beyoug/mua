<script lang="ts">
    import {
        AlertCircle,
        FileText,
        FileUp,
        FolderOpen,
        Link,
    } from "@lucide/svelte";
    import { fade } from "svelte/transition";
    import { compactPath } from "$lib/utils/path";

    interface Props {
        urls: string;
        filename: string;
        savePath: string;
        validationError: string;
        isSelectingFile: boolean;
        onUrlsChange: (value: string) => void;
        onFilenameChange: (value: string) => void;
        onUrlInput: () => void;
        onUrlBlur: () => void;
        onSelectFolder: () => void;
        onSelectTorrentFile: () => void;
    }

    const {
        urls,
        filename,
        savePath,
        validationError,
        isSelectingFile,
        onUrlsChange,
        onFilenameChange,
        onUrlInput,
        onUrlBlur,
        onSelectFolder,
        onSelectTorrentFile,
    }: Props = $props();

    let displayPath = $state("");

    $effect(() => {
        compactPath(savePath).then((p) => (displayPath = p));
    });
</script>

<div class="dialog-body" in:fade={{ duration: 150 }}>
    <div class="form-group">
        <label for="urls">
            <Link size={14} />
            <span>下载链接 (支持 Magnet)</span>
            <div class="label-actions">
                <button
                    class="btn-xs-secondary"
                    onclick={onSelectTorrentFile}
                    disabled={isSelectingFile}
                >
                    {#if isSelectingFile}
                        <span
                            style="animation: spin 1s linear infinite; display: inline-flex;"
                        >
                            <AlertCircle size={12} />
                        </span>
                        <span>打开中...</span>
                    {:else}
                        <FileUp size={12} />
                        <span>打开种子文件</span>
                    {/if}
                </button>
            </div>
        </label>

        {#if validationError}
            <span class="error-inline" style="margin-top: 4px;">
                <AlertCircle size={12} />
                {validationError}
            </span>
        {/if}

        <textarea
            id="urls"
            placeholder="输入 HTTP/HTTPS/Magnet 链接，每行一个"
            value={urls}
            oninput={(event) => {
                onUrlsChange(
                    (event.currentTarget as HTMLTextAreaElement).value,
                );
                onUrlInput();
            }}
            onblur={onUrlBlur}
            class:error={!!validationError}
        ></textarea>
    </div>

    <div class="form-group">
        <label>
            <FolderOpen size={14} />
            <span>保存位置</span>
        </label>
        <button class="path-selector" onclick={onSelectFolder}>
            <span class="path-text" title={savePath}
                >{displayPath || savePath}</span
            >
            <FolderOpen size={14} />
        </button>
    </div>

    <div class="form-group">
        <label>
            <FileText size={14} />
            <span>保存文件名</span>
        </label>
        <input
            type="text"
            class="text-input"
            placeholder="留空则使用默认文件名"
            value={filename}
            oninput={(event) =>
                onFilenameChange(
                    (event.currentTarget as HTMLInputElement).value,
                )}
        />
    </div>
</div>

<style>
    .dialog-body {
        display: flex;
        flex-direction: column;
        gap: 16px;
        flex: 1;
    }

    .form-group {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .form-group label {
        display: flex;
        align-items: center;
        gap: 6px;
        font-size: 13px;
        color: var(--text-secondary);
        width: 100%;
    }

    .label-actions {
        margin-left: auto;
        display: flex;
        gap: 8px;
    }

    .error-inline {
        display: flex;
        align-items: center;
        gap: 4px;
        font-size: 12px;
        color: var(--danger-color);
    }

    textarea,
    input,
    .path-selector {
        padding: 12px 14px;
        background: var(--input-bg, rgba(255, 255, 255, 0.05));
        border: 1px solid var(--border-color, rgba(255, 255, 255, 0.1));
        border-radius: 10px;
        color: var(--text-primary);
        font-size: 14px;
        outline: none;
        transition: all 0.2s ease;
    }

    textarea:focus,
    input:focus {
        border-color: var(--accent-primary);
        box-shadow: 0 0 0 3px
            color-mix(in srgb, var(--accent-primary) 15%, transparent);
    }

    textarea {
        height: 100px;
        resize: none;
        white-space: nowrap;
        overflow-x: auto;
    }

    textarea.error {
        border-color: var(--danger-color);
    }

    .path-selector {
        display: flex;
        align-items: center;
        justify-content: space-between;
        cursor: pointer;
        text-align: left;
    }

    .path-selector:hover {
        border-color: var(--accent-primary);
    }

    .path-text {
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .btn-xs-secondary {
        display: flex;
        align-items: center;
        gap: 4px;
        padding: 4px 8px;
        background: var(--bg-tertiary);
        border: 1px solid var(--border-color);
        color: var(--text-secondary);
        border-radius: 6px;
        font-size: 12px;
        cursor: pointer;
        transition: all 0.2s;
    }

    .btn-xs-secondary:hover {
        background: var(--bg-hover);
        color: var(--primary-color);
        border-color: var(--primary-color);
    }

    @keyframes spin {
        from {
            transform: rotate(0deg);
        }

        to {
            transform: rotate(360deg);
        }
    }
</style>
