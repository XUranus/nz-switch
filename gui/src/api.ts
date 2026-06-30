import { invoke } from "@tauri-apps/api/core";
import type {
  StatusInfo,
  ConfigInfo,
  MirrorGroup,
  DnsPreset,
  DoctorCheck,
  MirrorTestResultInfo,
  IpLocation,
} from "./types";

export async function getStatus(): Promise<StatusInfo> {
  return invoke("get_status");
}

export async function switchProfile(name: string): Promise<string> {
  return invoke("switch_profile", { name });
}

export async function getConfig(): Promise<ConfigInfo> {
  return invoke("get_config");
}

export async function listMirrors(tool?: string): Promise<MirrorGroup[]> {
  return invoke("list_mirrors", { tool: tool || null });
}

export async function setMirror(tool: string, source: string): Promise<string> {
  return invoke("set_mirror", { tool, source });
}

export async function resetMirror(tool: string): Promise<string> {
  return invoke("reset_mirror", { tool });
}

export async function getDnsPresets(): Promise<DnsPreset[]> {
  return invoke("get_dns_presets");
}

export async function getCurrentDns(): Promise<string[]> {
  return invoke("get_current_dns");
}

export async function runDoctor(): Promise<DoctorCheck[]> {
  return invoke("run_doctor");
}

export async function resetConfig(): Promise<string> {
  return invoke("reset_config");
}

export async function getRawConfig(): Promise<{ toml: string; path: string }> {
  return invoke("get_raw_config");
}

export async function saveRawConfig(toml: string): Promise<string> {
  return invoke("save_raw_config", { toml });
}

export async function testMirrors(tool?: string): Promise<MirrorTestResultInfo[]> {
  return invoke("test_mirrors", { tool: tool || null });
}

/** 启动流式测速，通过回调接收每个结果 */
export async function testMirrorsStreaming(tool?: string): Promise<void> {
  return invoke("test_mirrors_streaming", { tool: tool || null });
}

export async function getPlatformInfo(): Promise<{ id: string; name: string }> {
  return invoke("get_platform_info");
}

export async function getInstalledTools(): Promise<string[]> {
  return invoke("get_installed_tools");
}

/** 检测所有已安装工具的当前镜像源（读取系统实际配置） */
export async function detectMirrors(): Promise<Record<string, string>> {
  return invoke("detect_mirrors");
}

/** 获取当前 IP 归属地 */
export async function getIpLocation(): Promise<IpLocation> {
  return invoke("get_ip_location");
}
