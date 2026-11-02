import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiFetch } from './client';
import { components } from './schema';
import { GraphResponse } from './mapping';

export type QueryBody = components['schemas']['QueryBody'];
export type QueryResponseBody = components['schemas']['QueryResponseBody'];
export type CreateNodeRequest = components['schemas']['CreateNodeRequest'];
export type CreateNodeResponse = components['schemas']['CreateNodeResponse'];
export type SetNodeStateRequest = components['schemas']['SetNodeStateRequest'];
export type DepositRequest = components['schemas']['DepositRequest'];
export type DepositResponse = components['schemas']['DepositResponse'];
export type AisProbeRequest = components['schemas']['AisProbeRequest'];
export type AisProbeResponse = components['schemas']['AisProbeResponse'];

export interface HealthReport {
  status: 'Healthy' | 'Degraded' | 'Unhealthy';
  wal_ok: boolean;
  db_ok: boolean;
  latency_ok: boolean;
  version: string;
  uptime_seconds: number;
}

export const QUERY_KEYS = {
  health: ['health'] as const,
  graph: ['graph'] as const,
};

/**
 * Health check hook with 5s polling.
 */
export function useHealthQuery() {
  return useQuery<HealthReport>({
    queryKey: QUERY_KEYS.health,
    queryFn: () => apiFetch<HealthReport>('/health'),
    refetchInterval: 5000,
    retry: 1,
  });
}

/**
 * Live graph topology hook.
 */
export function useGraphQuery(enabled: boolean = true) {
  return useQuery<GraphResponse>({
    queryKey: QUERY_KEYS.graph,
    queryFn: () => apiFetch<GraphResponse>('/graph'),
    enabled,
    staleTime: 2000,
  });
}

/**
 * Stimulate Query mutation (POST /query).
 */
export function useQueryMutation() {
  const queryClient = useQueryClient();
  return useMutation<QueryResponseBody, Error, QueryBody>({
    mutationFn: body =>
      apiFetch<QueryResponseBody>('/query', {
        method: 'POST',
        body: JSON.stringify(body),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.graph });
    },
  });
}

/**
 * Mitosis / Node creation mutation (POST /nodes).
 */
export function useCreateNodeMutation() {
  const queryClient = useQueryClient();
  return useMutation<CreateNodeResponse, Error, CreateNodeRequest>({
    mutationFn: body =>
      apiFetch<CreateNodeResponse>('/nodes', {
        method: 'POST',
        body: JSON.stringify(body),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.graph });
    },
  });
}

/**
 * Node deletion mutation (DELETE /nodes/:id).
 */
export function useDeleteNodeMutation() {
  const queryClient = useQueryClient();
  return useMutation<void, Error, number>({
    mutationFn: id =>
      apiFetch<void>(`/nodes/${id}`, {
        method: 'DELETE',
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.graph });
    },
  });
}

/**
 * Node state mutation (POST /nodes/:id/state).
 */
export function useSetNodeStateMutation() {
  const queryClient = useQueryClient();
  return useMutation<void, Error, { id: number; state: SetNodeStateRequest['state'] }>({
    mutationFn: ({ id, state }) =>
      apiFetch<void>(`/nodes/${id}/state`, {
        method: 'POST',
        body: JSON.stringify({ state }),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.graph });
    },
  });
}

/**
 * Pheromone deposit mutation (POST /pheromone/deposit).
 */
export function useDepositMutation() {
  const queryClient = useQueryClient();
  return useMutation<DepositResponse, Error, DepositRequest>({
    mutationFn: body =>
      apiFetch<DepositResponse>('/pheromone/deposit', {
        method: 'POST',
        body: JSON.stringify(body),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.graph });
    },
  });
}

/**
 * AIS anomaly probe mutation (POST /ais/probe).
 */
export function useAisProbeMutation() {
  return useMutation<AisProbeResponse, Error, AisProbeRequest>({
    mutationFn: body =>
      apiFetch<AisProbeResponse>('/ais/probe', {
        method: 'POST',
        body: JSON.stringify(body),
      }),
  });
}
