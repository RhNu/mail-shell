import { Show } from 'solid-js';
import { RawHeadersDialog } from '../../components/raw-headers-dialog';
import { useMessageDetail } from '../../features/messages/queries';

export function MessageHeadersDialogMount(props: {
  query: ReturnType<typeof useMessageDetail>;
  open: boolean;
  onClose: () => void;
}) {
  return (
    <Show when={props.query.data}>
      <RawHeadersDialog
        messageId={props.query.data!.id}
        open={props.open}
        onClose={props.onClose}
      />
    </Show>
  );
}
