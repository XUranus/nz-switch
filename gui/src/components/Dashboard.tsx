import React, { useState, useEffect, useCallback, useMemo } from "react";
import type { StatusInfo, MirrorGroup, IpLocation } from "../types";
import { IconRefresh, IconGlobe, IconSettings, getToolIcon, IconCheck } from "../icons";
import { detectMirrors, getInstalledTools, listMirrors, getIpLocation } from "../api";

interface Props {
  status: StatusInfo | null;
}

const TABS = [
  { key: "mirrors", label: "镜像源" },
  { key: "env", label: "环境变量" },
  { key: "proxy", label: "代理" },
  { key: "git", label: "Git" },
  { key: "dns", label: "DNS" },
] as const;

type TabKey = (typeof TABS)[number]["key"];

export default function Dashboard({ status }: Props) {
  const [activeTab, setActiveTab] = useState<TabKey>("mirrors");
  const [detectedMirrors, setDetectedMirrors] = useState<Record<string, string>>({});
  const [installedTools, setInstalledTools] = useState<Set<string>>(new Set());
  const [mirrorGroups, setMirrorGroups] = useState<MirrorGroup[]>([]);
  const [ipLocation, setIpLocation] = useState<IpLocation | null>(null);
  const [ipLoading, setIpLoading] = useState(false);

  const loadExtras = useCallback(async () => {
    try {
      const [detected, installed, groups] = await Promise.all([detectMirrors(), getInstalledTools(), listMirrors()]);
      setDetectedMirrors(detected);
      setInstalledTools(new Set(installed));
      setMirrorGroups(groups);
    } catch (e) {
      console.error("Failed to load dashboard extras:", e);
    }
  }, []);

  const loadIpLocation = useCallback(async () => {
    setIpLoading(true);
    try {
      const loc = await getIpLocation();
      setIpLocation(loc);
    } catch (e) {
      console.error("Failed to load IP location:", e);
    } finally {
      setIpLoading(false);
    }
  }, []);

  useEffect(() => {
    loadExtras();
    loadIpLocation();
  }, [loadExtras, loadIpLocation]);

  // 合并 profile mirrors 和 detected mirrors（必须在条件 return 之前，遵守 Hooks 规则）
  // 使用 mirrorGroups 的顺序（来自平台 JSON，稳定）作为规范排序
  const allTools = useMemo(
    () => {
      if (!status) return new Set<string>();
      const groupOrder = mirrorGroups.map(g => g.tool);
      const groupOrderSet = new Set(groupOrder);
      const extraTools = [...new Set([...Object.keys(status.mirrors), ...Object.keys(detectedMirrors)])]
        .filter(t => !groupOrderSet.has(t));
      return new Set([...groupOrder, ...extraTools]);
    },
    [status?.mirrors, detectedMirrors, mirrorGroups]
  );
  const installedMirrorTools = useMemo(
    () => [...allTools].filter(t => installedTools.has(t)),
    [allTools, installedTools]
  );

  if (!status) {
    return <div className="loading-state">加载中...</div>;
  }

  const isCn = status.current_profile === "cn";

  return (
    <div>
      {/* Profile Card */}
      <div className="glass-card dashboard-profile-card">
        <div className="dashboard-profile-left">
          <div className="profile-icon-wrap">
            <IconGlobe size={26} />
          </div>
          <div>
            <div className="dashboard-profile-name">{status.display_name}</div>
            <div className="dashboard-profile-sub">Profile: {status.current_profile}</div>
            {status.has_local_config && (
              <div className="dashboard-local-badge">
                <IconSettings size={11} />
                有项目级配置
              </div>
            )}
          </div>
        </div>
        <div className="dashboard-profile-stats">
          <StatChip label="镜像源" value={installedMirrorTools.length} />
          <span className="stat-chip-divider" />
          <StatChip label="环境变量" value={Object.keys(status.env).length} />
          <span className="stat-chip-divider" />
          <StatChip label="代理" value={status.proxy ? 1 : 0} />
          <span className="stat-chip-divider" />
          <StatChip label="DNS" value={status.dns?.servers.length ?? 0} />
          <span className="stat-chip-divider" />
          <ProfileIpInline ipLocation={ipLocation} ipLoading={ipLoading} isCn={isCn} onRefresh={loadIpLocation} />
        </div>
      </div>

      {/* Tab Bar */}
      <div className="glass-tabs" role="tablist">
        {TABS.map((tab) => (
          <button
            key={tab.key}
            className={`glass-tab ${activeTab === tab.key ? "active" : ""}`}
            onClick={() => setActiveTab(tab.key)}
            role="tab"
            aria-selected={activeTab === tab.key}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {/* Tab Content */}
      <div className="glass-card">
        {activeTab === "mirrors" && (
          <MirrorsTab
            profileMirrors={status.mirrors}
            detectedMirrors={detectedMirrors}
            installedTools={installedTools}
            mirrorGroups={mirrorGroups}
          />
        )}
        {activeTab === "env" && <EnvTab status={status} />}
        {activeTab === "proxy" && <ProxyTab status={status} />}
        {activeTab === "git" && <GitTab status={status} />}
        {activeTab === "dns" && <DnsTab status={status} />}
      </div>
    </div>
  );
}

function MirrorsTab({
  profileMirrors,
  detectedMirrors,
  installedTools,
  mirrorGroups,
}: {
  profileMirrors: Record<string, string>;
  detectedMirrors: Record<string, string>;
  installedTools: Set<string>;
  mirrorGroups: MirrorGroup[];
}) {
  // 使用 mirrorGroups 的顺序（来自平台 JSON，稳定）作为规范排序
  const groupOrder = mirrorGroups.map(g => g.tool);
  const groupOrderSet = new Set(groupOrder);
  const extraTools = [...new Set([...Object.keys(profileMirrors), ...Object.keys(detectedMirrors)])]
    .filter(t => !groupOrderSet.has(t));
  const tools = [...groupOrder, ...extraTools].filter(t => installedTools.has(t));

  if (tools.length === 0) {
    return <Empty text="无已安装的工具" />;
  }

  // tool → mirror name → display_name / url 查找表
  // tool → url → name 反查表（处理 profile 中存了 URL 而非 name 的情况）
  const displayNameMap: Record<string, Record<string, string>> = {};
  const urlMap: Record<string, Record<string, string>> = {};
  const urlToNameMap: Record<string, Record<string, string>> = {};
  for (const g of mirrorGroups) {
    displayNameMap[g.tool] = {};
    urlMap[g.tool] = {};
    urlToNameMap[g.tool] = {};
    for (const m of g.mirrors) {
      displayNameMap[g.tool][m.name] = m.display_name;
      urlMap[g.tool][m.name] = m.url;
      // 用 trimEnd('/') 后的 URL 做 key，兼容有无尾部斜杠
      urlToNameMap[g.tool][m.url.replace(/\/+$/, "")] = m.name;
    }
  }

  return (
    <table className="dashboard-table">
      <thead>
        <tr>
          <th>工具</th>
          <th>当前镜像源</th>
          <th>URL</th>
        </tr>
      </thead>
      <tbody>
        {tools.map((tool) => {
          const profileVal = profileMirrors[tool];
          const detectedVal = detectedMirrors[tool];
          let active = profileVal || detectedVal;
          // 如果 active 是 URL，反查为镜像名
          if (active && !displayNameMap[tool]?.[active]) {
            const norm = active.replace(/\/+$/, "");
            let resolved: string | undefined = urlToNameMap[tool]?.[norm];

            // /simple 后缀匹配 (pip)
            if (!resolved && norm.startsWith("http")) {
              const urlEntries = Object.entries(urlToNameMap[tool] || {});
              // norm 可能带 /simple，尝试去掉后匹配
              const normWithoutSimple = norm.replace(/\/simple$/, "");
              resolved = urlEntries.find(([url]) =>
                `${url}/simple` === norm || url === normWithoutSimple
              )?.[1];
            }

            // 双向前缀匹配，normalized.length > 8 防止误匹配过短 URL
            if (!resolved && norm.startsWith("http") && norm.length > 8) {
              const urlEntries = Object.entries(urlToNameMap[tool] || {});
              resolved = urlEntries.find(([url]) =>
                norm.startsWith(url) || url.startsWith(norm)
              )?.[1];
            }

            // 域名级别匹配：仅在 URL 无明显路径时尝试
            if (!resolved && norm.startsWith("http")) {
              const afterProto = norm.replace(/^https?:\/\//, "");
              const rawPath = afterProto.includes("/")
                ? afterProto.slice(afterProto.indexOf("/") + 1)
                : "";
              if (!rawPath) {
                const rawHost = afterProto.split("/")[0];
                const urlEntries = Object.entries(urlToNameMap[tool] || {});
                resolved = urlEntries.find(([url]) => {
                  const entryHost = url.replace(/^https?:\/\//, "").split("/")[0];
                  return entryHost === rawHost;
                })?.[1];
              }
            }

            if (resolved) active = resolved;
          }
          // 优先使用 display_name
          const displayName = active ? (displayNameMap[tool]?.[active] || active) : null;
          const mirrorUrl = active ? (urlMap[tool]?.[active] || null) : null;
          return (
            <tr key={tool}>
              <td className="dashboard-table-tool">
                {getToolIcon(tool, 14)}
                {tool}
              </td>
              <td className="dashboard-table-value">
                {displayName ? (
                  <span className="mirror-active-tag">
                    <IconCheck size={10} />
                    {displayName}
                  </span>
                ) : (
                  <span className="dashboard-table-empty">未配置</span>
                )}
              </td>
              <td className="dashboard-table-url">
                {mirrorUrl ? mirrorUrl : <span className="dashboard-table-empty">—</span>}
              </td>
            </tr>
          );
        })}
      </tbody>
    </table>
  );
}

function EnvTab({ status }: { status: StatusInfo }) {
  if (Object.keys(status.env).length === 0) {
    return <Empty text="无自定义环境变量" />;
  }
  return (
    <div className="table-grid">
      {Object.entries(status.env).map(([key, value]) => (
        <Row key={key} label={key} value={value} />
      ))}
    </div>
  );
}

function ProxyTab({ status }: { status: StatusInfo }) {
  if (!status.proxy) {
    return <Empty text="未配置代理" />;
  }
  return (
    <div className="table-grid">
      <Row label="地址" value={status.proxy.address} />
      <Row label="类型" value={status.proxy.proxy_type} />
    </div>
  );
}

function GitTab({ status }: { status: StatusInfo }) {
  if (!status.git) {
    return <Empty text="无自定义 Git 配置" />;
  }
  return (
    <div className="table-grid">
      {status.git.github_mirror && <Row label="GitHub 镜像" value={status.git.github_mirror} />}
      {status.git.proxy && <Row label="Git 代理" value={status.git.proxy} />}
    </div>
  );
}

function DnsTab({ status }: { status: StatusInfo }) {
  if (!status.dns) {
    return <Empty text="未配置 DNS" />;
  }
  return (
    <div className="table-grid">
      <Row label="服务器" value={status.dns.servers.join(", ")} />
    </div>
  );
}

const Row = React.memo(function Row({ label, value, icon }: { label: string; value: string; icon?: React.ReactNode }) {
  return (
    <div className="glass-row">
      <span className="glass-row-label">
        {icon}
        {label}
      </span>
      <span className="glass-row-value">{value}</span>
    </div>
  );
});

const Empty = React.memo(function Empty({ text }: { text: string }) {
  return <div className="empty-state">{text}</div>;
});

const StatChip = React.memo(function StatChip({ label, value }: { label: string; value: number }) {
  return (
    <div className="stat-chip">
      <span className="stat-chip-value">{value}</span>
      <span className="stat-chip-label">{label}</span>
    </div>
  );
});

function ProfileIpInline({ ipLocation, ipLoading, isCn, onRefresh }: {
  ipLocation: IpLocation | null;
  ipLoading: boolean;
  isCn: boolean;
  onRefresh: () => void;
}) {
  const mismatch = ipLocation ? (isCn && !ipLocation.is_cn) || (!isCn && ipLocation.is_cn) : false;

  return (
    <div className={`profile-ip-inline ${mismatch ? "ip-mismatch" : ""}`}>
      {ipLoading ? (
        <span className="ip-location-loading">检测中...</span>
      ) : ipLocation ? (
        <>
          <span className="ip-location-icon">🌐</span>
          <span className="ip-location-addr">{ipLocation.ip}</span>
          <span className="ip-location-region">
            {[ipLocation.country, ipLocation.region, ipLocation.city].filter(Boolean).join(" · ")}
          </span>
          {mismatch && (
            <span className="ip-location-warn">⚠️ 不一致</span>
          )}
          <button className="glass-btn ip-location-refresh" onClick={onRefresh} title="重新检测">
            <IconRefresh size={12} />
          </button>
        </>
      ) : (
        <span className="ip-location-failed">检测失败</span>
      )}
    </div>
  );
}

