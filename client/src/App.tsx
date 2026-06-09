import { Route } from '@solidjs/router';
import { AppShell } from './app/app-shell';
import { appRoutes } from './app/routes';
import { ArchiveRoute } from './app/screens/archive-route';
import { InboxRoute } from './app/screens/inbox-route';
import { MessageDetailRoute } from './app/screens/message-detail-route';
import { NotFoundRoute } from './app/screens/not-found-route';
import { TaggedInboxRoute } from './app/screens/tagged-inbox-route';

export default function App() {
  return (
    <Route path={appRoutes.shell} component={AppShell}>
      <Route path={appRoutes.inbox} component={InboxRoute} />
      <Route path={appRoutes.archive} component={ArchiveRoute} />
      <Route path={appRoutes.messageDetail} component={MessageDetailRoute} />
      <Route path={appRoutes.tagInbox} component={TaggedInboxRoute} />
      <Route path={appRoutes.notFound} component={NotFoundRoute} />
    </Route>
  );
}
