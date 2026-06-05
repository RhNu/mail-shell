import type { components } from '../../api/generated/schema';
import { apiClient } from '../../api/core/client';
import { executeJson } from '../../api/core/response';

export type Tag = components['schemas']['Tag'];

export function listTags(): Promise<Tag[]> {
  return executeJson(apiClient.GET('/api/tags'));
}
