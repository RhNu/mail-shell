export const appRoutes = {
  shell: '/',
  inbox: '/',
  messageDetail: '/messages/:messageId',
  tagInbox: '/tags/:tagId',
  notFound: '*',
} as const;

export function messageDetailHref(messageId: string): string {
  return `/messages/${messageId}`;
}

export function tagInboxHref(tagId: string | number): string {
  return `/tags/${tagId}`;
}
