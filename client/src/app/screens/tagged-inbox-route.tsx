import { createMemo } from 'solid-js';
import { useParams } from '@solidjs/router';
import { useTagsList } from '../../features/tags/queries';
import { InboxScreen } from '../../components/inbox-screen';
import { TagChip } from '../../components/ui/tag-chip';

export function TaggedInboxRoute() {
  const params = useParams<{ tagId: string }>();
  const tagId = createMemo(() => Number(params.tagId));
  const tagsQuery = useTagsList();
  const tag = createMemo(() => tagsQuery.data?.find((t) => t.id === tagId()));

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
            {tag()?.label ?? '已标签'}
          </h1>
        </div>
      }
      query={() => ({ tag: tagId(), mailbox: 'inbox' })}
      tagChip={tag() ? <TagChip label={tag()!.label} active /> : undefined}
      emptyDescription="没有符合此标签的邮件。"
    />
  );
}
