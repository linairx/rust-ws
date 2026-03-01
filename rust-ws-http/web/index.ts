/**
 * rust-ws-http Web Interface
 *
 * 使用 @wasmer/sdk 运行 WASIX 模块
 */

import { init, runWasix } from "@wasmer/sdk";
import wasmUrl from "../../target/wasm32-wasip1/release/rust-ws-http.wasm?url";

// 请求类型
interface Request {
  cmd: string;
  uuid?: string;
  host?: string;
  port?: number;
  name?: string;
  ws_path?: string;
  url?: string;
}

// 响应类型
interface Response {
  ok: boolean;
  data?: Record<string, unknown>;
  error?: string;
}

// 全局 WASM 模块
let wasmModule: WebAssembly.Module | null = null;

/**
 * 初始化 Wasmer SDK
 */
async function initialize(): Promise<WebAssembly.Module> {
  if (wasmModule) {
    return wasmModule;
  }

  await init();
  wasmModule = await WebAssembly.compileStreaming(fetch(wasmUrl));
  return wasmModule;
}

/**
 * 调用 WASM 模块
 */
async function callWasm<T extends Request>(request: T): Promise<Response> {
  const module = await initialize();

  const instance = await runWasix(module, {});

  const stdin = instance.stdin.getWriter();
  const encoder = new TextEncoder();
  const requestJson = JSON.stringify(request);

  await stdin.write(encoder.encode(requestJson));
  await stdin.close();

  const result = await instance.wait();

  if (!result.ok) {
    return {
      ok: false,
      error: `WASM execution failed with code ${result.code}`,
    };
  }

  const stdout = result.stdout;
  if (!stdout) {
    return {
      ok: false,
      error: "No output from WASM module",
    };
  }

  try {
    return JSON.parse(stdout) as Response;
  } catch (e) {
    return {
      ok: false,
      error: `Failed to parse response: ${e}`,
    };
  }
}

// ============ API 函数 ============

/**
 * 健康检查
 */
export async function health(): Promise<Response> {
  return callWasm({ cmd: "health" });
}

/**
 * 显示帮助
 */
export async function help(): Promise<Response> {
  return callWasm({ cmd: "help" });
}

/**
 * 生成订阅内容
 */
export async function generateSubscription(options: {
  uuid: string;
  host: string;
  port: number;
  name?: string;
  ws_path?: string;
}): Promise<Response> {
  return callWasm({
    cmd: "sub",
    uuid: options.uuid,
    host: options.host,
    port: options.port,
    name: options.name,
    ws_path: options.ws_path,
  });
}

/**
 * 生成各协议链接
 */
export async function generateUrls(options: {
  uuid: string;
  host: string;
  port: number;
  name?: string;
  ws_path?: string;
}): Promise<Response> {
  return callWasm({
    cmd: "urls",
    uuid: options.uuid,
    host: options.host,
    port: options.port,
    name: options.name,
    ws_path: options.ws_path,
  });
}

/**
 * 解析代理链接
 */
export async function parseUrl(url: string): Promise<Response> {
  return callWasm({
    cmd: "parse",
    url,
  });
}

// ============ UI 逻辑 ============

function $(id: string): HTMLElement {
  return document.getElementById(id)!;
}

function showOutput(data: unknown) {
  const output = $("output");
  if (typeof data === "string") {
    output.textContent = data;
  } else {
    output.textContent = JSON.stringify(data, null, 2);
  }
}

function getInputValue(id: string): string {
  return ($(id) as HTMLInputElement).value.trim();
}

function getInputNumber(id: string): number {
  return parseInt(getInputValue(id)) || 0;
}

async function main() {
  const output = $("output");
  const statusText = $("status-text");

  // 初始化
  output.textContent = "Initializing Wasmer SDK...";
  statusText.textContent = "Loading...";
  statusText.className = "status-loading";

  try {
    await initialize();
    statusText.textContent = "Ready";
    statusText.className = "status-ready";
    output.textContent = "Ready! Use the tabs above to access different features.";
  } catch (e) {
    statusText.textContent = "Error";
    statusText.className = "status-error";
    output.textContent = `Init failed: ${e}`;
    return;
  }

  // Tab switching
  const tabs = document.querySelectorAll(".tab-btn");
  const panels = document.querySelectorAll(".panel");

  tabs.forEach((tab) => {
    tab.addEventListener("click", () => {
      tabs.forEach((t) => t.classList.remove("active"));
      panels.forEach((p) => p.classList.remove("active"));
      tab.classList.add("active");
      const panelId = tab.getAttribute("data-panel");
      $(panelId!).classList.add("active");
    });
  });

  // Help button
  $("help-btn")?.addEventListener("click", async () => {
    const result = await help();
    showOutput(result);
  });

  // Health button
  $("health-btn")?.addEventListener("click", async () => {
    const result = await health();
    showOutput(result);
  });

  // Generate URLs button
  $("urls-btn")?.addEventListener("click", async () => {
    const uuid = getInputValue("uuid");
    const host = getInputValue("host");
    const port = getInputNumber("port");
    const name = getInputValue("name") || "Proxy Node";
    const wsPath = getInputValue("ws-path");

    if (!uuid || !host || !port) {
      showOutput({ error: "Please fill in UUID, Host, and Port" });
      return;
    }

    const result = await generateUrls({ uuid, host, port, name, ws_path: wsPath });
    showOutput(result);
  });

  // Generate Subscription button
  $("sub-btn")?.addEventListener("click", async () => {
    const uuid = getInputValue("uuid");
    const host = getInputValue("host");
    const port = getInputNumber("port");
    const name = getInputValue("name") || "Proxy Node";
    const wsPath = getInputValue("ws-path");

    if (!uuid || !host || !port) {
      showOutput({ error: "Please fill in UUID, Host, and Port" });
      return;
    }

    const result = await generateSubscription({ uuid, host, port, name, ws_path: wsPath });
    showOutput(result);
  });

  // Parse URL button
  $("parse-btn")?.addEventListener("click", async () => {
    const url = getInputValue("parse-url");

    if (!url) {
      showOutput({ error: "Please enter a proxy URL" });
      return;
    }

    const result = await parseUrl(url);
    showOutput(result);
  });

  // Copy subscription button
  $("copy-btn")?.addEventListener("click", async () => {
    const output = $("output").textContent || "";
    try {
      await navigator.clipboard.writeText(output);
      const btn = $("copy-btn") as HTMLButtonElement;
      const originalText = btn.textContent;
      btn.textContent = "Copied!";
      setTimeout(() => {
        btn.textContent = originalText;
      }, 1500);
    } catch (e) {
      showOutput({ error: "Failed to copy to clipboard" });
    }
  });
}

// 启动
main();
