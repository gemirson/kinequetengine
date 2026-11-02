import { PipelineStageMetric } from '../types/engine';

export interface StageDefinition {
  stageNum: number;
  key: string;
  name: string;
  description: string;
  defaultDurationMs: number;
}

export const STAGE_DEFINITIONS: StageDefinition[] = [
  {
    stageNum: 1,
    key: 'validate',
    name: 'Validation & Limits',
    description: 'Vector format & bounds validation',
    defaultDurationMs: 0.12,
  },
  {
    stageNum: 2,
    key: 'rate_limit',
    name: 'Tenant Semaphore',
    description: 'Rate limit & concurrency slots',
    defaultDurationMs: 0.08,
  },
  {
    stageNum: 3,
    key: 'plan',
    name: 'Execution Plan',
    description: 'Query decomposition & routing',
    defaultDurationMs: 0.15,
  },
  {
    stageNum: 4,
    key: 'retrieve',
    name: 'SIMD Vector Retrieval',
    description: 'Cosine AVX2/Rayon similarity',
    defaultDurationMs: 1.15,
  },
  {
    stageNum: 5,
    key: 'expand',
    name: 'Semantic Expansion',
    description: 'Graph BFS/DFS 2-hop traversal',
    defaultDurationMs: 0.65,
  },
  {
    stageNum: 6,
    key: 'ecma_update',
    name: 'ECMA State Decay',
    description: 'Entropy & apoptosis evolution',
    defaultDurationMs: 0.28,
  },
  {
    stageNum: 7,
    key: 'mce_encode',
    name: 'mRNA Synthesis',
    description: 'Context to executable instruction',
    defaultDurationMs: 0.4,
  },
  {
    stageNum: 8,
    key: 'execute',
    name: 'Action Execution',
    description: 'Direct decision dispatch',
    defaultDurationMs: 0.35,
  },
  {
    stageNum: 9,
    key: 'persist',
    name: 'WAL Checksum Commit',
    description: 'KineSQL page manager write',
    defaultDurationMs: 0.25,
  },
  {
    stageNum: 10,
    key: 'metrics',
    name: 'Metrics Telemetry',
    description: 'Percentile histogram recording',
    defaultDurationMs: 0.18,
  },
];

/**
 * Map API response stages (Record<string, number>) to the 10-stage display metrics.
 */
export function mapApiStagesToMetrics(
  apiStages?: Record<string, number>
): PipelineStageMetric[] {
  return STAGE_DEFINITIONS.map(def => {
    const rawDuration = apiStages ? apiStages[def.key] : undefined;
    const durationMs =
      rawDuration !== undefined
        ? parseFloat(rawDuration.toFixed(3))
        : def.defaultDurationMs;

    return {
      stageNum: def.stageNum,
      name: def.name,
      description: def.description,
      durationMs,
    };
  });
}
