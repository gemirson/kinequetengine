import React from 'react';
import { Play, Sparkles, ShieldAlert, Trash2, Cpu, Activity, RefreshCw } from 'lucide-react';

interface HeaderBarProps {
  onStimulateQuery: () => void;
  onMitosis: () => void;
  onCleanApoptosis: () => void;
  onTriggerImmuneTest: () => void;
  showHighways: boolean;
  onToggleHighways: () => void;
  isBackendConnected: boolean;
  onCheckBackend: () => void;
  activeCount: number;
  p99Latency: number;
}

export const HeaderBar: React.FC<HeaderBarProps> = ({
  onStimulateQuery,
  onMitosis,
  onCleanApoptosis,
  onTriggerImmuneTest,
  showHighways,
  onToggleHighways,
  isBackendConnected,
  onCheckBackend,
  activeCount,
  p99Latency,
}) => {
  return (
    <header className="h-16 bg-bio-surface border-b border-bio-border px-6 flex items-center justify-between select-none">
      {/* Brand & Living Status */}
      <div className="flex items-center gap-6">
        <div className="flex items-center gap-3">
          <div className="w-9 h-9 rounded-xl bg-gradient-to-tr from-bio-stem via-bio-specialized to-bio-pheromone flex items-center justify-center shadow-lg shadow-bio-specialized/20">
            <span className="text-lg">🧬</span>
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h1 className="font-bold text-bio-text text-base tracking-tight">KineQuetEngine</h1>
              <span className="text-[10px] px-1.5 py-0.5 rounded bg-bio-card border border-bio-border text-bio-muted font-mono">v6.0</span>
            </div>
            <p className="text-[11px] text-bio-muted flex items-center gap-1.5">
              <span className="w-2 h-2 rounded-full bg-bio-stem animate-pulse" />
              Living Cognitive Engine · Homeostatic
            </p>
          </div>
        </div>

        {/* Homeostasis Telemetry Badges */}
        <div className="hidden lg:flex items-center gap-3 border-l border-bio-border pl-6 text-xs">
          <div className="bg-bio-card px-3 py-1.5 rounded-lg border border-bio-border flex items-center gap-2">
            <Activity size={14} className="text-bio-specialized" />
            <span className="text-bio-muted">P99 Latency:</span>
            <span className="font-mono font-semibold text-bio-text">{p99Latency.toFixed(1)}ms</span>
          </div>
          <div className="bg-bio-card px-3 py-1.5 rounded-lg border border-bio-border flex items-center gap-2">
            <Cpu size={14} className="text-bio-stem" />
            <span className="text-bio-muted">Active Cells:</span>
            <span className="font-mono font-semibold text-bio-text">{activeCount}</span>
          </div>
        </div>
      </div>

      {/* Biological Action Triggers */}
      <div className="flex items-center gap-2">
        <button
          onClick={onStimulateQuery}
          className="flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg bg-bio-specialized text-white text-xs font-medium hover:bg-bio-specialized/90 transition shadow-lg shadow-bio-specialized/25 active:scale-95"
          title="Simulate query vector through ACO and Graph"
        >
          <Play size={14} fill="currentColor" />
          Stimulate Query
        </button>

        <button
          onClick={onMitosis}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-bio-card hover:bg-bio-border border border-bio-border text-bio-stem text-xs font-medium transition active:scale-95"
          title="Create new Stem cell nodes"
        >
          <Sparkles size={14} />
          Mitosis (+Stem)
        </button>

        <button
          onClick={onTriggerImmuneTest}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-bio-card hover:bg-bio-border border border-bio-border text-bio-shield text-xs font-medium transition active:scale-95"
          title="Simulate prompt injection / semantic anomaly"
        >
          <ShieldAlert size={14} />
          Test AIS Shield
        </button>

        <button
          onClick={onCleanApoptosis}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-bio-card hover:bg-bio-border border border-bio-border text-bio-muted hover:text-bio-apoptosis text-xs font-medium transition active:scale-95"
          title="Recycle and clean dead cells"
        >
          <Trash2 size={14} />
          Purge Apoptosis
        </button>

        <div className="w-[1px] h-6 bg-bio-border mx-1" />

        {/* Highway Toggle */}
        <button
          onClick={onToggleHighways}
          className={`px-3 py-1.5 rounded-lg text-xs font-medium border transition ${
            showHighways
              ? 'bg-bio-pheromone/20 text-bio-pheromone border-bio-pheromone/40'
              : 'bg-bio-card text-bio-muted border-bio-border hover:text-bio-text'
          }`}
          title="Filter to only show ACO Pheromone Highways"
        >
          Highways
        </button>

        {/* Backend Status indicator */}
        <button
          onClick={onCheckBackend}
          className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg bg-bio-card border border-bio-border text-xs text-bio-muted hover:text-bio-text transition"
          title="Click to check connection to local kce-server (port 8080)"
        >
          <span className={`w-2 h-2 rounded-full ${isBackendConnected ? 'bg-bio-stem' : 'bg-amber-500'}`} />
          <span className="font-mono text-[11px]">{isBackendConnected ? 'kce:8080' : 'Sim Mode'}</span>
          <RefreshCw size={12} className="opacity-70 hover:opacity-100" />
        </button>
      </div>
    </header>
  );
};
