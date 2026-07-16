import { useState, useEffect, useCallback } from "react";
import { getStatus, switchProfile } from "./api";
import type { StatusInfo } from "./types";
import { errorMessage } from "./utils";
import { useToast } from "./hooks/useToast";
import Dashboard from "./components/Dashboard";
import MirrorPanel from "./components/MirrorPanel";
import DoctorPanel from "./components/DoctorPanel";
import SettingsPanel from "./components/SettingsPanel";
import ConfirmModal from "./components/ConfirmModal";
import TitleBar from "./components/TitleBar";
import { IconDashboard, IconMirror, IconDoctor, IconSettings, IconSun, IconMoon } from "./icons";
import "./glass.css";

type Tab = "dashboard" | "mirrors" | "doctor" | "settings";
type Theme = "light" | "dark" | "system";

function getSystemTheme(): "light" | "dark" {
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function applyTheme(theme: Theme) {
  const root = document.documentElement;
  if (theme === "system") {
    root.removeAttribute("data-theme");
  } else {
    root.setAttribute("data-theme", theme);
  }
}

function App() {
  const [tab, setTab] = useState<Tab>("dashboard");
  const [status, setStatus] = useState<StatusInfo | null>(null);
  const [loading, setLoading] = useState(false);
  const [statusError, setStatusError] = useState<string | null>(null);
  const { message: switchToast, showToast } = useToast();
  const [pendingSwitch, setPendingSwitch] = useState<"cn" | "global" | null>(null);
  const [theme, setTheme] = useState<Theme>(() => {
    const saved = localStorage.getItem("nz-switch-theme");
    if (saved === "light" || saved === "dark" || saved === "system") return saved;
    return "system";
  });

  // 应用主题
  useEffect(() => {
    applyTheme(theme);
    localStorage.setItem("nz-switch-theme", theme);
  }, [theme]);

  // 监听系统主题变化（当 theme=system 时自动跟随）
  useEffect(() => {
    if (theme !== "system") return;
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = () => applyTheme("system");
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  }, [theme]);

  const cycleTheme = () => {
    setTheme(prev => {
      if (prev === "system") return getSystemTheme() === "dark" ? "light" : "dark";
      if (prev === "dark") return "light";
      return "system";
    });
  };

  const themeIcon = theme === "system"
    ? (getSystemTheme() === "dark" ? <IconMoon size={14} /> : <IconSun size={14} />)
    : theme === "dark" ? <IconMoon size={14} /> : <IconSun size={14} />;

  const themeLabel = theme === "system" ? "跟随系统" : theme === "dark" ? "深色" : "浅色";

  const loadStatus = useCallback(async () => {
    setStatusError(null);
    try {
      const s = await getStatus();
      setStatus(s);
    } catch (e) {
      console.error("Failed to load status:", errorMessage(e));
      setStatusError(errorMessage(e));
    }
  }, []);

  useEffect(() => {
    loadStatus();
  }, [loadStatus, tab]);

  const handleSwitch = async (name: "cn" | "global") => {
    setLoading(true);
    try {
      const msg = await switchProfile(name);
      showToast(msg, "ok");
      await loadStatus();
    } catch (e) {
      showToast(errorMessage(e), "error");
    } finally {
      setLoading(false);
    }
  };

  const isCn = status?.current_profile === "cn";

  return (
    <div className="app">
      <TitleBar />

      {/* Top Nav */}
      <nav className="top-nav" aria-label="主导航">
        <div className="top-nav-left">
          <button
            className={`profile-btn ${isCn ? "is-cn" : ""}`}
            onClick={() => setPendingSwitch(isCn ? "global" : "cn")}
            disabled={loading}
          >
            <span>{loading ? "切换中..." : isCn ? "中国内地" : "海外"}</span>
          </button>
          {switchToast && (
            <div className={`nav-toast ${switchToast.type === "error" ? "nav-toast-error" : "nav-toast-warn"}`} title={switchToast.text}>
              ⚠️ {switchToast.text.length > 40 ? switchToast.text.slice(0, 40) + "..." : switchToast.text}
            </div>
          )}
        </div>
        <div className="top-nav-tabs">
          <button className={`nav-tab ${tab === "dashboard" ? "active" : ""}`} onClick={() => setTab("dashboard")} aria-current={tab === "dashboard" ? "page" : undefined}>
            <IconDashboard size={14} />
            状态总览
          </button>
          <button className={`nav-tab ${tab === "mirrors" ? "active" : ""}`} onClick={() => setTab("mirrors")} aria-current={tab === "mirrors" ? "page" : undefined}>
            <IconMirror size={14} />
            镜像源
          </button>
          <button className={`nav-tab ${tab === "doctor" ? "active" : ""}`} onClick={() => setTab("doctor")} aria-current={tab === "doctor" ? "page" : undefined}>
            <IconDoctor size={14} />
            诊断
          </button>
          <button className={`nav-tab ${tab === "settings" ? "active" : ""}`} onClick={() => setTab("settings")} aria-current={tab === "settings" ? "page" : undefined}>
            <IconSettings size={14} />
            设置
          </button>
          <span className="nav-divider" />
          <button className="theme-toggle-btn" onClick={cycleTheme} title={`当前: ${themeLabel}，点击切换`}>
            {themeIcon}
            <span>{themeLabel}</span>
          </button>
        </div>
      </nav>

      {/* Main Content */}
      <main className="main-content">
        {statusError && !status ? (
          <div className="loading-state">
            <span style={{ color: "var(--status-error)" }}>加载失败: {statusError}</span>
            <button className="glass-btn glass-btn-accent" onClick={loadStatus} style={{ marginTop: 12, fontSize: 13 }}>
              重试
            </button>
          </div>
        ) : (
          <>
            {tab === "dashboard" && <Dashboard status={status} onRefresh={loadStatus} />}
            {tab === "mirrors" && <MirrorPanel onMirrorChange={loadStatus} />}
            {tab === "doctor" && <DoctorPanel />}
            {tab === "settings" && <SettingsPanel />}
          </>
        )}
      </main>

      <ConfirmModal
        open={pendingSwitch !== null}
        title={pendingSwitch === "cn" ? "切换到中国内地模式" : "切换到海外模式"}
        message={
          pendingSwitch === "cn"
            ? "将自动配置国内镜像源（pip、npm、cargo 等）、DNS（223.5.5.5）和 GitHub 加速代理。这会修改你的 git 全局配置和各工具的配置文件。"
            : "将恢复为海外默认源（crates.io、registry.npmjs.org 等）、DNS（8.8.8.8）并清除 GitHub 代理配置。这会修改你的 git 全局配置和各工具的配置文件。"
        }
        confirmLabel={pendingSwitch === "cn" ? "切换到中国内地" : "切换到海外"}
        variant="warning"
        onConfirm={() => {
          if (pendingSwitch) handleSwitch(pendingSwitch);
          setPendingSwitch(null);
        }}
        onCancel={() => setPendingSwitch(null)}
      />
    </div>
  );
}

export default App;
