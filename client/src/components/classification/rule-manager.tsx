import { createSignal, For } from 'solid-js';
import {
  useClassificationMutations,
  useLabels,
  useRules,
} from '../../features/classification/queries';

// oxlint-disable-next-line max-lines-per-function
export function RuleManager() {
  const rules = useRules();
  const labels = useLabels();
  const mutations = useClassificationMutations();
  const [name, setName] = createSignal('');
  const [field, setField] = createSignal('envelope_to');
  const [value, setValue] = createSignal('');
  const [labelId, setLabelId] = createSignal('');
  const submit = (event: SubmitEvent) => {
    event.preventDefault();
    if (!name().trim() || !value().trim()) return;
    mutations.createRule.mutate(
      {
        name: name().trim(),
        enabled: true,
        priority: 0,
        stop_processing: false,
        conditions: [{ field: field(), operator: 'contains', value: value().trim() }],
        actions: { add_label_ids: labelId() ? [Number(labelId())] : [] },
      },
      {
        onSuccess: () => {
          setName('');
          setValue('');
        },
      },
    );
  };
  return (
    <section class="flex flex-col gap-3 rounded-sm border border-zinc-200 p-4 dark:border-zinc-800">
      <div>
        <h2 class="font-semibold">收件规则</h2>
        <p class="text-xs text-zinc-500">规则按优先级执行，不会自动拆解加号地址。</p>
      </div>
      <form class="grid gap-2 sm:grid-cols-2" onSubmit={submit}>
        <input
          class="rounded-sm border p-2 text-sm dark:bg-zinc-950"
          value={name()}
          onInput={(event) => setName(event.currentTarget.value)}
          placeholder="规则名称"
        />
        <select
          class="rounded-sm border p-2 text-sm dark:bg-zinc-950"
          value={field()}
          onChange={(event) => setField(event.currentTarget.value)}
        >
          <option value="envelope_to">收件地址</option>
          <option value="from">发件人</option>
          <option value="subject">主题</option>
        </select>
        <input
          class="rounded-sm border p-2 text-sm dark:bg-zinc-950"
          value={value()}
          onInput={(event) => setValue(event.currentTarget.value)}
          placeholder="包含文本"
        />
        <select
          class="rounded-sm border p-2 text-sm dark:bg-zinc-950"
          value={labelId()}
          onChange={(event) => setLabelId(event.currentTarget.value)}
        >
          <option value="">不添加标签</option>
          <For each={labels.data ?? []}>
            {(label) => <option value={label.id}>{label.name}</option>}
          </For>
        </select>
        <button
          type="submit"
          class="rounded-sm bg-zinc-900 px-3 py-2 text-sm text-white dark:bg-zinc-100 dark:text-zinc-900"
        >
          添加规则
        </button>
      </form>
      <For each={rules.data ?? []}>
        {(rule) => (
          <div class="flex items-center justify-between text-sm">
            <span>
              {rule.name} · {rule.conditions[0]?.field} 包含 {rule.conditions[0]?.value}
            </span>
            <button
              type="button"
              class="text-xs text-red-600"
              onClick={() => mutations.deleteRule.mutate(rule.id)}
            >
              删除
            </button>
          </div>
        )}
      </For>
    </section>
  );
}
