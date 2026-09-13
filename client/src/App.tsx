import { Route } from '@solidjs/router';
import { AppShell } from './app/app-shell';
import { appRoutes } from './app/routes';
import {
  ArchiveRoute,
  ClassificationRoute,
  FacetsRoute,
  InboxRoute,
  LabelInboxRoute,
  MessageDetailRoute,
  NotFoundRoute,
  SavedViewRoute,
  SearchRoute,
  StarredRoute,
  TrashRoute,
  UnreadRoute,
} from './app/screens';

export default function App() {
  return (
    <Route path={appRoutes.shell} component={AppShell}>
      <Route path={appRoutes.inbox} component={InboxRoute} />
      <Route path={appRoutes.archive} component={ArchiveRoute} />
      <Route path={appRoutes.unread} component={UnreadRoute} />
      <Route path={appRoutes.starred} component={StarredRoute} />
      <Route path={appRoutes.trash} component={TrashRoute} />
      <Route path={appRoutes.messageDetail} component={MessageDetailRoute} />
      <Route path={appRoutes.labelInbox} component={LabelInboxRoute} />
      <Route path={appRoutes.savedView} component={SavedViewRoute} />
      <Route path={appRoutes.facets} component={FacetsRoute} />
      <Route path={appRoutes.classification} component={ClassificationRoute} />
      <Route path="/search" component={SearchRoute} />
      <Route path={appRoutes.notFound} component={NotFoundRoute} />
    </Route>
  );
}
