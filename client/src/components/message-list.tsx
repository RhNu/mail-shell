import { type JSX, For } from 'solid-js';
import type { Mailbox, MessageSummary } from '../features/messages/models';
import { MessageListItem } from './message-list-item';

export type MessageListProps = {
  messages: MessageSummary[];
  attachmentCounts: Map<string, number>;
  activeMessageId?: string;
  returnTo: string;
  // eslint-disable-next-line no-unused-vars
  onMoveToMailbox?: (_id: string, _mailbox: Mailbox) => void;
  // eslint-disable-next-line no-unused-vars
  onDelete?: (_id: string) => void;
  onUpdateState?: (
    // eslint-disable-next-line no-unused-vars
    _id: string,
    // eslint-disable-next-line no-unused-vars
    _state: { read?: boolean; starred?: boolean; trashed?: boolean },
  ) => void;
  trashView?: boolean;
  selectionMode?: boolean;
  selectedIds?: Set<string>;
  // eslint-disable-next-line no-unused-vars
  onSelectedChange?: (_id: string, _selected: boolean) => void;
  actionsDisabled?: boolean;
};

export function MessageList(props: MessageListProps): JSX.Element {
  const hasMessages = () => props.messages.length > 0;

  return (
    <div
      class={[
        'border-t border-zinc-200 dark:border-zinc-800',
        hasMessages() ? 'animate-fade-in' : '',
      ].join(' ')}
    >
      <For each={props.messages}>
        {(message) => (
          <MessageListItem
            message={message}
            tags={message.labels}
            attachmentCount={props.attachmentCounts.get(message.id) ?? message.attachment_count}
            active={message.id === props.activeMessageId}
            returnTo={props.returnTo}
            onMoveToMailbox={props.onMoveToMailbox}
            onDelete={props.onDelete}
            onUpdateState={props.onUpdateState}
            trashView={props.trashView}
            selectionMode={props.selectionMode}
            selected={props.selectedIds?.has(message.id)}
            onSelectedChange={props.onSelectedChange}
            actionsDisabled={props.actionsDisabled}
          />
        )}
      </For>
    </div>
  );
}
