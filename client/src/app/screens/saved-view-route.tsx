import { createMemo } from 'solid-js';
import { useParams, useSearchParams } from '@solidjs/router';
import { InboxScreen } from '../../components/inbox-screen';
import { useSavedViews } from '../../features/classification/queries';
import type { MessageListQuery } from '../../features/messages/models';

export function SavedViewRoute() {
  const params = useParams<{ viewId: string }>();
  const views = useSavedViews();
  const view = createMemo(() => views.data?.find((item) => item.id === Number(params.viewId)));
  return (
    <InboxScreen
      title={<h1 class="text-xl font-semibold">{view()?.name ?? '智能视图'}</h1>}
      query={() => (view()?.query as MessageListQuery | undefined) ?? { mailbox: 'inbox' }}
      emptyDescription="没有符合此视图的邮件。"
    />
  );
}

export function SearchRoute() {
  const [params] = useSearchParams<{ q?: string }>();
  return (
    <InboxScreen
      title={<h1 class="text-xl font-semibold">搜索</h1>}
      subtitle={params.q ? `“${params.q}”` : undefined}
      query={() => ({ mailbox: 'inbox', q: params.q })}
      emptyDescription="没有匹配的邮件。"
    />
  );
}
