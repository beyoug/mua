<!--
  UaSelector.svelte
  User Agent 选择器 - 从 AddTaskDialog 中提取，支持预设/自定义/历史记录
-->
<script lang="ts">
    import { Trash2, ChevronRight } from '@lucide/svelte';
    import { fade } from 'svelte/transition';
    import { clickOutside } from '$lib';
    import { appSettings, saveAppSettings } from '$lib/stores/settings';

    interface Props {
        /** 当前选中的 UA 值（空字符串=默认, 'custom'=自定义模式, 其他=具体 UA 字符串） */
        selectedValue: string;
        /** 自定义 UA 输入值 */
        customValue: string;
        /** 选中值变更回调 */
        onValueChange: (value: string) => void;
        /** 自定义 UA 变更回调 */
        onCustomChange: (value: string) => void;
    }

    let { selectedValue, customValue, onValueChange, onCustomChange }: Props = $props();

    // 内置 UA 预设
    const BUILTIN_UAS = [
        { name: 'Chrome', value: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36' },
        { name: 'Firefox', value: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:120.0) Gecko/20100101 Firefox/120.0' },
        { name: 'Safari', value: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Safari/605.1.15' }
    ];

    let isDropdownOpen = $state(false);

    function truncateUa(ua: string) {
        if (ua.length > 40) return ua.substring(0, 37) + '...';
        return ua;
    }

    // 组合展示用的 UA 列表
    const displayUas = $derived([
        { id: 'default', name: '默认', value: '', builtin: true },
        ...($appSettings.uaHistory || []).map((val, index) => ({
            id: `history-${index}`,
            name: truncateUa(val),
            value: val,
            builtin: false
        })),
        ...BUILTIN_UAS.filter(b => !($appSettings.uaHistory || []).includes(b.value)).map((b, index) => ({
            id: `builtin-${index}`,
            name: b.name,
            value: b.value,
            builtin: true
        }))
    ]);

    const activeUaName = $derived.by(() => {
        if (selectedValue === 'custom') return '自定义';
        if (selectedValue === '') return '默认';
        const found = displayUas.find(u => u.value === selectedValue);
        return found ? found.name : truncateUa(selectedValue);
    });

    const isCustomUaInvalid = $derived(selectedValue === 'custom' && !customValue.trim());

    function handleSelect(value: string) {
        onValueChange(value);
        isDropdownOpen = false;
    }

    async function removeHistoryItem(uaValue: string) {
        const history = $appSettings.uaHistory || [];
        const newHistory = history.filter(v => v !== uaValue);
        await saveAppSettings({ ...$appSettings, uaHistory: newHistory });
        if (selectedValue === uaValue) onValueChange('');
    }

    /** 判断给定 UA 是否为内置预设 */
    export function isBuiltinUa(ua: string): boolean {
        return BUILTIN_UAS.some(b => b.value === ua);
    }

    /** 获取有效的 UA 值（解析 custom 模式） */
    export function getEffectiveUa(): string {
        return selectedValue === 'custom' ? customValue : selectedValue;
    }
</script>

<div class="ua-manager" use:clickOutside={() => isDropdownOpen = false}>
    <button
        class="ua-dropdown-trigger"
        class:open={isDropdownOpen}
        onclick={() => isDropdownOpen = !isDropdownOpen}
    >
        <span class="trigger-text">{activeUaName}</span>
        <ChevronRight size={14} class="chevron" />
    </button>

    {#if isDropdownOpen}
        <div class="ua-dropdown-content" transition:fade={{ duration: 150 }}>
            <div class="ua-list-container">
                {#each displayUas as ua}
                    <div class="ua-option" class:active={selectedValue === ua.value && selectedValue !== 'custom'}>
                        <button class="ua-select-btn" onclick={() => handleSelect(ua.value)}>
                            <span class="ua-name">{ua.name}</span>
                        </button>
                        {#if !ua.builtin}
                            <button class="ua-delete-btn" onclick={() => removeHistoryItem(ua.value)} title="删除记录">
                                <Trash2 size={12} />
                            </button>
                        {/if}
                    </div>
                {/each}
                <div class="ua-option" class:active={selectedValue === 'custom'}>
                    <button class="ua-select-btn" onclick={() => handleSelect('custom')}>
                        <span class="ua-name">自定义...</span>
                    </button>
                </div>
            </div>
        </div>
    {/if}

    {#if selectedValue === 'custom'}
        <input
            type="text"
            class="ua-custom-input"
            class:error={isCustomUaInvalid}
            placeholder="输入自定义 User Agent"
            value={customValue}
            oninput={(e) => onCustomChange(e.currentTarget.value)}
            transition:fade={{ duration: 150 }}
        />
    {/if}
</div>

<style>
    .ua-manager { display: flex; flex-direction: column; gap: 8px; position: relative; }
    .ua-dropdown-trigger {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 10px 14px;
        background: var(--input-bg);
        border: 1px solid var(--border-color);
        border-radius: 10px;
        color: var(--text-primary);
        cursor: pointer;
    }
    .ua-dropdown-trigger:hover { border-color: var(--accent-primary); }
    .ua-dropdown-trigger :global(.chevron) { transition: transform 0.2s; }
    .ua-dropdown-trigger.open :global(.chevron) { transform: rotate(90deg); }

    .ua-dropdown-content {
        position: absolute;
        top: 100%;
        left: 0;
        right: 0;
        margin-top: 4px;
        background: var(--overlay-bg, var(--dialog-bg));
        backdrop-filter: blur(20px) saturate(180%);
        border: 1px solid var(--border-color, rgba(255, 255, 255, 0.15));
        border-radius: 12px;
        box-shadow: 0 10px 30px -5px rgba(0, 0, 0, 0.3);
        z-index: 1000;
        overflow: hidden;
    }

    .ua-list-container { max-height: 240px; overflow-y: auto; }
    .ua-option {
        display: flex;
        align-items: center;
        padding: 2px 8px;
    }
    .ua-option:hover { background: var(--surface-hover); }
    .ua-option.active { color: var(--accent-primary); background: color-mix(in srgb, var(--accent-primary) 5%, transparent); }

    .ua-select-btn {
        flex: 1;
        text-align: left;
        padding: 8px;
        background: transparent;
        border: none;
        color: inherit;
        font-size: 13px;
        cursor: pointer;
    }

    .ua-delete-btn {
        padding: 6px;
        background: transparent;
        border: none;
        color: var(--text-tertiary);
        cursor: pointer;
        border-radius: 4px;
    }
    .ua-delete-btn:hover { color: var(--danger-color); background: rgba(239, 68, 68, 0.1); }

    .ua-custom-input {
        margin-top: 8px;
        width: 100%;
        padding: 10px 14px;
        background: var(--input-bg);
        border: 1px solid var(--border-color);
        border-radius: 10px;
        color: var(--text-primary);
        font-size: 13px;
        outline: none;
        transition: all 0.2s;
    }

    .ua-custom-input:focus {
        border-color: var(--accent-primary);
        box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent-primary) 10%, transparent);
    }

    .ua-custom-input.error {
        border-color: var(--danger-color);
        background: color-mix(in srgb, var(--danger-color) 4%, var(--input-bg));
    }

    .ua-custom-input.error:focus {
        box-shadow: 0 0 0 2px color-mix(in srgb, var(--danger-color) 15%, transparent);
    }
</style>
