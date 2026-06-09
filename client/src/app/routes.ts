export const appRoutes = {
  shell: '/',
  inbox: '/',
  archive: '/archive',
  messageDetail: '/messages/:messageId',
  tagInbox: '/tags/:tagId',
  notFound: '*',
} as const;

export function messageDetailHref(messageId: string, returnTo?: string): string {
  const path = `/messages/${messageId}`;
  if (!returnTo) return path;

  return `${path}?returnTo=${encodeURIComponent(returnTo)}`;
}

export function tagInboxHref(tagId: string | number): string {
  return `/tags/${tagId}`;
}
