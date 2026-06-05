import { apiGet } from './client';
import type { HealthResponse } from './types';

export function checkHealth(): Promise<HealthResponse> {
  return apiGet('/healthz');
}
