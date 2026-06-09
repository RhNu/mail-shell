import { createMemo, type JSX, For } from 'solid-js';
import type { Mailbox, MessageSummary } from '../features/messages/models';
import type { Tag } from '../features/tags/api';
import { MessageListItem } from './message-list-item';

export type MessageListProps = {
  messages: MessageSummary[];
  tagsMap: Map<string, Tag[]>;
  attachmentCounts: Map<string, number>;
  activeMessageId?: string;
  searchQuery: string;
  returnTo: string;
  // eslint-disable-next-line no-unused-vars
  onMoveToMailbox?: (_id: string, _mailbox: Mailbox) => void;
  // eslint-disable-next-line no-unused-vars
  onDelete?: (_id: string) => void;
  actionsDisabled?: boolean;
};

export function MessageList(props: MessageListProps): JSX.Element {
  const filteredMessages = createMemo(() => {
    const query = props.searchQuery.trim().toLowerCase();
    if (!query) return props.messages;

    return props.messages.filter((m) => {
      const from = m.from_address.toLowerCase();
      const subject = (m.subject ?? '').toLowerCase();
      const tags = (props.tagsMap.get(m.id) ?? []).map((t) => t.label.toLowerCase()).join(' ');
      return from.includes(query) || subject.includes(query) || tags.includes(query);
    });
  });

  return (
    <div class="border-t border-zinc-200 dark:border-zinc-800">
      <For each={filteredMessages()}>
        {(message) => (
          <MessageListItem
            message={message}
            tags={props.tagsMap.get(message.id) ?? []}
            attachmentCount={props.attachmentCounts.get(message.id)}
            active={message.id === props.activeMessageId}
            returnTo={props.returnTo}
            onMoveToMailbox={props.onMoveToMailbox}
            onDelete={props.onDelete}
            actionsDisabled={props.actionsDisabled}
          />
        )}
      </For>
    </div>
  );
}
