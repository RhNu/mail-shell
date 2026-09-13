import { createMutation, createQuery, useQueryClient } from '@tanstack/solid-query';
import {
  createLabel,
  createRule,
  createSavedView,
  deleteLabel,
  deleteRule,
  deleteSavedView,
  listFacets,
  listLabels,
  listRules,
  listSavedViews,
  setMessageLabels,
  type LabelWriteRequest,
  type RuleWriteRequest,
  type SavedViewWriteRequest,
} from './api';

const invalidateClassification = (client: ReturnType<typeof useQueryClient>) =>
  Promise.all([
    client.invalidateQueries({ queryKey: ['classification'] }),
    client.invalidateQueries({ queryKey: ['messages'] }),
  ]);

export const useLabels = () =>
  createQuery(() => ({ queryKey: ['classification', 'labels'], queryFn: listLabels }));
export const useRules = () =>
  createQuery(() => ({ queryKey: ['classification', 'rules'], queryFn: listRules }));
export const useSavedViews = () =>
  createQuery(() => ({ queryKey: ['classification', 'views'], queryFn: listSavedViews }));
export const useFacets = (kind: () => string) =>
  createQuery(() => ({
    queryKey: ['classification', 'facets', kind()],
    queryFn: () => listFacets(kind()),
  }));

export function useClassificationMutations() {
  const client = useQueryClient();
  return {
    createLabel: createMutation(() => ({
      mutationFn: (request: LabelWriteRequest) => createLabel(request),
      onSuccess: () => invalidateClassification(client),
    })),
    deleteLabel: createMutation(() => ({
      mutationFn: deleteLabel,
      onSuccess: () => invalidateClassification(client),
    })),
    createRule: createMutation(() => ({
      mutationFn: (request: RuleWriteRequest) => createRule(request),
      onSuccess: () => invalidateClassification(client),
    })),
    deleteRule: createMutation(() => ({
      mutationFn: deleteRule,
      onSuccess: () => invalidateClassification(client),
    })),
    createView: createMutation(() => ({
      mutationFn: (request: SavedViewWriteRequest) => createSavedView(request),
      onSuccess: () => invalidateClassification(client),
    })),
    deleteView: createMutation(() => ({
      mutationFn: deleteSavedView,
      onSuccess: () => invalidateClassification(client),
    })),
    setMessageLabels: createMutation(() => ({
      mutationFn: ({ id, labelIds }: { id: string; labelIds: number[] }) =>
        setMessageLabels(id, labelIds),
      onSuccess: () => invalidateClassification(client),
    })),
  };
}
