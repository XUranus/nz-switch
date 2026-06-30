import { useState, useEffect, useRef, useCallback, useMemo } from "react";
import { getRawConfig, saveRawConfig } from "../api";
import { IconRefresh, IconSettings } from "../icons";
import { errorMessage } from "../utils";

/** 将 TOML 字符串分词为带类型的 token 数组 */
interface Token {
  type: "key" | "str" | "num" | "bool" | "comment" | "section" | "punct" | "ws" | "date";
  value: string;
}

function tokenizeToml(raw: string): Token[] {
  const tokens: Token[] = [];
  const re = /(#.*)|(\[[\w.-]+\])|("(?:[^"\\]|\\.)*")|('(?:[^'\\]|\\.)*')|(\btrue\b|\bfalse\b)|(\d{4}-\d{2}-\d{2}(?:[T ]\d{2}:\d{2}:\d{2})?)|(-?\d+(?:\.\d+)?)|(\s+)|([\[\]=,{}.])|([^\s\[\]=,{}#"']+)/g;
  let m: RegExpExecArray | null;
  while ((m = re.exec(raw)) !== null) {
    const [full, comment, section, dstr, sstr, bool, date, num, ws, punct, bare] = m;
    if (comment) {
      tokens.push({ type: "comment", value: comment });
    } else if (section) {
      tokens.push({ type: "section", value: section });
    } else if (dstr || sstr) {
      tokens.push({ type: "str", value: dstr || sstr! });
    } else if (bool) {
      tokens.push({ type: "bool", value: bool });
    } else if (date) {
      tokens.push({ type: "date", value: date });
    } else if (num) {
      tokens.push({ type: "num", value: num });
    } else if (ws) {
      tokens.push({ type: "ws", value: ws });
    } else if (punct) {
      tokens.push({ type: "punct", value: punct });
    } else if (bare) {
      // bare key (left side of =)
      tokens.push({ type: "key", value: bare });
    } else {
      tokens.push({ type: "punct", value: full });
    }
  }
  return tokens;
}

/** 用 React span 渲染 TOML token */
function HighlightedToml({ toml }: { toml: string }) {
  const tokens = useMemo(() => tokenizeToml(toml), [toml]);
  return (
    <pre className="toml-highlight" aria-hidden="true">
      {tokens.map((t, i) => {
        switch (t.type) {
          case "key": return <span key={i} className="toml-key">{t.value}</span>;
          case "str": return <span key={i} className="toml-str">{t.value}</span>;
          case "num": return <span key={i} className="toml-num">{t.value}</span>;
          case "bool": return <span key={i} className="toml-bool">{t.value}</span>;
          case "comment": return <span key={i} className="toml-comment">{t.value}</span>;
          case "section": return <span key={i} className="toml-section">{t.value}</span>;
          case "date": return <span key={i} className="toml-date">{t.value}</span>;
          default: return <span key={i}>{t.value}</span>;
        }
      })}
      {"\n"}
    </pre>
  );
}

export default function SettingsPanel() {
  const [toml, setToml] = useState("");
  const [originalToml, setOriginalToml] = useState("");
  const [configPath, setConfigPath] = useState("");
  const [toast, setToast] = useState<{ type: "ok" | "error"; msg: string } | null>(null);
  const [saving, setSaving] = useState(false);
  const [dirty, setDirty] = useState(false);
  const [loadError, setLoadError] = useState<string | null>(null);
  const toastTimer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const preRef = useRef<HTMLPreElement>(null);

  const showToast = useCallback((type: "ok" | "error", msg: string) => {
    setToast({ type, msg });
    clearTimeout(toastTimer.current);
    toastTimer.current = setTimeout(() => setToast(null), 3000);
  }, []);

  const loadConfig = useCallback(async () => {
    setLoadError(null);
    try {
      const raw = await getRawConfig();
      setToml(raw.toml);
      setOriginalToml(raw.toml);
      setConfigPath(raw.path);
      setDirty(false);
    } catch (e) {
      setLoadError(errorMessage(e));
    }
  }, []);

  useEffect(() => {
    loadConfig();
    return () => clearTimeout(toastTimer.current);
  }, [loadConfig]);

  const handleChange = (value: string) => {
    setToml(value);
    setDirty(value !== originalToml);
  };

  const handleScroll = useCallback(() => {
    if (textareaRef.current && preRef.current) {
      preRef.current.scrollTop = textareaRef.current.scrollTop;
      preRef.current.scrollLeft = textareaRef.current.scrollLeft;
    }
  }, []);

  const handleSave = async () => {
    setSaving(true);
    try {
      const msg = await saveRawConfig(toml);
      setOriginalToml(toml);
      setDirty(false);
      showToast("ok", msg);
    } catch (e) {
      showToast("error", `保存失败: ${errorMessage(e)}`);
    } finally {
      setSaving(false);
    }
  };

  const handleReset = () => {
    setToml(originalToml);
    setDirty(false);
  };

  const lineCount = useMemo(() => toml.split("\n").length, [toml]);
  const gutterLines = useMemo(
    () => Array.from({ length: lineCount }, (_, i) => (
      <div key={i + 1} className="toml-line-num">{i + 1}</div>
    )),
    [lineCount]
  );

  if (loadError) {
    return (
      <div>
        <div className="page-header">
          <h2 className="page-title">
            <IconSettings size={22} />
            设置
          </h2>
        </div>
        <div className="empty-state">
          <span style={{ color: "var(--status-error)" }}>加载失败: {loadError}</span>
          <button className="glass-btn glass-btn-accent" onClick={loadConfig} style={{ marginTop: 8, fontSize: 12 }}>
            重试
          </button>
        </div>
      </div>
    );
  }

  return (
    <div>
      <div className="page-header">
        <h2 className="page-title">
          <IconSettings size={22} />
          设置
        </h2>
        <div className="settings-actions">
          <button className="glass-btn" onClick={loadConfig}>
            <IconRefresh size={14} />
            重新加载
          </button>
          <button className="glass-btn" onClick={handleReset} disabled={!dirty}>
            撤销更改
          </button>
          <button
            className="glass-btn glass-btn-accent"
            onClick={handleSave}
            disabled={saving || !dirty}
          >
            {saving ? "保存中..." : "保存"}
          </button>
        </div>
      </div>

      {toast && <div className={`toast toast-${toast.type}`} role="status">{toast.msg}</div>}

      {/* Config path hint */}
      <div className="settings-path-bar">
        <span className="settings-path-label">配置文件路径</span>
        <code className="settings-path-value">{configPath}</code>
      </div>

      {/* TOML Editor with syntax highlighting */}
      <div className="toml-editor-wrap">
        <div className="toml-editor-gutter">
          {gutterLines}
        </div>
        <div className="toml-editor-area">
          <HighlightedToml toml={toml} />
          <textarea
            ref={textareaRef}
            className="toml-editor"
            value={toml}
            onChange={(e) => handleChange(e.target.value)}
            onScroll={handleScroll}
            spellCheck={false}
            wrap="off"
            aria-label="TOML 配置编辑器"
          />
        </div>
      </div>

      {dirty && (
        <div className="toml-dirty-hint">
          有未保存的更改
        </div>
      )}
    </div>
  );
}
