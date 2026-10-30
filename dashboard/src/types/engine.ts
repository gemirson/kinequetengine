export type NodeState = 'Stem' | 'Progenitor' | 'Specialized' | 'Apoptosis';

export interface EcmaNodeData {
  id: number;
  label?: string;
  state: NodeState;
  usage_count: number;
  entropy: number;     // 0.0 stable .. 1.0 chaotic
  connections: number;
  maturity: number;    // 0.0 .. 1.0
  x?: number;
  y?: number;
  vx?: number;
  vy?: number;
  radius?: number;
  isPathogen?: boolean; // Injected AIS antigen
  highlighted?: boolean;
}

export interface GraphEdgeData {
  from: number;
  to: number;
  weight: number;      // 0.0 .. 1.0
  pheromone: number;   // tau: 0.01 .. 10.0
  particles?: Array<{ progress: number; speed: number }>;
}

export interface SystemMetrics {
  p99LatencyMs: number;
  p50LatencyMs: number;
  throughputQps: number;
  pheromoneEvaporationRate: number;
  averageTau: number;
  apoptosisRatePerMin: number;
  stemCount: number;
  progenitorCount: number;
  specializedCount: number;
  apoptosisCount: number;
  immuneThreatsBlocked: number;
  walSizeBytes: number;
  activeMemoryMb: number;
  uptimeSeconds: number;
}

export interface PipelineStageMetric {
  name: string;
  durationMs: number;
  description: string;
  stageNum: number;
}
