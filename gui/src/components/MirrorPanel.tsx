import { useState, useEffect, useRef, useCallback } from "react";
import { listMirrors, setMirror, resetMirror, testMirrorsStreaming, getPlatformInfo, getStatus, getInstalledTools, detectMirrors } from "../api";
import type { MirrorGroup, MirrorTestResultInfo, MirrorLatencyInfo, MirrorTestEventPayload } from "../types";
import { errorMessage } from "../utils";
import { useToast } from "../hooks/useToast";
import { getToolIcon, IconRefresh, IconCheck } from "../icons";
import { listen } from "@tauri-apps/api/event";

function latencyClass(ms: number): string {
  if (ms < 0) return "latency-timeout";
  if (ms < 100) return "latency-fast";
  if (ms < 300) return "latency-medium";
  return "latency-slow";
}

interface Props {
  onMirrorChange?: () => void;
}

export default function MirrorPanel({ onMirrorChange }: Props) {
  const [groups, setGroups] = useState<MirrorGroup[]>([]);
  const [allGroups, setAllGroups] = useState<MirrorGroup[]>([]);
  const [selectedTool, setSelectedTool] = useState<string>("");
  const { message, showToast } = useToast();
  const [loading, setLoading] = useState(false);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [testing, setTesting] = useState(false);
  const [testResults, setTestResults] = useState<MirrorTestResultInfo[]>([]);
  const [platformName, setPlatformName] = useState<string>("");
  const [currentMirrors, setCurrentMirrors] = useState<Record<string, string>>({});
  const [detectedMirrors, setDetectedMirrors] = useState<Record<string, string>>({});
  const [installedTools, setInstalledTools] = useState<Set<string>>(new Set());
  // C3 fix: 保存测速监听器的 unlisten 引用，组件卸载时清理
  const unlistenRefs = useRef<Array<() => void>>([]);
  // C4 fix: 请求 ID 防止竞态
  const loadRequestId = useRef(0);
  // Timeout ref for mirror test safety
  const testTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    return () => {
      // C3 fix: 组件卸载时清理所有事件监听器
      unlistenRefs.current.forEach(fn => fn());
      unlistenRefs.current = [];
      if (testTimeoutRef.current) clearTimeout(testTimeoutRef.current);
    };
  }, []);

  const loadCurrentMirrors = useCallback(async () => {
    try {
      const status = await getStatus();
      setCurrentMirrors(status.mirrors);
    } catch (e) {
      console.error("Failed to load current mirrors:", e);
    }
  }, []);

  const loadDetectedMirrors = useCallback(async () => {
    try {
      const detected = await detectMirrors();
      setDetectedMirrors(detected);
    } catch (e) {
      console.error("Failed to detect mirrors:", e);
    }
  }, []);

  // C4 fix: 用 useCallback + selectedTool 依赖，添加请求 ID 防竞态
  const load = useCallback(async () => {
    const requestId = ++loadRequestId.current;
    setLoading(true);
    setLoadError(null);
    try {
      const [data, platform, installed] = await Promise.all([
        listMirrors(selectedTool || undefined),
        getPlatformInfo(),
        getInstalledTools(),
      ]);
      // C4 fix: 如果已经有更新的请求，丢弃本次结果
      if (requestId !== loadRequestId.current) return;
      setGroups(data);
      setPlatformName(platform.name);
      setInstalledTools(new Set(installed));
      if (!selectedTool) setAllGroups(data);
    } catch (e) {
      if (requestId !== loadRequestId.current) return;
      const msg = errorMessage(e);
      console.error("Failed to load mirrors:", msg);
      setLoadError(msg);
    } finally {
      if (requestId === loadRequestId.current) setLoading(false);
    }
  }, [selectedTool]);

  useEffect(() => {
    load();
    loadCurrentMirrors();
    loadDetectedMirrors();
    setTestResults([]);
  }, [selectedTool, load, loadCurrentMirrors, loadDetectedMirrors]);

  // 只展示已安装的工具
  const installedGroups = allGroups.filter(g => installedTools.has(g.tool));
  const filteredGroups = groups.filter(g => installedTools.has(g.tool));
  const toolList = [{ key: "", label: "全部" }, ...installedGroups.map(g => ({ key: g.tool, label: g.tool }))];

  const handleSet = async (tool: string, source: string) => {
    if (busy) return;
    setBusy(true);
    try {
      const msg = await setMirror(tool, source);
      showToast(msg, "ok");
      onMirrorChange?.();
      await Promise.all([loadCurrentMirrors(), loadDetectedMirrors()]);
    } catch (e) {
      showToast(errorMessage(e), "error");
    } finally {
      setBusy(false);
    }
  };

  const handleReset = async (tool: string) => {
    if (busy) return;
    setBusy(true);
    try {
      const msg = await resetMirror(tool);
      showToast(msg, "ok");
      onMirrorChange?.();
      await Promise.all([loadCurrentMirrors(), loadDetectedMirrors()]);
    } catch (e) {
      showToast(errorMessage(e), "error");
    } finally {
      setBusy(false);
    }
  };

  const handleTest = async () => {
    if (testing) return;
    setTesting(true);
    setTestResults([]);

    // Timeout protection: auto-stop if mirror-test-done never fires
    if (testTimeoutRef.current) clearTimeout(testTimeoutRef.current);
    testTimeoutRef.current = setTimeout(() => {
      setTesting(false);
      showToast("测速超时，请重试", "error");
      testTimeoutRef.current = null;
    }, 60_000);

    const unlistenResult = await listen<MirrorTestEventPayload>(
      "mirror-test-result",
      (event) => {
        const { tool, name, url, latency_ms } = event.payload;
        setTestResults((prev) => {
          const existing = prev.find((r) => r.tool === tool);
          const item: MirrorLatencyInfo = { name, url, latency_ms };
          if (existing) {
            return prev.map((r) =>
              r.tool === tool ? { ...r, results: [...r.results, item] } : r
            );
          }
          return [...prev, { tool, results: [item], recommended: null }];
        });
      }
    );

    const unlistenDone = await listen("mirror-test-done", () => {
      if (testTimeoutRef.current) {
        clearTimeout(testTimeoutRef.current);
        testTimeoutRef.current = null;
      }
      setTestResults((prev) =>
        prev.map((group) => {
          const sorted = [...group.results].sort(
            (a, b) => (a.latency_ms ?? Infinity) - (b.latency_ms ?? Infinity)
          );
          const recommended = sorted[0]?.latency_ms != null ? sorted[0].name : null;
          return { ...group, results: sorted, recommended };
        })
      );
      // C3 fix: 清理引用
      unlistenRefs.current = unlistenRefs.current.filter(fn => fn !== unlistenResult && fn !== unlistenDone);
      unlistenResult();
      unlistenDone();
      setTesting(false);
    });

    // C3 fix: 保存 unlisten 引用，组件卸载时调用
    unlistenRefs.current.push(unlistenResult, unlistenDone);

    try {
      await testMirrorsStreaming(selectedTool || undefined);
    } catch (e) {
      showToast(errorMessage(e), "error");
      if (testTimeoutRef.current) {
        clearTimeout(testTimeoutRef.current);
        testTimeoutRef.current = null;
      }
      setTesting(false);
      unlistenResult();
      unlistenDone();
      unlistenRefs.current = unlistenRefs.current.filter(fn => fn !== unlistenResult && fn !== unlistenDone);
    }
  };

  const getLatency = (tool: string, name: string): number | -1 | null => {
    const group = testResults.find((r) => r.tool === tool);
    if (!group) return null;
    const item = group.results.find((r) => r.name === name);
    if (!item) return null;
    return item.latency_ms ?? -1;
  };

  const getRecommended = (tool: string): string | null => {
    const group = testResults.find((r) => r.tool === tool);
    return group?.recommended ?? null;
  };

  // 有推荐结果（测速完成后显示）
  const hasRecommendations = !testing && testResults.some(r => r.recommended);

  const handleSetAllOptimal = async () => {
    if (busy) return;
    const toSet = testResults
      .filter(r => r.recommended)
      .filter(r => {
        const active = currentMirrors[r.tool] || detectedMirrors[r.tool];
        return active !== r.recommended;
      });
    if (toSet.length === 0) {
      showToast("所有工具已使用最优镜像源", "ok");
      return;
    }
    setBusy(true);
    const success: string[] = [];
    const failed: string[] = [];
    for (const r of toSet) {
      try {
        await setMirror(r.tool, r.recommended!);
        success.push(r.tool);
      } catch {
        failed.push(r.tool);
      }
    }
    onMirrorChange?.();
    await Promise.all([loadCurrentMirrors(), loadDetectedMirrors()]);
    setBusy(false);
    if (failed.length === 0) {
      showToast(`已设置 ${success.length} 个工具的最优镜像源`, "ok");
    } else {
      showToast(`成功 ${success.length}，失败 ${failed.length}: ${failed.join(", ")}`, "error");
    }
  };

  return (
    <div aria-label="镜像源管理">
      <div className="page-header">
        <h2 className="page-title">
          镜像源管理
          {platformName && <span className="platform-badge">{platformName}</span>}
        </h2>
        <div className="page-header-actions">
          {hasRecommendations && (
            <button className="glass-btn glass-btn-success" onClick={handleSetAllOptimal} disabled={busy}>
              ⚡ 一键设置最优
            </button>
          )}
          <button className="glass-btn glass-btn-accent" onClick={handleTest} disabled={testing}>
            {testing ? "测速中..." : "⚡ 开始测速"}
          </button>
        </div>
      </div>

      {message && (
        <div className={`toast ${message.type === "error" ? "toast-error" : "toast-ok"}`} role="status">
          {message.text}
        </div>
      )}

      <div className="glass-pill-group">
        {toolList.map((t) => (
          <button
            key={t.key || "all"}
            className={`glass-pill ${selectedTool === t.key ? "active" : ""}`}
            onClick={() => setSelectedTool(t.key)}
            disabled={loading}
          >
            {t.key && getToolIcon(t.key, 14)}
            {t.label}
          </button>
        ))}
      </div>

      {loading && groups.length === 0 ? (
        <div className="loading-state">加载中...</div>
      ) : loadError ? (
        <div className="empty-state">
          <span style={{ color: "var(--status-error)" }}>加载失败: {loadError}</span>
          <button className="glass-btn glass-btn-accent" onClick={load} style={{ marginTop: 8, fontSize: 12 }}>
            重试
          </button>
        </div>
      ) : filteredGroups.length === 0 ? (
        <div className="empty-state">未检测到已安装的工具</div>
      ) : (
        filteredGroups.map((group) => {
          const recommended = getRecommended(group.tool);
          const activeMirror = currentMirrors[group.tool] || detectedMirrors[group.tool];
          // 查找当前镜像源的 display_name
          const activeMirrorEntry = activeMirror
            ? group.mirrors.find(m => m.name === activeMirror)
            : null;
          const activeDisplayName = activeMirrorEntry?.display_name || activeMirror;
          // "官方" 时显示具体 URL
          const officialUrl = activeDisplayName === "官方"
            ? activeMirrorEntry?.url
            : null;
          return (
            <div key={group.tool} className="glass-card">
              {/* 卡片头部 */}
              <div className="mirror-card-header">
                <div className="mirror-card-title">
                  {getToolIcon(group.tool, 20)}
                  <span className="tool-label">{group.display_name || group.tool}</span>
                  {activeMirror && (
                    <span className="mirror-active-tag">
                      <IconCheck size={11} />
                      {officialUrl ? (
                        <>{officialUrl}</>
                      ) : (
                        activeDisplayName
                      )}
                    </span>
                  )}
                </div>
                <button
                  className="glass-btn glass-btn-danger mirror-reset-btn"
                  onClick={() => handleReset(group.tool)}
                  disabled={busy}
                >
                  <IconRefresh size={11} />
                  重置
                </button>
              </div>

              {/* 配置文件路径 */}
              {group.config_path && (
                <div className="mirror-config-path">
                  {group.config_type === "file" ? "📄" : group.config_type === "env" ? "🔧" : "📋"} {group.config_path}
                </div>
              )}

              {/* 内部表格 */}
              <table className="mirror-table">
                <thead>
                  <tr>
                    <th className="col-head-name">镜像</th>
                    <th className="col-head-status">状态</th>
                    <th>URL</th>
                    <th className="col-head-latency">延迟</th>
                    <th className="col-head-action">操作</th>
                  </tr>
                </thead>
                <tbody>
                  {group.mirrors.map((m) => {
                    const latency = getLatency(group.tool, m.name);
                    const isRecommended = recommended === m.name;
                    const isActive = activeMirror === m.name;
                    return (
                      <tr
                        key={m.name}
                        className={`${isActive ? "row-active" : ""} ${isRecommended ? "row-recommended" : ""}`}
                      >
                        <td className="col-name">
                          <span className="mirror-name">{m.display_name}</span>
                        </td>
                        <td className="col-status">
                          {isActive && <span className="mirror-badge mirror-badge-active">当前</span>}
                          {isRecommended && !isActive && <span className="mirror-badge">推荐</span>}
                        </td>
                        <td className="col-url" title={m.url}>{m.url}</td>
                        <td className="col-latency">
                          {testing && latency === null ? (
                            <span className="latency-tag latency-testing"><span className="spin-icon" /></span>
                          ) : latency !== null ? (
                            <span className={`latency-tag ${latency === -1 ? "latency-timeout" : latencyClass(latency)}`}>
                              {latency === -1 ? "超时" : `${latency}ms`}
                            </span>
                          ) : null}
                        </td>
                        <td className="col-action">
                          <button
                            className={`glass-btn ${isActive ? "glass-btn-success" : "glass-btn-accent"}`}
                            onClick={() => handleSet(group.tool, m.name)}
                            disabled={busy || isActive}
                          >
                            {isActive ? "✓ 使用中" : "使用"}
                          </button>
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          );
        })
      )}
    </div>
  );
}
