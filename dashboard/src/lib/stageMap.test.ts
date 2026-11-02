import { describe, it, expect } from 'vitest';
import { mapApiStagesToMetrics, STAGE_DEFINITIONS } from './stageMap';

describe('stageMap', () => {
  it('returns exactly 10 stages in sequential order with defaults', () => {
    const metrics = mapApiStagesToMetrics();

    expect(metrics).toHaveLength(10);
    expect(metrics.map(m => m.stageNum)).toEqual([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

    for (let i = 0; i < metrics.length; i++) {
      expect(metrics[i].name).toBe(STAGE_DEFINITIONS[i].name);
      expect(metrics[i].durationMs).toBe(STAGE_DEFINITIONS[i].defaultDurationMs);
    }
  });

  it('maps custom stage durations from API response', () => {
    const apiStages = {
      validate: 0.05,
      rate_limit: 0.02,
      plan: 0.1,
      retrieve: 2.345,
      expand: 0.88,
      ecma_update: 0.42,
      mce_encode: 0.31,
      execute: 0.55,
      persist: 0.19,
      metrics: 0.09,
    };

    const metrics = mapApiStagesToMetrics(apiStages);

    expect(metrics).toHaveLength(10);
    const retrieve = metrics.find(m => m.name === 'SIMD Vector Retrieval');
    expect(retrieve?.durationMs).toBe(2.345);

    const execute = metrics.find(m => m.name === 'Action Execution');
    expect(execute?.durationMs).toBe(0.55);
  });
});
