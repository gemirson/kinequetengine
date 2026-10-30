import React from 'react';
import { EcmaNodeData, GraphEdgeData } from '../types/engine';
import { X, Zap, Skull, Activity, Share2 } from 'lucide-react';

interface NodeInspectorProps {
  node: EcmaNodeData | null;
  edges: GraphEdgeData[];
  onClose: () => void;
  onStateChange: (nodeId: number, newState: EcmaNodeData['state']) => void;
  onReinforcePheromone: (nodeId: number) => void;
}

export const NodeInspector: React.FC<NodeInspectorProps> = ({
  node,
  edges,
  onClose,
  onStateChange,
  onReinforcePheromone,
}) => {
  if (!node) return null;

  // Find neighbor edges
  const neighborEdges = edges.filter(e => e.from === node.id || e.to === node.id);

  const getStateBadge = (state: EcmaNodeData['state']) => {
    switch (state) {
      case 'Stem':
        return <span className="px-2 py-0.5 rounded text-xs bg-bio-stem/20 text-bio-stem border border-bio-stem/40 font-mono">STEM CELL</span>;
      case 'Progenitor':
        return <span className="px-2 py-0.5 rounded text-xs bg-bio-progenitor/20 text-bio-progenitor border border-bio-progenitor/40 font-mono">PROGENITOR</span>;
      case 'Specialized':
        return <span className="px-2 py-0.5 rounded text-xs bg-bio-specialized/20 text-bio-specialized border border-bio-specialized/40 font-mono">SPECIALIZED</span>;
      case 'Apoptosis':
        return <span className="px-2 py-0.5 rounded text-xs bg-bio-apoptosis/20 text-bio-apoptosis border border-bio-apoptosis/40 font-mono">APOPTOSIS</span>;
    }
  };

  return (
    <div className="w-84 bg-bio-surface/95 backdrop-blur border-l border-bio-border h-full flex flex-col p-5 shadow-2xl overflow-y-auto">
      {/* Header */}
      <div className="flex items-center justify-between pb-4 border-b border-bio-border">
        <div>
          <div className="text-[11px] font-mono text-bio-muted uppercase tracking-wider">Node Inspector</div>
          <h3 className="font-semibold text-bio-text text-base truncate max-w-[200px]" title={node.label}>
            {node.label || `Node #${node.id}`}
          </h3>
        </div>
        <button
          onClick={onClose}
          className="p-1 rounded-lg hover:bg-bio-card text-bio-muted hover:text-bio-text transition"
        >
          <X size={18} />
        </button>
      </div>

      {/* State & Core Meta */}
      <div className="py-4 space-y-3 border-b border-bio-border">
        <div className="flex items-center justify-between">
          <span className="text-xs text-bio-muted">Lifecycle State</span>
          {getStateBadge(node.state)}
        </div>
        <div className="flex items-center justify-between text-xs">
          <span className="text-bio-muted font-mono">ID</span>
          <span className="font-mono text-bio-text">#{node.id}</span>
        </div>
        <div className="flex items-center justify-between text-xs">
          <span className="text-bio-muted">Usage Count</span>
          <span className="font-mono text-bio-text">{node.usage_count.toLocaleString()} calls</span>
        </div>
        <div className="flex items-center justify-between text-xs">
          <span className="text-bio-muted">Total Synapses</span>
          <span className="font-mono text-bio-text">{neighborEdges.length} edges</span>
        </div>
      </div>

      {/* Biological Metrics */}
      <div className="py-4 space-y-4 border-b border-bio-border">
        <div className="text-xs font-semibold text-bio-muted uppercase tracking-wider">Cellular Properties</div>

        {/* Maturity */}
        <div className="space-y-1">
          <div className="flex justify-between text-xs">
            <span className="text-bio-muted">Maturity Factor</span>
            <span className="font-mono text-bio-specialized font-medium">{Math.round(node.maturity * 100)}%</span>
          </div>
          <div className="w-full h-2 bg-bio-card rounded-full overflow-hidden border border-bio-border">
            <div
              className="h-full bg-gradient-to-r from-bio-stem via-bio-progenitor to-bio-specialized rounded-full transition-all duration-300"
              style={{ width: `${Math.min(node.maturity * 100, 100)}%` }}
            />
          </div>
        </div>

        {/* Entropy */}
        <div className="space-y-1">
          <div className="flex justify-between text-xs">
            <span className="text-bio-muted">Entropy (Chaos Level)</span>
            <span className={`font-mono font-medium ${node.entropy > 0.5 ? 'text-bio-apoptosis' : 'text-bio-stem'}`}>
              {node.entropy.toFixed(3)}
            </span>
          </div>
          <div className="w-full h-2 bg-bio-card rounded-full overflow-hidden border border-bio-border">
            <div
              className={`h-full rounded-full transition-all duration-300 ${
                node.entropy > 0.5 ? 'bg-bio-apoptosis' : 'bg-bio-stem'
              }`}
              style={{ width: `${Math.min(node.entropy * 100, 100)}%` }}
            />
          </div>
        </div>
      </div>

      {/* Neighbor Synapses */}
      <div className="py-4 space-y-3 border-b border-bio-border flex-1">
        <div className="flex items-center justify-between">
          <span className="text-xs font-semibold text-bio-muted uppercase tracking-wider">ACO Synapses</span>
          <span className="text-[10px] font-mono text-bio-pheromone">τ level</span>
        </div>
        <div className="space-y-2 max-h-48 overflow-y-auto pr-1">
          {neighborEdges.length === 0 ? (
            <div className="text-xs text-bio-muted italic">No active connections.</div>
          ) : (
            neighborEdges.map((e, idx) => {
              const otherId = e.from === node.id ? e.to : e.from;
              return (
                <div
                  key={idx}
                  className="p-2 bg-bio-card rounded-lg border border-bio-border flex items-center justify-between text-xs"
                >
                  <div className="flex items-center gap-1.5">
                    <Share2 size={13} className="text-bio-muted" />
                    <span className="font-mono text-bio-text">Node #{otherId}</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="text-[10px] text-bio-muted">w: {e.weight}</span>
                    <span className="font-mono font-semibold text-bio-pheromone">
                      {e.pheromone.toFixed(1)}τ
                    </span>
                  </div>
                </div>
              );
            })
          )}
        </div>
      </div>

      {/* Biological Intervention Actions */}
      <div className="pt-4 space-y-2">
        <div className="text-xs font-semibold text-bio-muted uppercase tracking-wider mb-2">Interventions</div>
        
        <button
          onClick={() => onReinforcePheromone(node.id)}
          className="w-full flex items-center justify-center gap-2 px-3 py-2 bg-bio-pheromone/15 hover:bg-bio-pheromone/25 text-bio-pheromone border border-bio-pheromone/40 rounded-lg text-xs font-medium transition"
        >
          <Zap size={14} />
          Reinforce Pheromones (+ACO)
        </button>

        {node.state !== 'Specialized' && (
          <button
            onClick={() => onStateChange(node.id, node.state === 'Stem' ? 'Progenitor' : 'Specialized')}
            className="w-full flex items-center justify-center gap-2 px-3 py-2 bg-bio-specialized/15 hover:bg-bio-specialized/25 text-bio-specialized border border-bio-specialized/40 rounded-lg text-xs font-medium transition"
          >
            <Activity size={14} />
            Differentiate Cell ({node.state === 'Stem' ? 'Stem → Progenitor' : 'Progenitor → Mature'})
          </button>
        )}

        {node.state !== 'Apoptosis' && (
          <button
            onClick={() => onStateChange(node.id, 'Apoptosis')}
            className="w-full flex items-center justify-center gap-2 px-3 py-2 bg-bio-apoptosis/10 hover:bg-bio-apoptosis/20 text-bio-apoptosis border border-bio-apoptosis/30 rounded-lg text-xs font-medium transition"
          >
            <Skull size={14} />
            Trigger Apoptosis (Decay)
          </button>
        )}
      </div>
    </div>
  );
};
