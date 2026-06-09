import { Archive } from 'lucide-solid';
import { InboxScreen } from '../../components/inbox-screen';

export function ArchiveRoute() {
  return (
    <InboxScreen
      title={
        <h1 class="flex items-center gap-2 text-xl font-semibold text-zinc-900 dark:text-zinc-100">
          <Archive size={20} aria-hidden="true" />
          归档
        </h1>
      }
      query={() => ({ mailbox: 'archive' })}
      emptyDescription="归档区暂无邮件。"
    />
  );
}
