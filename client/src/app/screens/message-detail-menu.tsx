import { MessageActionMenu } from '../../components/message-action-menu';
import type { Mailbox } from '../../features/messages/models';

type MessageDetailMenuProps = {
  id: string;
  mailbox: Mailbox;
  isRead: boolean;
  isStarred: boolean;
  trashed: boolean;
  disabled: boolean;
  onViewHeaders: () => void;
  // eslint-disable-next-line no-unused-vars
  onMoveToMailbox: (mailbox: Mailbox) => void;
  onDelete?: () => void;
  // eslint-disable-next-line no-unused-vars
  onUpdateState: (state: { read?: boolean; starred?: boolean; trashed?: boolean }) => void;
};

export function MessageDetailMenu(props: MessageDetailMenuProps) {
  return (
    <MessageActionMenu
      messageId={props.id}
      mailbox={props.mailbox}
      onViewHeaders={props.onViewHeaders}
      onMoveToMailbox={props.onMoveToMailbox}
      onDelete={props.onDelete}
      onTrash={props.trashed ? undefined : () => props.onUpdateState({ trashed: true })}
      onRestore={props.trashed ? () => props.onUpdateState({ trashed: false }) : undefined}
      onSetRead={(read) => props.onUpdateState({ read })}
      onSetStarred={(starred) => props.onUpdateState({ starred })}
      isRead={props.isRead}
      isStarred={props.isStarred}
      disabled={props.disabled}
    />
  );
}

export { MessageLabelsEditor } from '../../components/classification/message-labels-editor';
