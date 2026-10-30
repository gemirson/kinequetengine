import React, { useState } from 'react';
import { ChevronUp, ChevronDown, CheckCircle2, AlertTriangle, Zap } from 'lucide-react';
import { PipelineStageMetric } from '../types/engine';

interface PipelineTelemetryProps {
  lastQueryTimeMs: number;
  lastAction: string;
  stages: PipelineStageMetric[];
  anomalyBlocked: boolean;
}

export const PipelineTelemetry: React.FC<PipelineTelemetryProps> = ({
  lastQueryTimeMs,
  lastAction,
  stages,
  anomalyBlocked,
}) => {
  const [isExpanded, setIsExpanded] = useState(false);

  return (
    <div className="bg-bio-surface border-t border-bio-border select-none transition-all">
      {/* Mini Bar Summary */}
      <div
        onClick={() => setIsExpanded(!isExpanded)}
        className="h-11 px-6 flex items-center justify-between cursor-pointer hover:bg-bio-card/50 transition"
      >
        <div className="flex items-center gap-4 text-xs">
          <div className="flex items-center gap-1.5 font-semibold text-bio-text">
            <Zap size={14} className="text-bio-specialized" />
            <span>10-Stage Cognitive Pipeline</span>
          </div>

          <div className="flex items-center gap-2">
            <span className="text-bio-muted">Latency:</span>
            <span className="font-mono font-bold text-bio-stem">{lastQueryTimeMs.toFixed(2)} ms</span>
          </div>

          <div className="hidden sm:flex items-center gap-2">
            <span className="text-bio-muted">mRNA Intent:</span>
            <span className="font-mono text-bio-specialized font-medium">{lastAction}</span>
          </div>

          {anomalyBlocked && (
            <div className="flex items-center gap-1.5 px-2 py-0.5 rounded bg-bio-apoptosis/20 text-bio-apoptosis border border-bio-apoptosis/30 text-[11px] font-medium animate-pulse">
              <AlertTriangle size={12} />
              <span>AIS Blocked Threat</span>
            </div>
          )}
        </div>

        <div className="flex items-center gap-2 text-bio-muted text-xs">
          <span>{isExpanded ? 'Hide Stage Breakdown' : 'Show 10 Stages Breakdown'}</span>
          {isExpanded ? <ChevronDown size={16} /> : <ChevronUp size={16} />}
        </div>
      </div>

      {/* Expanded Stage Grid */}
      {isExpanded && (
        <div className="p-5 border-t border-bio-border bg-bio-bg/50">
          <div className="grid grid-cols-2 md:grid-cols-5 gap-3">
            {stages.map((stage) => (
              <div
                key={stage.stageNum}
                className="p-3 bg-bio-card rounded-lg border border-bio-border flex flex-col justify-between"
              >
                <div className="flex items-center justify-between mb-1">
                  <span className="text-[10px] font-mono text-bio-muted">Stage #{stage.stageNum}</span>
                  <CheckCircle2 size={12} className="text-bio-stem" />
                </div>
                <div className="font-medium text-xs text-bio-text truncate" title={stage.name}>
                  {stage.name}
                </div>
                <div className="text-[11px] text-bio-muted truncate mb-2">{stage.description}</div>
                <div className="flex items-center justify-between border-t border-bio-border pt-1.5 mt-auto">
                  <span className="text-[10px] text-bio-muted">Duration</span>
                  <span className="font-mono text-xs font-semibold text-bio-stem">
                    {stage.durationMs < 1 ? `${Math.round(stage.durationMs * 1000)} µs` : `${stage.durationMs.toFixed(2)} ms`}
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};
