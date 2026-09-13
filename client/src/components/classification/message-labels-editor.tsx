import { createMemo, For } from 'solid-js';
import { useClassificationMutations, useLabels } from '../../features/classification/queries';

export function MessageLabelsEditor(props: { messageId: string; labelIds: number[] }) {
  const labels = useLabels();
  const mutations = useClassificationMutations();
  const selected = createMemo(() => new Set(props.labelIds));
  const toggle = (id: number, checked: boolean) => {
    const next = new Set(selected());
    if (checked) next.add(id);
    else next.delete(id);
    mutations.setMessageLabels.mutate({ id: props.messageId, labelIds: [...next] });
  };
  return (
    <div class="flex flex-wrap items-center gap-2 text-xs">
      <span class="text-zinc-500">标签：</span>
      <For each={labels.data ?? []}>
        {(label) => (
          <label class="flex cursor-pointer items-center gap-1 rounded-full border border-zinc-300 px-2 py-1 dark:border-zinc-700">
            <input
              type="checkbox"
              checked={selected().has(label.id)}
              onChange={(event) => toggle(label.id, event.currentTarget.checked)}
            />
            {label.name}
          </label>
        )}
      </For>
    </div>
  );
}
