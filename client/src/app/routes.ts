export const appRoutes = {
  shell: '/',
  inbox: '/',
  archive: '/archive',
  unread: '/unread',
  starred: '/starred',
  trash: '/trash',
  messageDetail: '/messages/:messageId',
  labelInbox: '/labels/:labelId',
  savedView: '/views/:viewId',
  facets: '/facets',
  classification: '/classification',
  notFound: '*',
} as const;

export function messageDetailHref(messageId: string, returnTo?: string): string {
  const path = `/messages/${messageId}`;
  if (!returnTo) return path;

  return `${path}?returnTo=${encodeURIComponent(returnTo)}`;
}

export function tagInboxHref(tagId: string | number): string {
  return `/labels/${tagId}`;
}
