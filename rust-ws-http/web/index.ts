/**
 * rust-ws-http Web Interface
 *
 * 使用 @wasmer/sdk 运行 WASIX 模块
 * 通信模式: stdin/stdout
 */

import { init, runWasix } from "@wasmer/sdk";
// WASM 文件路径 (在 workspace 根目录的 target 中)
import wasmUrl from "../../target/wasm32-wasip1/release/rust-ws-http.wasm?url";

// 请求类型
interface HealthRequest {
  cmd: "health";
}

interface SubRequest {
  cmd: "sub";
  server?: string;
  port?: number;
  password?: string;
  method?: string;
  name?: string;
}

type Request = HealthRequest | SubRequest;

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

  // 运行 WASIX 实例
  const instance = await runWasix(module, {});

  // 写入 stdin
  const stdin = instance.stdin.getWriter();
  const encoder = new TextEncoder();
  const requestJson = JSON.stringify(request);

  await stdin.write(encoder.encode(requestJson));
  await stdin.close();

  // 等待执行完成
  const result = await instance.wait();

  if (!result.ok) {
    return {
      ok: false,
      error: `WASM execution failed with code ${result.code}`,
    };
  }

  // 解析 stdout
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
 * 生成订阅链接
 */
export async function generateSub(options: {
  server: string;
  port: number;
  password: string;
  method?: string;
  name?: string;
}): Promise<Response> {
  return callWasm({
    cmd: "sub",
    server: options.server,
    port: options.port,
    password: options.password,
    method: options.method,
    name: options.name,
  });
}

// ============ UI 逻辑 ============

async function main() {
  const output = document.getElementById("output") as HTMLPreElement;
  const healthBtn = document.getElementById("health-btn") as HTMLButtonElement;
  const subBtn = document.getElementById("sub-btn") as HTMLButtonElement;
  const serverInput = document.getElementById("server") as HTMLInputElement;
  const portInput = document.getElementById("port") as HTMLInputElement;
  const passwordInput = document.getElementById("password") as HTMLInputElement;
  const methodInput = document.getElementById("method") as HTMLInputElement;
  const nameInput = document.getElementById("name") as HTMLInputElement;

  // 初始化
  output.textContent = "Initializing Wasmer SDK...";

  try {
    await initialize();
    output.textContent = "Ready! Click buttons to test.";
  } catch (e) {
    output.textContent = `Init failed: ${e}`;
    return;
  }

  // 健康检查
  healthBtn.addEventListener("click", async () => {
    output.textContent = "Calling health...";
    const result = await health();
    output.textContent = JSON.stringify(result, null, 2);
  });

  // 生成订阅
  subBtn.addEventListener("click", async () => {
    const server = serverInput.value || "example.com";
    const port = parseInt(portInput.value) || 443;
    const password = passwordInput.value || "password";
    const method = methodInput.value || "aes-256-gcm";
    const name = nameInput.value || "Proxy Node";

    output.textContent = "Generating subscription...";
    const result = await generateSub({ server, port, password, method, name });
    output.textContent = JSON.stringify(result, null, 2);
  });
}

// 启动
main();
