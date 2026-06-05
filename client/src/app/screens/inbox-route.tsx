import { InboxScreen } from '../../components/inbox-screen';

export function InboxRoute() {
  return (
    <InboxScreen
      title={<h1 class="text-xl font-semibold text-zinc-900 dark:text-zinc-100">收件箱</h1>}
      query={() => ({})}
    />
  );
}
