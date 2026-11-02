import { EcmaNodeData, GraphEdgeData, NodeState } from '../types/engine';
import { components } from './schema';

export type GraphResponse = components['schemas']['GraphResponse'];
export type GraphNodeDto = components['schemas']['GraphNodeDto'];
export type GraphEdgeDto = components['schemas']['GraphEdgeDto'];

/**
 * Format node label based on state and id.
 */
export function formatNodeLabel(node: GraphNodeDto): string {
  return `${node.state}_Node_${node.id}`;
}

/**
 * Map API GraphResponse into frontend EcmaNodeData[] and GraphEdgeData[].
 * Preserves x/y/vx/vy/particles from previous nodes/edges when available.
 */
export function mapGraphResponse(
  data: GraphResponse,
  prevNodes?: EcmaNodeData[],
  prevEdges?: GraphEdgeData[]
): { nodes: EcmaNodeData[]; edges: GraphEdgeData[] } {
  const prevNodeMap = new Map<number, EcmaNodeData>();
  if (prevNodes) {
    for (const n of prevNodes) {
      prevNodeMap.set(n.id, n);
    }
  }

  const prevEdgeMap = new Map<string, GraphEdgeData>();
  if (prevEdges) {
    for (const e of prevEdges) {
      prevEdgeMap.set(`${e.from}_${e.to}`, e);
      prevEdgeMap.set(`${e.to}_${e.from}`, e);
    }
  }

  const nodeCount = data.nodes.length;
  const nodes: EcmaNodeData[] = data.nodes.map((dto, index) => {
    const existing = prevNodeMap.get(dto.id);
    const state = dto.state as NodeState;

    let x = existing?.x;
    let y = existing?.y;

    if (x === undefined || y === undefined) {
      const angle = (index / Math.max(nodeCount, 1)) * 2 * Math.PI;
      const ringRadius = 200 + (index % 4) * 60;
      x = ringRadius * Math.cos(angle);
      y = ringRadius * Math.sin(angle);
    }

    const radius =
      state === 'Specialized'
        ? 14
        : state === 'Progenitor'
        ? 11
        : state === 'Stem'
        ? 9
        : 8;

    return {
      id: dto.id,
      label: existing?.label || formatNodeLabel(dto),
      state,
      usage_count: dto.usage_count,
      entropy: parseFloat(dto.entropy.toFixed(3)),
      connections: Number(dto.connections),
      maturity: parseFloat(dto.maturity.toFixed(2)),
      x,
      y,
      vx: existing?.vx ?? 0,
      vy: existing?.vy ?? 0,
      radius,
      isPathogen: existing?.isPathogen ?? false,
      highlighted: existing?.highlighted ?? false,
    };
  });

  const pheromoneMap = data.pheromone || {};

  const edges: GraphEdgeData[] = data.edges.map(dto => {
    const key = `${dto.from}_${dto.to}`;
    const existing = prevEdgeMap.get(key);

    const pheromone =
      pheromoneMap[key] ??
      pheromoneMap[`${dto.to}_${dto.from}`] ??
      existing?.pheromone ??
      1.0;

    const isHighway = pheromone >= 3.0;

    const particles =
      existing?.particles && existing.particles.length > 0
        ? existing.particles
        : Array.from({ length: isHighway ? 3 : 1 }, () => ({
            progress: Math.random(),
            speed: 0.005 + Math.random() * 0.01,
          }));

    return {
      from: dto.from,
      to: dto.to,
      weight: parseFloat(dto.weight.toFixed(2)),
      pheromone: parseFloat(pheromone.toFixed(2)),
      particles,
    };
  });

  return { nodes, edges };
}
