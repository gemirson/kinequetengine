import React, { useState, useEffect, useCallback } from 'react';
import { generateInitialGraph } from './data/mockData';
import { EcmaNodeData, GraphEdgeData, PipelineStageMetric } from './types/engine';
import { BioGraphCanvas } from './components/BioGraphCanvas';
import { NodeInspector } from './components/NodeInspector';
import { HeaderBar } from './components/HeaderBar';
import { PipelineTelemetry } from './components/PipelineTelemetry';

const DEFAULT_STAGES: PipelineStageMetric[] = [
  { stageNum: 1, name: 'Validation & Limits', description: 'Tenant semaphore check', durationMs: 0.12 },
  { stageNum: 2, name: 'AIS Threat Filter', description: 'Antigen self/non-self check', durationMs: 0.45 },
  { stageNum: 3, name: 'SIMD Vector Retrieval', description: 'Cosine AVX2/NEON similarity', durationMs: 1.15 },
  { stageNum: 4, name: 'ACO Highway Routing', description: 'Pheromone path reinforcement', durationMs: 0.82 },
  { stageNum: 5, name: 'Semantic Expansion', description: 'BFS/DFS 2-hop traversal', durationMs: 0.65 },
  { stageNum: 6, name: 'ECMA State Decay', description: 'Entropy & apoptosis eval', durationMs: 0.28 },
  { stageNum: 7, name: 'mRNA Synthesis', description: 'Context into executable codon', durationMs: 0.40 },
  { stageNum: 8, name: 'Action Execution', description: 'Direct decision dispatch', durationMs: 0.35 },
  { stageNum: 9, name: 'Pheromone Evap', description: 'Global tau evaporation step', durationMs: 0.18 },
  { stageNum: 10, name: 'WAL Checksum Commit', description: 'KineSQL page manager write', durationMs: 0.52 },
];

export const App: React.FC = () => {
  const [graph, setGraph] = useState(() => generateInitialGraph(26));
  const [selectedNode, setSelectedNode] = useState<EcmaNodeData | null>(null);
  const [showHighways, setShowHighways] = useState(false);
  const [activePathNodes, setActivePathNodes] = useState<number[]>([]);
  const [lastLatencyMs, setLastLatencyMs] = useState(4.92);
  const [lastAction, setLastAction] = useState('RiskEval::ApprovePayload');
  const [stages, setStages] = useState<PipelineStageMetric[]>(DEFAULT_STAGES);
  const [anomalyBlocked, setAnomalyBlocked] = useState(false);
  const [isBackendConnected, setIsBackendConnected] = useState(false);

  // Check connection to local KCE engine backend
  const checkBackend = useCallback(async () => {
    try {
      const res = await fetch('/api/health');
      if (res.ok) {
        setIsBackendConnected(true);
      } else {
        setIsBackendConnected(false);
      }
    } catch {
      setIsBackendConnected(false);
    }
  }, []);

  useEffect(() => {
    checkBackend();
  }, [checkBackend]);

  // Biological Action: Stimulate Query (Send ants through ACO trails)
  const handleStimulateQuery = () => {
    setAnomalyBlocked(false);

    // Pick 3 to 5 connected nodes for the query path
    const nodes = graph.nodes;
    if (nodes.length < 3) return;

    const startNode = nodes[Math.floor(Math.random() * nodes.length)];
    const path: number[] = [startNode.id];

    // Find path using graph edges
    let curr = startNode.id;
    for (let step = 0; step < 3; step++) {
      const nextEdge = graph.edges.find(e => (e.from === curr || e.to === curr) && !path.includes(e.from === curr ? e.to : e.from));
      if (nextEdge) {
        curr = nextEdge.from === curr ? nextEdge.to : nextEdge.from;
        path.push(curr);
      }
    }

    setActivePathNodes(path);

    // Deposit pheromones along the traversed edges
    setGraph(prev => {
      const updatedEdges = prev.edges.map(edge => {
        const isTraversed =
          (path.includes(edge.from) && path.includes(edge.to));
        if (isTraversed) {
          return {
            ...edge,
            pheromone: Math.min(edge.pheromone + 1.2, 10.0), // Deposit pheromone
            particles: [
              ...(edge.particles || []),
              { progress: 0, speed: 0.015 + Math.random() * 0.01 },
            ],
          };
        }
        return edge;
      });

      // Increase usage count on traversed nodes
      const updatedNodes = prev.nodes.map(n => {
        if (path.includes(n.id)) {
          return {
            ...n,
            usage_count: n.usage_count + 1,
            maturity: Math.min(n.maturity + 0.02, 1.0),
          };
        }
        return n;
      });

      return { nodes: updatedNodes, edges: updatedEdges };
    });

    // Simulate realistic sub-millisecond to low ms pipeline latency
    const totalLatency = 3.2 + Math.random() * 2.8;
    setLastLatencyMs(totalLatency);

    const actions = [
      'RiskEval::CreditApproved',
      'DataEnrich::SynthesisComplete',
      'Classification::Tier1Entity',
      'ActionDispatch::ExecutePayment',
    ];
    setLastAction(actions[Math.floor(Math.random() * actions.length)]);

    // Slightly randomize stages
    setStages(prev =>
      prev.map(s => ({
        ...s,
        durationMs: parseFloat((s.durationMs * (0.85 + Math.random() * 0.3)).toFixed(2)),
      }))
    );

    // Clear active path highlight after 2.5s
    setTimeout(() => {
      setActivePathNodes([]);
    }, 2500);
  };

  // Biological Action: Mitosis (+New Stem Cells)
  const handleMitosis = () => {
    const newId = 2000 + Math.floor(Math.random() * 8000);
    const parentNode = graph.nodes[Math.floor(Math.random() * graph.nodes.length)];

    const newNode: EcmaNodeData = {
      id: newId,
      label: `Stem_Mitosis_${newId}`,
      state: 'Stem',
      usage_count: 1,
      entropy: 0.72,
      connections: 1,
      maturity: 0.1,
      x: (parentNode?.x || 0) + (Math.random() - 0.5) * 80,
      y: (parentNode?.y || 0) + (Math.random() - 0.5) * 80,
      vx: (Math.random() - 0.5) * 4,
      vy: (Math.random() - 0.5) * 4,
      radius: 9,
    };

    const newEdge: GraphEdgeData = {
      from: parentNode.id,
      to: newId,
      weight: 0.5,
      pheromone: 1.0,
      particles: [{ progress: 0, speed: 0.006 }],
    };

    setGraph(prev => ({
      nodes: [...prev.nodes, newNode],
      edges: [...prev.edges, newEdge],
    }));
  };

  // Biological Action: Clean Apoptosis (Recycle dead cells)
  const handleCleanApoptosis = () => {
    const apoptosisIds = graph.nodes.filter(n => n.state === 'Apoptosis').map(n => n.id);
    if (apoptosisIds.length === 0) return;

    setGraph(prev => ({
      nodes: prev.nodes.filter(n => !apoptosisIds.includes(n.id)),
      edges: prev.edges.filter(e => !apoptosisIds.includes(e.from) && !apoptosisIds.includes(e.to)),
    }));

    if (selectedNode && apoptosisIds.includes(selectedNode.id)) {
      setSelectedNode(null);
    }
  };

  // Biological Action: Test AIS Immune Shield (Semantic Anomaly Attack)
  const handleTriggerImmuneTest = () => {
    setAnomalyBlocked(true);

    const pathogenId = 9999;
    const existing = graph.nodes.find(n => n.id === pathogenId);
    if (existing) return;

    const pathogenNode: EcmaNodeData = {
      id: pathogenId,
      label: 'MALICIOUS_PROMPT_INJECTION',
      state: 'Apoptosis',
      usage_count: 0,
      entropy: 0.99,
      connections: 1,
      maturity: 0.0,
      x: 0,
      y: 0,
      radius: 13,
      isPathogen: true,
    };

    const anchorNode = graph.nodes[0];
    const pathogenEdge: GraphEdgeData = {
      from: pathogenId,
      to: anchorNode.id,
      weight: 0.1,
      pheromone: 0.01, // Pheromone penalty
    };

    setGraph(prev => ({
      nodes: [pathogenNode, ...prev.nodes],
      edges: [pathogenEdge, ...prev.edges],
    }));

    setSelectedNode(pathogenNode);

    // AIS automatically marks it for immediate apoptosis and neutralizes it after 4 seconds
    setTimeout(() => {
      setGraph(prev => ({
        nodes: prev.nodes.filter(n => n.id !== pathogenId),
        edges: prev.edges.filter(e => e.from !== pathogenId && e.to !== pathogenId),
      }));
      setAnomalyBlocked(false);
      setSelectedNode(null);
    }, 4000);
  };

  // Change individual node state
  const handleStateChange = (nodeId: number, newState: EcmaNodeData['state']) => {
    setGraph(prev => ({
      ...prev,
      nodes: prev.nodes.map(n => (n.id === nodeId ? { ...n, state: newState } : n)),
    }));
    if (selectedNode && selectedNode.id === nodeId) {
      setSelectedNode(prev => (prev ? { ...prev, state: newState } : null));
    }
  };

  // Reinforce ACO pheromone
  const handleReinforcePheromone = (nodeId: number) => {
    setGraph(prev => ({
      ...prev,
      edges: prev.edges.map(e => {
        if (e.from === nodeId || e.to === nodeId) {
          return { ...e, pheromone: Math.min(e.pheromone + 2.5, 10.0) };
        }
        return e;
      }),
    }));
  };

  return (
    <div className="flex flex-col h-screen w-screen overflow-hidden bg-bio-bg">
      {/* Top Header & Homeostasis Actions */}
      <HeaderBar
        onStimulateQuery={handleStimulateQuery}
        onMitosis={handleMitosis}
        onCleanApoptosis={handleCleanApoptosis}
        onTriggerImmuneTest={handleTriggerImmuneTest}
        showHighways={showHighways}
        onToggleHighways={() => setShowHighways(!showHighways)}
        isBackendConnected={isBackendConnected}
        onCheckBackend={checkBackend}
        activeCount={graph.nodes.length}
        p99Latency={lastLatencyMs}
      />

      {/* Main Graph & Inspector Area */}
      <div className="flex-1 flex relative overflow-hidden">
        {/* Interactive Bio-Canvas (Grafo Vivo) */}
        <div className="flex-1 h-full relative">
          <BioGraphCanvas
            nodes={graph.nodes}
            edges={graph.edges}
            selectedNode={selectedNode}
            onSelectNode={setSelectedNode}
            showPheromoneHighways={showHighways}
            activePathNodes={activePathNodes}
          />
        </div>

        {/* Right Node Inspector Drawer */}
        {selectedNode && (
          <NodeInspector
            node={selectedNode}
            edges={graph.edges}
            onClose={() => setSelectedNode(null)}
            onStateChange={handleStateChange}
            onReinforcePheromone={handleReinforcePheromone}
          />
        )}
      </div>

      {/* Bottom 10-Stage Cognitive Pipeline Telemetry */}
      <PipelineTelemetry
        lastQueryTimeMs={lastLatencyMs}
        lastAction={lastAction}
        stages={stages}
        anomalyBlocked={anomalyBlocked}
      />
    </div>
  );
};

export default App;
