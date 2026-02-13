import { createMutation, createQueryEffect, type MutationResult } from './state';
import { APIFetch } from './state/errorHandle';
import type {
	McpServerCreateReq,
	McpServerCreateResp,
	McpServerDeleteReq,
	McpServerDeleteResp,
	McpServerListResp,
	McpServerWriteReq,
	McpServerWriteResp,
	McpCheckReq,
	McpCheckResp
} from './types';

let servers = $state<McpServerListResp | undefined>(undefined);

export function useMcpServersQueryEffect() {
	createQueryEffect<Record<string, never>, McpServerListResp>({
		path: 'mcp/list',
		body: {},
		updateData: (data) => {
			servers = data;
		}
	});
}

export function getMcpServers(): McpServerListResp | undefined {
	return servers;
}

async function refreshServers() {
	const res = await APIFetch<McpServerListResp>('mcp/list', {});
	if (res) servers = res;
}

export function createMcpServer(): MutationResult<McpServerCreateReq, McpServerCreateResp> {
	return createMutation({
		path: 'mcp/create',
		onSuccess() {
			void refreshServers();
		}
	});
}

export function deleteMcpServer(): MutationResult<McpServerDeleteReq, McpServerDeleteResp> {
	return createMutation({
		path: 'mcp/delete',
		onSuccess(_data, param) {
			if (servers !== undefined) {
				servers = {
					...servers,
					list: servers.list.filter((s) => s.id !== param.id)
				};
			}
		}
	});
}

export function updateMcpServer(): MutationResult<McpServerWriteReq, McpServerWriteResp> {
	return createMutation({
		path: 'mcp/write',
		onSuccess() {
			void refreshServers();
		}
	});
}

export function checkMcpConfig(): MutationResult<McpCheckReq, McpCheckResp> {
	return createMutation({
		path: 'mcp/check'
	});
}

export async function readMcpServerConfig(id: number): Promise<string> {
	const res = await APIFetch<McpServerListResp>('mcp/list', {});
	if (!res) throw new Error('No response from server');
	const server = res.list.find((s) => s.id === id);
	if (!server) throw new Error(`MCP server id=${id} not found`);
	return server.config_raw;
}

export const defaultMcpConfig = [
	'# MCP Server Configuration',
	'name = "my-server"',
	'enabled = true',
	'transport = "stdio"',
	'',
	'[stdio]',
	'command = "npx"',
	'args = ["-y", "@modelcontextprotocol/server-everything"]',
	'# env = { DEBUG = "mcp:*" }',
	'',
	'# Which chat modes can use this server\'s tools',
	'# Options: "normal", "search", "research"',
	'attached_modes = ["normal", "search", "research"]'
].join('\n');
