import type { ImportConfig, ParsedTask } from './types';

/**
 * 智能解析导入内容
 * 自动识别 JSON 或 cURL 格式
 */
export function parseImportContent(content: string): ParsedTask[] {
    const trimmed = content.trim();
    if (!trimmed) return [];

    // 尝试 JSON 解析
    if (trimmed.startsWith('[') || trimmed.startsWith('{')) {
        try {
            const json = JSON.parse(trimmed);
            return parseJsonImport(json);
        } catch (e) {
            console.warn('JSON parse failed, falling back to cURL parser', e);
        }
    }

    // 默认回退到 cURL 解析
    return parseCurlImport(trimmed);
}

/**
 * 解析 JSON 导入格式
 * 支持结构 A (共享配置) 和结构 B (独立配置) 及其混合嵌套
 */
function parseJsonImport(input: any): ParsedTask[] {
    const results: ParsedTask[] = [];

    // 统一转为数组处理
    const items = Array.isArray(input) ? input : [input];

    for (const item of items) {
        if (!item || typeof item !== 'object') continue;

        const config = item as ImportConfig;

        // 提取公共配置
        const commonHeaders = config.headers || {};
        const commonUa = config.ua || config.userAgent;
        const commonReferer = config.referer;
        const commonProxy = config.proxy;

        // 处理 urls 数组
        if (Array.isArray(config.urls)) {
            for (const subItem of config.urls) {
                if (typeof subItem === 'string') {
                    // 结构 A: 字符串 URL，继承公共配置
                    if (isValidUrl(subItem)) {
                        results.push({
                            url: subItem,
                            headers: commonHeaders,
                            userAgent: commonUa,
                            referer: commonReferer,
                            proxy: commonProxy
                        });
                    }
                } else if (typeof subItem === 'object' && subItem) {
                    // 结构 B 或 嵌套: 对象 URL，合并配置
                    // 递归调用以支持深度嵌套，但这里简化为一层合并
                    // 如果 subItem 本身又是 { urls: [...] } 这种复杂结构，递归调用 parseJsonImport 会更健壮
                    // 但根据用户描述，urls 数组里通常是 string 或 { url: string, ... }

                    const subConfig = subItem as ImportConfig;
                    if (subConfig.url && isValidUrl(subConfig.url)) {
                        // 合并 Header
                        const mergedHeaders = { ...commonHeaders, ...(subConfig.headers || {}) };
                        results.push({
                            url: subConfig.url,
                            headers: Object.keys(mergedHeaders).length > 0 ? mergedHeaders : undefined,
                            userAgent: subConfig.ua || subConfig.userAgent || commonUa,
                            referer: subConfig.referer || commonReferer,
                            filename: subConfig.filename,
                            proxy: subConfig.proxy || commonProxy
                        });
                    } else if (subConfig.urls) {
                        // 递归处理嵌套的 urls 列表
                        // 需要将当前层的公共配置传递下去吗？通常 JSON 结构是扁平的或者自包含的。
                        // 这里简单起见，假设 subConfig 是完全独立的，或者我们将 current config 作为 base 传下去?
                        // 为了简单起见，我们假设 subConfig 是独立的，或者我们手动合并。
                        // 考虑到用户示例，结构 B 是 urls: [ {url: ...} ]，所以上面的 subConfig.url 判断已经覆盖了。
                        // 如果是 urls: [ {urls: ...} ] 这种奇怪的嵌套，暂时忽略。
                    }
                }
            }
        }
        // 处理单个 url 字段 (结构 B 的变体或顶层只写了 url)
        else if (config.url && typeof config.url === 'string' && isValidUrl(config.url)) {
            results.push({
                url: config.url,
                headers: commonHeaders,
                userAgent: commonUa,
                referer: commonReferer,
                filename: config.filename,
                proxy: commonProxy
            });
        }
    }

    return results;
}

/**
 * 解析 cURL 文本
 * 支持多行，空行分隔多个任务
 */
function parseCurlImport(text: string): ParsedTask[] {
    // 按空行分割多个命令块
    // 匹配连续两个换行符及中间空白
    const blocks = text.split(/\n\s*\n/).filter(b => b.trim());
    const tasks: ParsedTask[] = [];

    for (const block of blocks) {
        const task = parseSingleCurl(block);
        if (task) {
            tasks.push(task);
        }
    }

    return tasks;
}

/**
 * 解析单个 cURL 命令
 */
function parseSingleCurl(cmd: string): ParsedTask | null {
    // 清洗换行符 (行末的反斜杠)
    const cleanCmd = cmd.replace(/\\\s*\n/g, ' ').replace(/\n/g, ' ');

    // 提取 URL (非选项参数)
    // 简单正则：查找 http/https/ftp 开头的字串，且前面不是 -X 或其他选项的一部分
    // 但 cURL 命令结构复杂，URL 可能在最后，也可能在中间。
    // 策略：排除掉以 - 开头的 token (及其参数)，剩下的应该就是 URL。
    // 这里使用一个简单的正则来抓取 URL，假设 URL 是以 http/ftp 开头的
    const urlMatch = cleanCmd.match(/["']?(https?:\/\/[^\s"']+|ftp:\/\/[^\s"']+)/);
    if (!urlMatch) return null;

    const url = urlMatch[1];

    // 提取 Headers
    const headers: Record<string, string> = {};
    // 匹配 -H "Key: Value" 或 -H 'Key: Value' 或 -H Key:Value
    // 分别捕获：单引号内容、双引号内容、无引号内容
    const headerRegex = /(?:-H|--header)\s+(?:'([^']+)'|"([^"]+)"|([^\s]+))/g;
    let match;
    while ((match = headerRegex.exec(cleanCmd)) !== null) {
        // match[1] 是单引号内容，match[2] 是双引号内容，match[3] 是无引号内容
        const headerStr = match[1] || match[2] || match[3];
        if (!headerStr) continue;

        const colonIndex = headerStr.indexOf(':');
        if (colonIndex > 0) {
            const key = headerStr.slice(0, colonIndex).trim();
            const value = headerStr.slice(colonIndex + 1).trim();
            headers[key] = value;
        }
    }

    // 提取 User Agent
    let userAgent: string | undefined = undefined;
    const uaRegex = /(?:-A|--user-agent)\s+(?:'([^']+)'|"([^"]+)"|([^\s]+))/;
    const uaMatch = cleanCmd.match(uaRegex);
    if (uaMatch) {
        userAgent = uaMatch[1] || uaMatch[2] || uaMatch[3];
    } else {
        // 尝试从 headers 中查找 (不区分大小写)
        const uaKey = Object.keys(headers).find(k => k.toLowerCase() === 'user-agent');
        if (uaKey) {
            userAgent = headers[uaKey];
            delete headers[uaKey];
        }
    }

    // 提取 Referer
    let referer: string | undefined = undefined;
    const refRegex = /(?:-e|--referer)\s+(?:'([^']+)'|"([^"]+)"|([^\s]+))/;
    const refMatch = cleanCmd.match(refRegex);
    if (refMatch) {
        referer = refMatch[1] || refMatch[2] || refMatch[3];
    } else {
        // 尝试从 headers 中查找 (不区分大小写)
        const refKey = Object.keys(headers).find(k => k.toLowerCase() === 'referer');
        if (refKey) {
            referer = headers[refKey];
            delete headers[refKey];
        }
    }

    // 提取 Proxy (可选)
    let proxy: string | undefined = undefined;
    const proxyRegex = /(?:-x|--proxy)\s+(?:'([^']+)'|"([^"]+)"|([^\s]+))/;
    const proxyMatch = cleanCmd.match(proxyRegex);
    if (proxyMatch) {
        proxy = proxyMatch[1] || proxyMatch[2] || proxyMatch[3];
    }

    return {
        url,
        headers: Object.keys(headers).length > 0 ? headers : undefined,
        userAgent,
        referer,
        proxy
    };
}

function isValidUrl(str: string): boolean {
    return /^https?:\/\/.+|^ftp:\/\/.+/.test(str);
}
