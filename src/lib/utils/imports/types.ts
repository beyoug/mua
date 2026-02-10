export interface ImportConfig {
    url?: string; // 单个 URL (用于结构 B)
    urls?: string[] | ImportConfig[]; // URL 列表 或 嵌套配置列表 (结构 A/B 混合)
    headers?: Record<string, string>;
    ua?: string;
    userAgent?: string; // 兼容 ua 字段
    filename?: string;
    referer?: string;
    proxy?: string;
}

export interface ParsedTask {
    url: string;
    headers?: Record<string, string>;
    userAgent?: string;
    filename?: string;
    referer?: string;
    proxy?: string;
}
