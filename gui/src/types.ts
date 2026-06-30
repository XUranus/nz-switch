export interface StatusInfo {
  current_profile: string;
  display_name: string;
  env: Record<string, string>;
  mirrors: Record<string, string>;
  proxy: ProxyInfo | null;
  git: GitInfo | null;
  dns: DnsInfo | null;
  has_local_config: boolean;
}

export interface ProxyInfo {
  address: string;
  proxy_type: string;
}

export interface GitInfo {
  github_mirror: string | null;
  proxy: string | null;
}

export interface DnsInfo {
  servers: string[];
}

export interface ConfigInfo {
  current_profile: string;
  profiles: Record<string, ProfileInfo>;
  config_path: string;
}

export interface ProfileInfo {
  display_name: string;
  env: Record<string, string>;
  mirrors: Record<string, string>;
  proxy: ProxyInfo | null;
  git: GitInfo | null;
  dns: DnsInfo | null;
}

export interface MirrorGroup {
  tool: string;
  display_name: string;
  config_type: string;
  config_path: string | null;
  mirrors: MirrorItem[];
}

export interface MirrorItem {
  name: string;
  display_name: string;
  url: string;
}

export interface DnsPreset {
  name: string;
  servers: string[];
}

export interface DoctorCheck {
  name: string;
  status: "ok" | "warn" | "error";
  message: string;
}

export interface MirrorLatencyInfo {
  name: string;
  url: string;
  latency_ms: number | null;
}

export interface MirrorTestResultInfo {
  tool: string;
  results: MirrorLatencyInfo[];
  recommended: string | null;
}

export interface MirrorTestEventPayload {
  tool: string;
  name: string;
  url: string;
  latency_ms: number | null;
}

export interface IpLocation {
  ip: string;
  country: string;
  region: string;
  city: string;
  is_cn: boolean;
}
