import { createMemo } from 'solid-js';
import { useParams } from '@solidjs/router';
import { useLabels } from '../../features/classification/queries';
import { InboxScreen } from '../../components/inbox-screen';
import { TagChip } from '../../components/ui/tag-chip';

export function LabelInboxRoute() {
  const params = useParams<{ labelId: string }>();
  const labelId = createMemo(() => Number(params.labelId));
  const labelsQuery = useLabels();
  const label = createMemo(() => labelsQuery.data?.find((item) => item.id === labelId()));

  return (
    <InboxScreen
      title={
        <div class="flex items-center gap-2">
          <a
            href="#/"
            class="text-sm text-zinc-500 transition-colors hover:text-zinc-900 dark:text-zinc-400 dark:hover:text-zinc-100"
          >
            收件箱
          </a>
          <span class="text-sm text-zinc-400 dark:text-zinc-500">/</span>
          <h1 class="text-xl font-semibold text-zinc-900 dark:text-zinc-100">
            {label()?.name ?? '标签'}
          </h1>
        </div>
      }
      query={() => ({ label: labelId(), mailbox: 'inbox' })}
      tagChip={label() ? <TagChip label={label()!.name} active /> : undefined}
      emptyDescription="没有符合此标签的邮件。"
    />
  );
}
