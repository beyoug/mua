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
            class="ui-field"
            class:error={!!validationError}
        ></textarea>
    </div>

    <div class="form-group">
        <label>
            <FolderOpen size={14} />
            <span>保存位置</span>
        </label>
        <button class="path-selector ui-field" onclick={onSelectFolder}>
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
            class="text-input ui-field"
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
        gap: 12px;
        flex: 1;
    }

    .form-group {
        display: flex;
        flex-direction: column;
        gap: 7px;
        padding: 10px;
        border-radius: 12px;
        background: color-mix(in srgb, var(--surface-hover) 72%, transparent);
    }

    .form-group label {
        display: flex;
        align-items: center;
        gap: 6px;
        font-size: 12px;
        font-weight: 500;
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
        font-size: 11px;
        line-height: 1.35;
        color: var(--semantic-danger);
        opacity: 0.95;
    }

    textarea,
    input,
    .path-selector {
        padding: 11px 12px;
        font-size: 13px;
    }

    textarea {
        height: 108px;
        resize: none;
        white-space: pre-wrap;
        overflow-wrap: anywhere;
        overflow-x: hidden;
    }

    textarea.error {
        border-color: var(--semantic-danger);
    }

    textarea.error:focus {
        border-color: var(--semantic-danger);
        box-shadow: var(--focus-ring-danger);
    }

    .path-selector {
        display: flex;
        align-items: center;
        justify-content: space-between;
        cursor: pointer;
        text-align: left;
    }

    .path-selector:hover {
        background: var(--control-bg-hover);
        box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent-primary) 10%, transparent);
    }

    .path-selector:focus-visible,
    .btn-xs-secondary:focus-visible,
    textarea:focus-visible,
    input:focus-visible {
        outline: none;
        box-shadow: var(--focus-ring);
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
        background: color-mix(in srgb, var(--surface-hover) 88%, transparent);
        border: none;
        color: var(--text-secondary);
        border-radius: 8px;
        font-size: 12px;
        cursor: pointer;
        transition: all 0.2s;
        box-shadow: var(--control-shadow-rest);
    }

    .btn-xs-secondary:hover {
        background: color-mix(in srgb, var(--accent-primary) 10%, transparent);
        color: var(--text-primary);
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
