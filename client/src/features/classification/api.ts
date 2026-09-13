import { apiClient } from '../../api/core/client';
import { executeJson, executeVoid } from '../../api/core/response';
import type { components } from '../../api/generated/schema';

export type Label = components['schemas']['Label'];
export type MessageLabel = components['schemas']['MessageLabel'];
export type LabelWriteRequest = components['schemas']['LabelWriteRequest'];
export type ClassificationRule = components['schemas']['ClassificationRule'];
export type RuleWriteRequest = components['schemas']['RuleWriteRequest'];
export type SavedView = components['schemas']['SavedView'];
export type SavedViewWriteRequest = components['schemas']['SavedViewWriteRequest'];
export type FacetValue = components['schemas']['FacetValue'];

export const listLabels = (): Promise<Label[]> => executeJson(apiClient.GET('/api/labels'));
export const createLabel = (body: LabelWriteRequest): Promise<Label> =>
  executeJson(apiClient.POST('/api/labels', { body }));
export const deleteLabel = (id: number): Promise<void> =>
  executeVoid(apiClient.DELETE('/api/labels/{id}', { params: { path: { id } } }));
export const setMessageLabels = (id: string, labelIds: number[]): Promise<void> =>
  executeVoid(
    apiClient.PUT('/api/messages/{id}/labels', {
      params: { path: { id } },
      body: { label_ids: labelIds },
    }),
  );

export const listRules = (): Promise<ClassificationRule[]> =>
  executeJson(apiClient.GET('/api/rules'));
export const createRule = (body: RuleWriteRequest): Promise<ClassificationRule> =>
  executeJson(apiClient.POST('/api/rules', { body }));
export const deleteRule = (id: number): Promise<void> =>
  executeVoid(apiClient.DELETE('/api/rules/{id}', { params: { path: { id } } }));

export const listSavedViews = (): Promise<SavedView[]> =>
  executeJson(apiClient.GET('/api/saved-views'));
export const createSavedView = (body: SavedViewWriteRequest): Promise<SavedView> =>
  executeJson(apiClient.POST('/api/saved-views', { body }));
export const deleteSavedView = (id: number): Promise<void> =>
  executeVoid(apiClient.DELETE('/api/saved-views/{id}', { params: { path: { id } } }));

export const listFacets = (kind: string): Promise<FacetValue[]> =>
  executeJson(apiClient.GET('/api/facets', { params: { query: { kind } } }));
