// SVG Icon 组件 — 使用各工具官方 logo

import React from "react";

const iconBaseStyle: React.CSSProperties = { display: "inline-block", verticalAlign: "middle" };
const toolIconStyle: React.CSSProperties = { ...iconBaseStyle, borderRadius: 3, objectFit: "contain" };

// ─── 工具/框架 Logo（使用 public/icons/ 下的官方 SVG）─────────────────

interface ToolIconProps {
  size?: number;
  src: string;
  alt: string;
}

const ToolIcon: React.FC<ToolIconProps> = ({ size = 20, src, alt }) => (
  <img
    src={src}
    alt={alt}
    width={size}
    height={size}
    style={toolIconStyle}
  />
);

export const IconPip: React.FC<{ size?: number }> = ({ size }) => <ToolIcon size={size} src="/icons/pip.svg" alt="pip" />;
export const IconNpm: React.FC<{ size?: number }> = ({ size }) => <ToolIcon size={size} src="/icons/npm.svg" alt="npm" />;
export const IconCargo: React.FC<{ size?: number }> = ({ size }) => <ToolIcon size={size} src="/icons/cargo.svg" alt="cargo" />;
export const IconGo: React.FC<{ size?: number }> = ({ size }) => <ToolIcon size={size} src="/icons/go.svg" alt="go" />;
export const IconDocker: React.FC<{ size?: number }> = ({ size }) => <ToolIcon size={size} src="/icons/docker.svg" alt="docker" />;
export const IconConda: React.FC<{ size?: number }> = ({ size }) => <ToolIcon size={size} src="/icons/conda.svg" alt="conda" />;
export const IconBrew: React.FC<{ size?: number }> = ({ size }) => <ToolIcon size={size} src="/icons/brew.svg" alt="brew" />;
export const IconApt: React.FC<{ size?: number }> = ({ size }) => <ToolIcon size={size} src="/icons/apt.svg" alt="apt" />;
export const IconChoco: React.FC<{ size?: number }> = ({ size }) => <ToolIcon size={size} src="/icons/choco.svg" alt="choco" />;
export const IconNuget: React.FC<{ size?: number }> = ({ size }) => <ToolIcon size={size} src="/icons/nuget.svg" alt="nuget" />;
export const IconMaven: React.FC<{ size?: number }> = ({ size }) => <ToolIcon size={size} src="/icons/maven.svg" alt="maven" />;
export const IconRuby: React.FC<{ size?: number }> = ({ size }) => <ToolIcon size={size} src="/icons/rubygems.svg" alt="rubygems" />;
export const IconComposer: React.FC<{ size?: number }> = ({ size }) => <ToolIcon size={size} src="/icons/composer.svg" alt="composer" />;
export const IconPub: React.FC<{ size?: number }> = ({ size }) => <ToolIcon size={size} src="/icons/pub.svg" alt="pub" />;
export const IconYarn: React.FC<{ size?: number }> = ({ size }) => <ToolIcon size={size} src="/icons/yarn.svg" alt="yarn" />;
export const IconPnpm: React.FC<{ size?: number }> = ({ size }) => <ToolIcon size={size} src="/icons/pnpm.svg" alt="pnpm" />;
export const IconBun: React.FC<{ size?: number }> = ({ size }) => <ToolIcon size={size} src="/icons/bun.svg" alt="bun" />;
export const IconDeno: React.FC<{ size?: number }> = ({ size }) => <ToolIcon size={size} src="/icons/deno.svg" alt="deno" />;
export const IconGradle: React.FC<{ size?: number }> = ({ size }) => <ToolIcon size={size} src="/icons/gradle.svg" alt="gradle" />;
export const IconCocoapods: React.FC<{ size?: number }> = ({ size }) => <ToolIcon size={size} src="/icons/cocoapods.svg" alt="cocoapods" />;
export const IconHuggingFace: React.FC<{ size?: number }> = ({ size }) => <ToolIcon size={size} src="/icons/huggingface.svg" alt="huggingface" />;
export const IconK8s: React.FC<{ size?: number }> = ({ size }) => <ToolIcon size={size} src="/icons/k8s.svg" alt="kubernetes" />;
export const IconGhcr: React.FC<{ size?: number }> = ({ size }) => <ToolIcon size={size} src="/icons/ghcr.svg" alt="ghcr" />;
export const IconQuay: React.FC<{ size?: number }> = ({ size }) => <ToolIcon size={size} src="/icons/quay.svg" alt="quay" />;
export const IconNodeJs: React.FC<{ size?: number }> = ({ size }) => <ToolIcon size={size} src="/icons/nodejs.svg" alt="nodejs" />;
export const IconPython: React.FC<{ size?: number }> = ({ size }) => <ToolIcon size={size} src="/icons/python.svg" alt="python" />;
export const IconRustup: React.FC<{ size?: number }> = ({ size }) => <ToolIcon size={size} src="/icons/rustup.svg" alt="rustup" />;
export const IconVSCode: React.FC<{ size?: number }> = ({ size }) => <ToolIcon size={size} src="/icons/vscode.svg" alt="vscode" />;
export const IconAndroid: React.FC<{ size?: number }> = ({ size }) => <ToolIcon size={size} src="/icons/android.svg" alt="android" />;
export const IconSwift: React.FC<{ size?: number }> = ({ size }) => <ToolIcon size={size} src="/icons/swift.svg" alt="swift" />;

// ─── 状态图标 ──────────────────────────────────────────────────────

export const IconCheck: React.FC<{ size?: number }> = ({ size = 18 }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" style={iconBaseStyle} aria-hidden="true">
    <circle cx="12" cy="12" r="10" fill="#064e3b" stroke="#6ee7b7" strokeWidth="1.5"/>
    <path d="M8 12.5l2.5 2.5 5.5-5.5" stroke="#6ee7b7" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
  </svg>
);

export const IconWarn: React.FC<{ size?: number }> = ({ size = 18 }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" style={iconBaseStyle} aria-hidden="true">
    <path d="M12 3L2 21h20L12 3z" fill="#713f12" stroke="#fcd34d" strokeWidth="1.5" strokeLinejoin="round"/>
    <line x1="12" y1="10" x2="12" y2="14" stroke="#fcd34d" strokeWidth="2" strokeLinecap="round"/>
    <circle cx="12" cy="17.5" r="1" fill="#fcd34d"/>
  </svg>
);

export const IconError: React.FC<{ size?: number }> = ({ size = 18 }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" style={iconBaseStyle} aria-hidden="true">
    <circle cx="12" cy="12" r="10" fill="#7f1d1d" stroke="#f87171" strokeWidth="1.5"/>
    <line x1="9" y1="9" x2="15" y2="15" stroke="#f87171" strokeWidth="2" strokeLinecap="round"/>
    <line x1="15" y1="9" x2="9" y2="15" stroke="#f87171" strokeWidth="2" strokeLinecap="round"/>
  </svg>
);

// ─── 功能图标 ──────────────────────────────────────────────────────

export const IconRefresh: React.FC<{ size?: number }> = ({ size = 16 }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" style={iconBaseStyle} aria-hidden="true">
    <path d="M21 2v6h-6"/>
    <path d="M3 12a9 9 0 0 1 15-6.7L21 8"/>
    <path d="M3 22v-6h6"/>
    <path d="M21 12a9 9 0 0 1-15 6.7L3 16"/>
  </svg>
);

export const IconDashboard: React.FC<{ size?: number }> = ({ size = 18 }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" style={iconBaseStyle} aria-hidden="true">
    <rect x="3" y="3" width="7" height="9" rx="1"/>
    <rect x="14" y="3" width="7" height="5" rx="1"/>
    <rect x="14" y="12" width="7" height="9" rx="1"/>
    <rect x="3" y="16" width="7" height="5" rx="1"/>
  </svg>
);

export const IconMirror: React.FC<{ size?: number }> = ({ size = 18 }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" style={iconBaseStyle} aria-hidden="true">
    <path d="M12 3v18"/>
    <path d="M5 6l-3 3 3 3"/>
    <path d="M19 6l3 3-3 3"/>
    <rect x="9" y="14" width="6" height="7" rx="1"/>
  </svg>
);

export const IconDoctor: React.FC<{ size?: number }> = ({ size = 18 }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" style={iconBaseStyle} aria-hidden="true">
    <path d="M22 12h-4l-3 9L9 3l-3 9H2"/>
  </svg>
);

export const IconSun: React.FC<{ size?: number }> = ({ size = 18 }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" style={iconBaseStyle} aria-hidden="true">
    <circle cx="12" cy="12" r="5"/>
    <line x1="12" y1="1" x2="12" y2="3"/>
    <line x1="12" y1="21" x2="12" y2="23"/>
    <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/>
    <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/>
    <line x1="1" y1="12" x2="3" y2="12"/>
    <line x1="21" y1="12" x2="23" y2="12"/>
    <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/>
    <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
  </svg>
);

export const IconMoon: React.FC<{ size?: number }> = ({ size = 18 }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" style={iconBaseStyle} aria-hidden="true">
    <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
  </svg>
);

export const IconSettings: React.FC<{ size?: number }> = ({ size = 18 }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" style={iconBaseStyle} aria-hidden="true">
    <circle cx="12" cy="12" r="3"/>
    <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/>
  </svg>
);

export const IconGlobe: React.FC<{ size?: number }> = ({ size = 20 }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" style={iconBaseStyle} aria-hidden="true">
    <circle cx="12" cy="12" r="10"/>
    <line x1="2" y1="12" x2="22" y2="12"/>
    <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>
  </svg>
);

export const IconChina: React.FC<{ size?: number }> = ({ size = 20 }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" style={iconBaseStyle} aria-hidden="true">
    <rect x="2" y="4" width="20" height="16" rx="2" fill="#DE2910"/>
    <polygon points="6,7 7.2,10.2 10.6,10.2 7.8,12.2 8.8,15.4 6,13.6 3.2,15.4 4.2,12.2 1.4,10.2 4.8,10.2" fill="#FFDE00"/>
    <polygon points="12,6 12.5,7.5 14,7.5 12.8,8.5 13.2,10 12,9 10.8,10 11.2,8.5 10,7.5 11.5,7.5" fill="#FFDE00"/>
    <polygon points="15,7 15.4,8.2 16.6,8.2 15.6,9 15.9,10.2 15,9.5 14.1,10.2 14.4,9 13.4,8.2 14.6,8.2" fill="#FFDE00"/>
    <polygon points="15,10 15.4,11.2 16.6,11.2 15.6,12 15.9,13.2 15,12.5 14.1,13.2 14.4,12 13.4,11.2 14.6,11.2" fill="#FFDE00"/>
    <polygon points="12,11 12.4,12.2 13.6,12.2 12.6,13 12.9,14.2 12,13.5 11.1,14.2 11.4,13 10.4,12.2 11.6,12.2" fill="#FFDE00"/>
  </svg>
);

export const IconGit: React.FC<{ size?: number }> = ({ size }) => <ToolIcon size={size} src="/icons/git.svg" alt="git" />;

// ─── 窗口控制图标 ─────────────────────────────────────────────────

export const IconMinimize: React.FC<{ size?: number }> = ({ size = 12 }) => (
  <svg width={size} height={size} viewBox="0 0 12 12" fill="none" style={iconBaseStyle} aria-hidden="true">
    <line x1="2" y1="6" x2="10" y2="6" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round"/>
  </svg>
);

export const IconMaximize: React.FC<{ size?: number }> = ({ size = 12 }) => (
  <svg width={size} height={size} viewBox="0 0 12 12" fill="none" style={iconBaseStyle} aria-hidden="true">
    <rect x="2" y="2" width="8" height="8" rx="1.5" stroke="currentColor" strokeWidth="1.2"/>
  </svg>
);

export const IconRestore: React.FC<{ size?: number }> = ({ size = 12 }) => (
  <svg width={size} height={size} viewBox="0 0 12 12" fill="none" style={iconBaseStyle} aria-hidden="true">
    <rect x="3.5" y="1" width="6.5" height="6.5" rx="1" stroke="currentColor" strokeWidth="1"/>
    <rect x="1.5" y="4" width="6.5" height="6.5" rx="1" stroke="currentColor" strokeWidth="1" fill="var(--bg-deep, #080810)"/>
  </svg>
);

export const IconClose: React.FC<{ size?: number }> = ({ size = 12 }) => (
  <svg width={size} height={size} viewBox="0 0 12 12" fill="none" style={iconBaseStyle} aria-hidden="true">
    <line x1="2.5" y1="2.5" x2="9.5" y2="9.5" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round"/>
    <line x1="9.5" y1="2.5" x2="2.5" y2="9.5" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round"/>
  </svg>
);

// ─── 工具图标映射 ─────────────────────────────────────────────────

export const toolIcons: Record<string, React.FC<{ size?: number }>> = {
  pip: IconPip,
  npm: IconNpm,
  yarn: IconYarn,
  pnpm: IconPnpm,
  bun: IconBun,
  deno: IconDeno,
  cargo: IconCargo,
  go: IconGo,
  docker: IconDocker,
  "k8s-gcr": IconK8s,
  "k8s-registry": IconK8s,
  ghcr: IconGhcr,
  quay: IconQuay,
  conda: IconConda,
  brew: IconBrew,
  apt: IconApt,
  choco: IconChoco,
  nuget: IconNuget,
  maven: IconMaven,
  gradle: IconGradle,
  rubygems: IconRuby,
  composer: IconComposer,
  pub: IconPub,
  cocoapods: IconCocoapods,
  huggingface: IconHuggingFace,
  nodejs: IconNodeJs,
  python: IconPython,
  rustup: IconRustup,
  vscode: IconVSCode,
  "android-maven": IconAndroid,
  "android-gradle": IconAndroid,
  swift: IconSwift,
};

export const getToolIcon = (tool: string, size = 20) => {
  const Icon = toolIcons[tool];
  return Icon ? <Icon size={size} /> : null;
};
