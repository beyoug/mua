<!--
  AdvancedSettingsPanel.svelte
  下载任务高级设置面板 - 从 AddTaskDialog 中拆分
-->
<script lang="ts">
    import { Link, Globe, FileText, Shield, Gauge, ChevronDown } from '@lucide/svelte';
    import UaSelector from './UaSelector.svelte';

    export interface AdvancedSettings {
        selectedUaValue: string;
        customUserAgent: string;
        referer: string;
        headers: string;
        proxy: string;
        maxDownloadLimitValue: string;
        maxDownloadLimitUnit: string;
    }

    interface Props {
        selectedUaValue: string;
        customUserAgent: string;
        referer: string;
        headers: string;
        proxy: string;
        maxDownloadLimitValue: string;
        maxDownloadLimitUnit: string;
        onUaValueChange: (v: string) => void;
        onCustomUaChange: (v: string) => void;
        onRefererChange: (v: string) => void;
        onHeadersChange: (v: string) => void;
        onProxyChange: (v: string) => void;
        onLimitValueChange: (v: string) => void;
        onLimitUnitChange: (v: string) => void;
        uaSelectorRef?: UaSelector;
        onUaSelectorMount?: (ref: UaSelector) => void;
    }

    let {
        selectedUaValue,
        customUserAgent,
        referer,
        headers,
        proxy,
        maxDownloadLimitValue,
        maxDownloadLimitUnit,
        onUaValueChange,
        onCustomUaChange,
        onRefererChange,
        onHeadersChange,
        onProxyChange,
        onLimitValueChange,
        onLimitUnitChange,
        uaSelectorRef = $bindable(),
    }: Props = $props();
</script>

<div class="advanced-panel">
    <div class="panel-body">
        <div class="form-row">
            <label>
                <Globe size={14} />
                <span>User Agent</span>
            </label>
            <UaSelector
                bind:this={uaSelectorRef}
                selectedValue={selectedUaValue}
                customValue={customUserAgent}
                onValueChange={onUaValueChange}
                onCustomChange={onCustomUaChange}
            />
        </div>

        <!-- Referer -->
        <div class="form-row">
            <label>
                <Link size={14} />
                <span>Referer</span>
            </label>
            <input class="ui-field" type="text" placeholder="https://example.com" value={referer} oninput={(e) => onRefererChange(e.currentTarget.value)} />
        </div>

        <!-- 自定义 Header -->
        <div class="form-row">
            <label>
                <FileText size={14} />
                <span>自定义 Header</span>
            </label>
            <textarea 
                placeholder="Key: Value (每行一个)" 
                value={headers}
                oninput={(e) => onHeadersChange(e.currentTarget.value)}
                rows="2"
                class="headers-textarea ui-field"
            ></textarea>
        </div>

        <!-- 代理服务器 -->
        <div class="form-row">
            <label>
                <Shield size={14} />
                <span>代理服务器</span>
            </label>
            <input class="ui-field" type="text" placeholder="[user:pass@]host:port (支持 http/socks5)" value={proxy} oninput={(e) => onProxyChange(e.currentTarget.value)} />
        </div>

        <!-- 速度限制 -->
        <div class="form-row">
            <label>
                <Gauge size={14} />
                <span>速度限制</span>
            </label>
            <div class="input-group ui-input-group">
                <input 
                    type="number" 
                    min="0" 
                    placeholder="0" 
                    class="grouped-input ui-field"
                    value={maxDownloadLimitValue}
                    oninput={(e) => onLimitValueChange(e.currentTarget.value)}
                />
                <div class="input-divider"></div>
                <div class="select-wrapper">
                    <select class="grouped-select ui-field" value={maxDownloadLimitUnit} onchange={(e) => onLimitUnitChange(e.currentTarget.value)}>
                        <option value="M">MB/s</option>
                        <option value="K">KB/s</option>
                    </select>
                    <ChevronDown size={14} class="select-icon" />
                </div>
            </div>
        </div>
    </div>
</div>

<style>
    .advanced-panel {
        padding: 16px;
        display: flex;
        flex-direction: column;
        gap: 14px;
        height: 100%;
        background: color-mix(in srgb, var(--glass-elevated-bg, var(--dialog-bg)) 72%, transparent);
        border-radius: 14px;
    }

    .panel-body {
        display: flex;
        flex-direction: column;
        gap: 10px;
        flex: 1;
    }

    .form-row {
        display: flex;
        flex-direction: column;
        gap: 7px;
        padding: 10px;
        border-radius: 12px;
        background: color-mix(in srgb, var(--surface-hover) 72%, transparent);
    }

    .form-row label {
        display: flex;
        align-items: center;
        gap: 6px;
        font-size: 12px;
        font-weight: 500;
        color: var(--text-secondary);
        width: 100%;
    }

    .form-row input,
    .form-row textarea {
        padding: 11px 12px;
    }

    .headers-textarea {
        width: 100%;
        padding: 10px 14px;
        font-size: 13px;
        resize: vertical;
        min-height: 88px;
        font-family: var(--font-mono, monospace);
        line-height: 1.5;
    }

    .input-group {
        overflow: hidden;
        box-shadow: var(--control-shadow-rest);
    }

    .grouped-input {
        flex: 1;
        background: transparent;
        border: none;
        padding: 11px 12px;
        color: var(--text-primary);
        font-size: 13px;
        outline: none;
        min-width: 0;
    }

    .input-divider {
        width: 1px;
        background: color-mix(in srgb, var(--border-subtle) 56%, transparent);
        margin: 8px 0;
    }

    .select-wrapper {
        position: relative;
        display: flex;
        align-items: center;
        padding-right: 12px;
    }

    .grouped-select {
        background: transparent;
        border: none;
        padding: 0 28px 0 16px;
        color: var(--text-primary);
        font-size: 13px;
        font-weight: 500;
        outline: none;
        cursor: pointer;
        transition: color 0.2s;
        -webkit-appearance: none;
        appearance: none;
        text-align: left;
        z-index: 1;
    }

    .grouped-select:hover {
        color: var(--text-primary);
    }

    .grouped-select option {
        color: #0f172a;
        background: #f8fafc;
    }

    .grouped-select:focus-visible {
        outline: none;
    }

    .grouped-select:hover + :global(.select-icon) {
        color: var(--text-primary);
    }

    :global(.select-icon) {
        position: absolute;
        right: 12px;
        pointer-events: none;
        color: var(--text-tertiary);
        transition: color 0.2s;
    }

    @-moz-document url-prefix() {
        .grouped-select {
            padding: 0 12px;
        }
    }
</style>
