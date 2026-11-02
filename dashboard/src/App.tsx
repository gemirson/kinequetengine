import React, { useState, useEffect, useCallback } from 'react';
import { generateInitialGraph } from './data/mockData';
import { EcmaNodeData, GraphEdgeData, PipelineStageMetric } from './types/engine';
import { BioGraphCanvas } from './components/BioGraphCanvas';
import { NodeInspector } from './components/NodeInspector';
import { HeaderBar } from './components/HeaderBar';
import { PipelineTelemetry } from './components/PipelineTelemetry';
import { mapApiStagesToMetrics } from './lib/stageMap';
import { mapGraphResponse } from './api/mapping';
import { subscribeToEngineEvents } from './api/events';
import {
  useHealthQuery,
  useGraphQuery,
  useQueryMutation,
  useCreateNodeMutation,
  useDeleteNodeMutation,
  useSetNodeStateMutation,
  useDepositMutation,
  useAisProbeMutation,
} from './api/hooks';

export const App: React.FC = () => {
  // Local graph state (either populated by mockData in Sim Mode, or by live /graph)
  const [graph, setGraph] = useState(() => generateInitialGraph(26));
  const [selectedNode, setSelectedNode] = useState<EcmaNodeData | null>(null);
  const [showHighways, setShowHighways] = useState(false);
  const [activePathNodes, setActivePathNodes] = useState<number[]>([]);
  const [lastLatencyMs, setLastLatencyMs] = useState(4.92);
  const [lastAction, setLastAction] = useState('RiskEval::ApprovePayload');
  const [stages, setStages] = useState<PipelineStageMetric[]>(() => mapApiStagesToMetrics());
  const [anomalyBlocked, setAnomalyBlocked] = useState(false);

  // React Query hooks
  const { data: healthData, isSuccess: isHealthSuccess, refetch: refetchHealth } = useHealthQuery();
  const isBackendConnected = isHealthSuccess && healthData?.status !== 'Unhealthy';

  const { data: graphResponse, refetch: refetchGraph } = useGraphQuery(isBackendConnected);

  const queryMutation = useQueryMutation();
  const createNodeMutation = useCreateNodeMutation();
  const deleteNodeMutation = useDeleteNodeMutation();
  const setNodeStateMutation = useSetNodeStateMutation();
  const depositMutation = useDepositMutation();
  const aisProbeMutation = useAisProbeMutation();

  // Sync live graph data when backend is connected
  useEffect(() => {
    if (isBackendConnected && graphResponse) {
      setGraph(prev => mapGraphResponse(graphResponse, prev.nodes, prev.edges));
    }
  }, [isBackendConnected, graphResponse]);

  // Subscribe to live SSE events when connected
  useEffect(() => {
    if (!isBackendConnected) return;

    const unsubscribe = subscribeToEngineEvents({
      onEvent: event => {
        switch (event.type) {
          case 'query_completed':
            setLastLatencyMs(event.latency_ms);
            setLastAction(event.action);
            refetchGraph();
            break;
          case 'node_created':
          case 'node_removed':
            refetchGraph();
            break;
          case 'pheromone_deposited':
            setGraph(prev => ({
              ...prev,
              edges: prev.edges.map(e => {
                if (
                  (e.from === event.from && e.to === event.to) ||
                  (e.from === event.to && e.to === event.from)
                ) {
                  return { ...e, pheromone: event.level };
                }
                return e;
              }),
            }));
            break;
          case 'ais_anomaly':
            setAnomalyBlocked(true);
            setTimeout(() => setAnomalyBlocked(false), 4000);
            break;
        }
      },
    });

    return unsubscribe;
  }, [isBackendConnected, refetchGraph]);

  // Check connection handler for manual refresh button
  const handleCheckBackend = useCallback(() => {
    refetchHealth();
  }, [refetchHealth]);

  // Biological Action: Stimulate Query
  const handleStimulateQuery = async () => {
    setAnomalyBlocked(false);

    if (isBackendConnected) {
      try {
        const queryVector = Array.from({ length: 128 }, () => (Math.random() - 0.5) * 0.1);
        const res = await queryMutation.mutateAsync({
          query_vector: queryVector,
          top_k: 5,
        });

        setLastLatencyMs(res.pipeline_ms);
        setLastAction(res.action);
        setStages(mapApiStagesToMetrics(res.stages));

        // Pick 2-3 nodes to highlight active path visually
        if (graph.nodes.length > 0) {
          const sampleCount = Math.min(graph.nodes.length, 3);
          const sample = graph.nodes.slice(0, sampleCount).map(n => n.id);
          setActivePathNodes(sample);
          setTimeout(() => setActivePathNodes([]), 2500);
        }
        return;
      } catch (err) {
        console.warn('Live query failed, falling back to local simulation:', err);
      }
    }

    // Sim Mode fallback
    const nodes = graph.nodes;
    if (nodes.length < 3) return;

    const startNode = nodes[Math.floor(Math.random() * nodes.length)];
    const path: number[] = [startNode.id];

    let curr = startNode.id;
    for (let step = 0; step < 3; step++) {
      const nextEdge = graph.edges.find(
        e => (e.from === curr || e.to === curr) && !path.includes(e.from === curr ? e.to : e.from)
      );
      if (nextEdge) {
        curr = nextEdge.from === curr ? nextEdge.to : nextEdge.from;
        path.push(curr);
      }
    }

    setActivePathNodes(path);

    setGraph(prev => {
      const updatedEdges = prev.edges.map(edge => {
        const isTraversed = path.includes(edge.from) && path.includes(edge.to);
        if (isTraversed) {
          return {
            ...edge,
            pheromone: Math.min(edge.pheromone + 1.2, 10.0),
            particles: [
              ...(edge.particles || []),
              { progress: 0, speed: 0.015 + Math.random() * 0.01 },
            ],
          };
        }
        return edge;
      });

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

    const totalLatency = 3.2 + Math.random() * 2.8;
    setLastLatencyMs(totalLatency);

    const actions = [
      'RiskEval::CreditApproved',
      'DataEnrich::SynthesisComplete',
      'Classification::Tier1Entity',
      'ActionDispatch::ExecutePayment',
    ];
    setLastAction(actions[Math.floor(Math.random() * actions.length)]);

    setStages(prev =>
      prev.map(s => ({
        ...s,
        durationMs: parseFloat((s.durationMs * (0.85 + Math.random() * 0.3)).toFixed(2)),
      }))
    );

    setTimeout(() => {
      setActivePathNodes([]);
    }, 2500);
  };

  // Biological Action: Mitosis (+New Stem Cells)
  const handleMitosis = async () => {
    const parentNode = graph.nodes[Math.floor(Math.random() * graph.nodes.length)];

    if (isBackendConnected) {
      try {
        const res = await createNodeMutation.mutateAsync({
          connect_to: parentNode?.id,
          weight: 0.5,
        });
        const newId = res.id;
        const newNode: EcmaNodeData = {
          id: newId,
          label: `Stem_Mitosis_${newId}`,
          state: 'Stem',
          usage_count: 1,
          entropy: 0.72,
          connections: parentNode ? 1 : 0,
          maturity: 0.1,
          x: (parentNode?.x || 0) + (Math.random() - 0.5) * 80,
          y: (parentNode?.y || 0) + (Math.random() - 0.5) * 80,
          vx: (Math.random() - 0.5) * 4,
          vy: (Math.random() - 0.5) * 4,
          radius: 9,
        };
        setGraph(prev => ({
          ...prev,
          nodes: [...prev.nodes, newNode],
        }));
        return;
      } catch (err) {
        console.warn('Live mitosis failed, falling back to local simulation:', err);
      }
    }

    // Sim Mode fallback
    const newId = 2000 + Math.floor(Math.random() * 8000);
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
      from: parentNode?.id ?? 1,
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
  const handleCleanApoptosis = async () => {
    const apoptosisIds = graph.nodes.filter(n => n.state === 'Apoptosis').map(n => n.id);
    if (apoptosisIds.length === 0) return;

    if (isBackendConnected) {
      try {
        await Promise.all(apoptosisIds.map(id => deleteNodeMutation.mutateAsync(id)));
      } catch (err) {
        console.warn('Live apoptosis cleanup failed:', err);
      }
    }

    setGraph(prev => ({
      nodes: prev.nodes.filter(n => !apoptosisIds.includes(n.id)),
      edges: prev.edges.filter(e => !apoptosisIds.includes(e.from) && !apoptosisIds.includes(e.to)),
    }));

    if (selectedNode && apoptosisIds.includes(selectedNode.id)) {
      setSelectedNode(null);
    }
  };

  // Biological Action: Test AIS Immune Shield (Semantic Anomaly Attack)
  const handleTriggerImmuneTest = async () => {
    setAnomalyBlocked(true);

    if (isBackendConnected) {
      try {
        const pathogenVector = Array(128).fill(3.0);
        await aisProbeMutation.mutateAsync({ vector: pathogenVector });
      } catch (err) {
        console.warn('AIS probe error:', err);
      }
    }

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
      to: anchorNode?.id ?? 1,
      weight: 0.1,
      pheromone: 0.01,
    };

    setGraph(prev => ({
      nodes: [pathogenNode, ...prev.nodes],
      edges: [pathogenEdge, ...prev.edges],
    }));

    setSelectedNode(pathogenNode);

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
  const handleStateChange = async (nodeId: number, newState: EcmaNodeData['state']) => {
    setGraph(prev => ({
      ...prev,
      nodes: prev.nodes.map(n => (n.id === nodeId ? { ...n, state: newState } : n)),
    }));
    if (selectedNode && selectedNode.id === nodeId) {
      setSelectedNode(prev => (prev ? { ...prev, state: newState } : null));
    }

    if (isBackendConnected) {
      try {
        await setNodeStateMutation.mutateAsync({ id: nodeId, state: newState });
      } catch (err) {
        console.warn('Live state update failed:', err);
      }
    }
  };

  // Reinforce ACO pheromone
  const handleReinforcePheromone = async (nodeId: number) => {
    const connectedEdges = graph.edges.filter(e => e.from === nodeId || e.to === nodeId);

    setGraph(prev => ({
      ...prev,
      edges: prev.edges.map(e => {
        if (e.from === nodeId || e.to === nodeId) {
          return { ...e, pheromone: Math.min(e.pheromone + 2.5, 10.0) };
        }
        return e;
      }),
    }));

    if (isBackendConnected) {
      try {
        await Promise.all(
          connectedEdges.map(e =>
            depositMutation.mutateAsync({
              from: e.from,
              to: e.to,
              amount: 2.5,
            })
          )
        );
      } catch (err) {
        console.warn('Live pheromone reinforcement failed:', err);
      }
    }
  };

  return (
    <div className="flex flex-col h-screen w-screen overflow-hidden bg-bio-bg">
      <HeaderBar
        onStimulateQuery={handleStimulateQuery}
        onMitosis={handleMitosis}
        onCleanApoptosis={handleCleanApoptosis}
        onTriggerImmuneTest={handleTriggerImmuneTest}
        showHighways={showHighways}
        onToggleHighways={() => setShowHighways(!showHighways)}
        isBackendConnected={isBackendConnected}
        onCheckBackend={handleCheckBackend}
        activeCount={graph.nodes.length}
        p99Latency={lastLatencyMs}
      />

      <div className="flex-1 flex relative overflow-hidden">
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
