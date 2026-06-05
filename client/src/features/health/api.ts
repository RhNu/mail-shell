import type { components } from '../../api/generated/schema';
import { apiClient } from '../../api/core/client';
import { executeJson } from '../../api/core/response';

export type HealthResponse = components['schemas']['HealthResponse'];

export function getHealth(): Promise<HealthResponse> {
  return executeJson(apiClient.GET('/api/healthz'));
}
