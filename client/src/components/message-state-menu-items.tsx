import { Menu as ArkMenu } from '@ark-ui/solid/menu';
import { Mail, MailOpen, Star, StarOff, Trash2, Undo2 } from 'lucide-solid';

const itemClass =
  'flex items-center gap-2 px-3 py-2 text-sm text-zinc-700 transition-colors data-[highlighted]:bg-zinc-200 data-[highlighted]:text-zinc-900 dark:text-zinc-300 dark:data-[highlighted]:bg-zinc-800 dark:data-[highlighted]:text-zinc-100';

export function MessageStateMenuItems(props: {
  isRead?: boolean;
  isStarred?: boolean;
  showRead: boolean;
  showStarred: boolean;
  showTrash: boolean;
  showRestore: boolean;
  // eslint-disable-next-line no-unused-vars
  onSelect: (value: string) => void;
}) {
  return (
    <>
      {props.showRead && (
        <ArkMenu.Item
          value="toggle-read"
          class={itemClass}
          onSelect={() => props.onSelect('toggle-read')}
        >
          {props.isRead ? <Mail size={16} /> : <MailOpen size={16} />}
          {props.isRead ? '标为未读' : '标为已读'}
        </ArkMenu.Item>
      )}
      {props.showStarred && (
        <ArkMenu.Item
          value="toggle-star"
          class={itemClass}
          onSelect={() => props.onSelect('toggle-star')}
        >
          {props.isStarred ? <StarOff size={16} /> : <Star size={16} />}
          {props.isStarred ? '取消星标' : '添加星标'}
        </ArkMenu.Item>
      )}
      {props.showRestore && (
        <ArkMenu.Item value="restore" class={itemClass} onSelect={() => props.onSelect('restore')}>
          <Undo2 size={16} />
          恢复
        </ArkMenu.Item>
      )}
      {props.showTrash && (
        <ArkMenu.Item value="trash" class={itemClass} onSelect={() => props.onSelect('trash')}>
          <Trash2 size={16} />
          移到垃圾箱
        </ArkMenu.Item>
      )}
    </>
  );
}
