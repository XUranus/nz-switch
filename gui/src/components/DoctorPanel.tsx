import React, { useState, useEffect, useCallback } from "react";
import { runDoctor } from "../api";
import type { DoctorCheck } from "../types";
import { IconCheck, IconWarn, IconError, IconRefresh } from "../icons";
import { errorMessage } from "../utils";

function StatusIcon({ status }: { status: DoctorCheck["status"] }) {
  switch (status) {
    case "ok": return <span className="doctor-icon doctor-icon-ok"><IconCheck size={14} /></span>;
    case "warn": return <span className="doctor-icon doctor-icon-warn"><IconWarn size={14} /></span>;
    case "error": return <span className="doctor-icon doctor-icon-error"><IconError size={14} /></span>;
    default: return <span className="doctor-icon"><IconWarn size={14} /></span>;
  }
}

export default function DoctorPanel() {
  const [checks, setChecks] = useState<DoctorCheck[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await runDoctor();
      setChecks(data);
    } catch (e) {
      const msg = errorMessage(e);
      console.error("Failed to run doctor:", msg);
      setError(msg);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const okCount = checks.filter((c) => c.status === "ok").length;
  const warnCount = checks.filter((c) => c.status === "warn").length;
  const errorCount = checks.filter((c) => c.status === "error").length;

  return (
    <div>
      <div className="page-header">
        <h2 className="page-title">环境诊断</h2>
        <button className="glass-btn" onClick={load} disabled={loading}>
          <IconRefresh size={14} />
          {loading ? "诊断中..." : "重新诊断"}
        </button>
      </div>

      {/* Summary Cards */}
      <div className="doctor-summary">
        <StatCard value={checks.length} label="检查项" color="var(--text-primary)" loading={loading} />
        <StatCard value={okCount} label="正常" color="var(--status-ok)" loading={loading} />
        <StatCard value={warnCount} label="警告" color="var(--status-warn)" loading={loading} />
        <StatCard value={errorCount} label="错误" color="var(--status-error)" loading={loading} />
      </div>

      {/* Error State */}
      {error && (
        <div className="glass-card doctor-error-card">
          <span style={{ color: "var(--status-error)" }}>诊断失败: {error}</span>
          <button className="glass-btn glass-btn-accent" onClick={load} style={{ marginTop: 8, fontSize: 12 }}>
            重试
          </button>
        </div>
      )}

      {/* Checks List */}
      <div className="glass-card doctor-checks-card">
        {checks.length === 0 && !loading ? (
          <div className="empty-state">无检查结果</div>
        ) : (
          checks.map((check) => (
            <div key={check.name} className="doctor-check-item">
              <StatusIcon status={check.status} />
              <div className="doctor-check-body">
                <div className="doctor-check-name">{check.name}</div>
                <div className={`doctor-check-msg doctor-check-${check.status}`}>
                  {check.message}
                </div>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}

const StatCard = React.memo(function StatCard({ value, label, color, loading }: { value: number; label: string; color: string; loading: boolean }) {
  return (
    <div className="stat-card">
      <div className="stat-value" style={{ color: loading ? "var(--text-tertiary)" : color }}>
        {loading ? "..." : value}
      </div>
      <div className="stat-label">{label}</div>
    </div>
  );
});
