import { createSignal, For } from 'solid-js';
import { useClassificationMutations, useLabels } from '../../features/classification/queries';

export function LabelManager() {
  const labels = useLabels();
  const mutations = useClassificationMutations();
  const [name, setName] = createSignal('');
  const submit = (event: SubmitEvent) => {
    event.preventDefault();
    if (!name().trim()) return;
    mutations.createLabel.mutate({ name: name().trim() }, { onSuccess: () => setName('') });
  };
  return (
    <section class="flex flex-col gap-3 rounded-sm border border-zinc-200 p-4 dark:border-zinc-800">
      <h2 class="font-semibold">用户标签</h2>
      <form class="flex gap-2" onSubmit={submit}>
        <input
          class="min-w-0 flex-1 rounded-sm border border-zinc-300 bg-transparent px-3 py-2 text-sm dark:border-zinc-700"
          value={name()}
          onInput={(event) => setName(event.currentTarget.value)}
          placeholder="新标签名称"
        />
        <button
          type="submit"
          class="rounded-sm bg-zinc-900 px-3 py-2 text-sm text-white dark:bg-zinc-100 dark:text-zinc-900"
        >
          添加
        </button>
      </form>
      <div class="flex flex-col gap-1">
        <For each={labels.data ?? []}>
          {(label) => (
            <div class="flex items-center justify-between rounded-sm px-2 py-1.5 text-sm hover:bg-zinc-100 dark:hover:bg-zinc-900">
              <span>
                {label.name} <span class="text-zinc-400">{label.message_count ?? 0}</span>
              </span>
              <button
                type="button"
                class="text-xs text-red-600"
                onClick={() => mutations.deleteLabel.mutate(label.id)}
              >
                删除
              </button>
            </div>
          )}
        </For>
      </div>
    </section>
  );
}
