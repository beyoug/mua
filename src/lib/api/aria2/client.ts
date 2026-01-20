/**
 * aria2 WebSocket JSON-RPC 客户端
 * 负责与 aria2c 进程建立 WebSocket 连接并发送 RPC 请求
 */

import type { Aria2Config, JsonRpcRequest, JsonRpcResponse } from './types';
import { DEFAULT_ARIA2_CONFIG } from '$lib/config/constants';

type RpcCallback = (response: JsonRpcResponse) => void;

/**
 * aria2 WebSocket 客户端
 */
export class Aria2Client {
    private ws: WebSocket | null = null;
    private config: Aria2Config;
    private callbacks: Map<string, RpcCallback> = new Map();
    private messageId = 0;
    private reconnectAttempts = 0;
    private maxReconnectAttempts = 5;

    constructor(config: Partial<Aria2Config> = {}) {
        this.config = { ...DEFAULT_ARIA2_CONFIG, ...config };
    }

    /**
     * 获取 WebSocket URL
     */
    private getUrl(): string {
        const protocol = this.config.secure ? 'wss' : 'ws';
        return `${protocol}://${this.config.host}:${this.config.port}/jsonrpc`;
    }

    /**
     * 连接到 aria2
     */
    async connect(): Promise<void> {
        return new Promise((resolve, reject) => {
            try {
                this.ws = new WebSocket(this.getUrl());

                this.ws.onopen = () => {
                    this.reconnectAttempts = 0;
                    console.log('[Aria2] Connected');
                    resolve();
                };

                this.ws.onmessage = (event) => {
                    this.handleMessage(event.data);
                };

                this.ws.onerror = (error) => {
                    console.error('[Aria2] WebSocket error:', error);
                    reject(error);
                };

                this.ws.onclose = () => {
                    console.log('[Aria2] Disconnected');
                    this.handleDisconnect();
                };
            } catch (error) {
                reject(error);
            }
        });
    }

    /**
     * 断开连接
     */
    disconnect(): void {
        if (this.ws) {
            this.ws.close();
            this.ws = null;
        }
    }

    /**
     * 发送 RPC 请求
     */
    async call<T>(method: string, params: unknown[] = []): Promise<T> {
        if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
            throw new Error('WebSocket is not connected');
        }

        const id = `${++this.messageId}`;
        const request: JsonRpcRequest = {
            jsonrpc: '2.0',
            id,
            method,
            params: this.config.secret ? [`token:${this.config.secret}`, ...params] : params
        };

        return new Promise((resolve, reject) => {
            this.callbacks.set(id, (response) => {
                if (response.error) {
                    reject(new Error(response.error.message));
                } else {
                    resolve(response.result as T);
                }
            });

            this.ws!.send(JSON.stringify(request));
        });
    }

    /**
     * 处理收到的消息
     */
    private handleMessage(data: string): void {
        try {
            const response: JsonRpcResponse = JSON.parse(data);
            const callback = this.callbacks.get(response.id);
            if (callback) {
                callback(response);
                this.callbacks.delete(response.id);
            }
        } catch (error) {
            console.error('[Aria2] Failed to parse message:', error);
        }
    }

    /**
     * 处理断开连接
     */
    private handleDisconnect(): void {
        if (this.reconnectAttempts < this.maxReconnectAttempts) {
            this.reconnectAttempts++;
            const delay = Math.min(1000 * 2 ** this.reconnectAttempts, 30000);
            console.log(`[Aria2] Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts})`);
            setTimeout(() => this.connect(), delay);
        }
    }

    /**
     * 检查连接状态
     */
    get isConnected(): boolean {
        return this.ws?.readyState === WebSocket.OPEN;
    }
}

/**
 * 创建 aria2 客户端单例
 */
let clientInstance: Aria2Client | null = null;

export function getAria2Client(config?: Partial<Aria2Config>): Aria2Client {
    if (!clientInstance) {
        clientInstance = new Aria2Client(config);
    }
    return clientInstance;
}
