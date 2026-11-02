import { describe, it, expect } from 'vitest';
import { mapGraphResponse, GraphResponse } from './mapping';

describe('mapping', () => {
  const sampleResponse: GraphResponse = {
    nodes: [
      {
        id: 10,
        state: 'Stem',
        usage_count: 5,
        entropy: 0.8,
        connections: 1,
        maturity: 0.1,
      },
      {
        id: 20,
        state: 'Specialized',
        usage_count: 200,
        entropy: 0.05,
        connections: 1,
        maturity: 0.95,
      },
    ],
    edges: [
      {
        from: 10,
        to: 20,
        weight: 0.75,
      },
    ],
    pheromone: {
      '10_20': 4.5,
      '20_10': 4.5,
    },
  };

  it('maps nodes and edges with circular layout when no previous coordinates exist', () => {
    const { nodes, edges } = mapGraphResponse(sampleResponse);

    expect(nodes).toHaveLength(2);
    expect(edges).toHaveLength(1);

    expect(nodes[0].id).toBe(10);
    expect(nodes[0].state).toBe('Stem');
    expect(nodes[0].radius).toBe(9);
    expect(typeof nodes[0].x).toBe('number');
    expect(typeof nodes[0].y).toBe('number');

    expect(nodes[1].id).toBe(20);
    expect(nodes[1].state).toBe('Specialized');
    expect(nodes[1].radius).toBe(14);

    expect(edges[0].from).toBe(10);
    expect(edges[0].to).toBe(20);
    expect(edges[0].weight).toBe(0.75);
    expect(edges[0].pheromone).toBe(4.5);
    expect(edges[0].particles).toBeDefined();
  });

  it('preserves existing x, y coordinates and particles', () => {
    const prevNodes = [
      {
        id: 10,
        state: 'Stem' as const,
        usage_count: 1,
        entropy: 0.9,
        connections: 1,
        maturity: 0.0,
        x: 123.45,
        y: -67.89,
      },
    ];

    const prevEdges = [
      {
        from: 10,
        to: 20,
        weight: 0.7,
        pheromone: 3.0,
        particles: [{ progress: 0.42, speed: 0.01 }],
      },
    ];

    const { nodes, edges } = mapGraphResponse(sampleResponse, prevNodes, prevEdges);

    expect(nodes[0].x).toBe(123.45);
    expect(nodes[0].y).toBe(-67.89);
    expect(edges[0].particles?.[0].progress).toBe(0.42);
  });
});
