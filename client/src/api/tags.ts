import { apiGet } from './client';
import type { Tag } from './types';

export function listTags(): Promise<Tag[]> {
  return apiGet('/tags');
}
