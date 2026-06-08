import { Menu as ArkMenu } from '@ark-ui/solid/menu';
import { Portal } from 'solid-js/web';
import { Ellipsis, FileDown, FileText } from 'lucide-solid';
import { rawMessageDownloadUrl } from '../features/messages/api';

export type MessageActionMenuProps = {
  messageId: string;
  onViewHeaders?: () => void;
};

function downloadEml(messageId: string) {
  const a = document.createElement('a');
  a.href = rawMessageDownloadUrl(messageId);
  a.download = '';
  a.click();
}

export function MessageActionMenu(props: MessageActionMenuProps) {
  return (
    <ArkMenu.Root>
      <ArkMenu.Trigger
        class="rounded-sm p-1.5 text-zinc-500 transition-colors hover:bg-zinc-100 hover:text-zinc-900 dark:text-zinc-400 dark:hover:bg-zinc-800 dark:hover:text-zinc-100"
        aria-label="更多操作"
      >
        <Ellipsis size={18} />
      </ArkMenu.Trigger>
      <Portal>
        <ArkMenu.Positioner class="z-50">
          <ArkMenu.Content class="min-w-[160px] rounded-sm border border-zinc-200 bg-zinc-50 py-1 shadow-lg dark:border-zinc-800 dark:bg-zinc-950">
            {props.onViewHeaders && (
              <ArkMenu.Item
                value="view-headers"
                class="flex items-center gap-2 px-3 py-2 text-sm text-zinc-700 transition-colors data-[highlighted]:bg-zinc-200 data-[highlighted]:text-zinc-900 dark:text-zinc-300 dark:data-[highlighted]:bg-zinc-800 dark:data-[highlighted]:text-zinc-100"
                onSelect={() => props.onViewHeaders?.()}
              >
                <FileText size={16} />
                查看原始字段
              </ArkMenu.Item>
            )}
            <ArkMenu.Item
              value="download-raw"
              class="flex items-center gap-2 px-3 py-2 text-sm text-zinc-700 transition-colors data-[highlighted]:bg-zinc-200 data-[highlighted]:text-zinc-900 dark:text-zinc-300 dark:data-[highlighted]:bg-zinc-800 dark:data-[highlighted]:text-zinc-100"
              onSelect={() => downloadEml(props.messageId)}
            >
              <FileDown size={16} />
              下载 EML 源文件
            </ArkMenu.Item>
          </ArkMenu.Content>
        </ArkMenu.Positioner>
      </Portal>
    </ArkMenu.Root>
  );
}
