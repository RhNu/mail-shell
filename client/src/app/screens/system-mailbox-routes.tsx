import { Star, Trash2, MailOpen } from 'lucide-solid';
import { InboxScreen } from '../../components/inbox-screen';
import { useEmptyTrash } from '../../features/messages/queries';

export function UnreadRoute() {
  return (
    <InboxScreen
      title={
        <h1 class="flex items-center gap-2 text-xl font-semibold">
          <MailOpen size={20} />
          未读
        </h1>
      }
      query={() => ({ mailbox: 'inbox', read: false })}
      emptyDescription="没有未读邮件。"
    />
  );
}

export function StarredRoute() {
  return (
    <InboxScreen
      title={
        <h1 class="flex items-center gap-2 text-xl font-semibold">
          <Star size={20} />
          星标
        </h1>
      }
      query={() => ({ mailbox: 'inbox', starred: true })}
      emptyDescription="没有星标邮件。"
    />
  );
}

export function TrashRoute() {
  const emptyTrash = useEmptyTrash();
  return (
    <InboxScreen
      title={
        <div class="flex flex-wrap items-center gap-3">
          <h1 class="flex items-center gap-2 text-xl font-semibold">
            <Trash2 size={20} />
            垃圾箱
          </h1>
          <button
            type="button"
            disabled={emptyTrash.isPending}
            class="rounded-sm border border-red-300 px-2 py-1 text-xs text-red-700 hover:bg-red-50 disabled:opacity-50 dark:border-red-900 dark:text-red-400"
            onClick={() => {
              if (window.confirm('永久删除垃圾箱中的全部邮件？此操作无法撤销。'))
                emptyTrash.mutate();
            }}
          >
            清空垃圾箱
          </button>
        </div>
      }
      subtitle="邮件将在移入垃圾箱 30 天后永久删除。"
      query={() => ({ mailbox: 'inbox', trashed: true })}
      emptyDescription="垃圾箱为空。"
    />
  );
}
