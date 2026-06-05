import { createEffect, createSignal, onCleanup } from 'solid-js';
import { PwaUpdateModal } from '../../components/pwa-update-modal';
import { useRegisterSW } from './pwa-register';

const SERVICE_WORKER_UPDATE_INTERVAL_MS = 60_000;

export function PwaUpdateController() {
  const [dismissed, setDismissed] = createSignal(false);
  let intervalId: ReturnType<typeof setInterval> | undefined;

  const {
    needRefresh: [needRefresh],
    updateServiceWorker,
  } = useRegisterSW({
    onRegistered(registration) {
      if (registration === undefined) {
        return;
      }

      intervalId = setInterval(() => {
        void registration.update();
      }, SERVICE_WORKER_UPDATE_INTERVAL_MS);
    },
  });

  createEffect(() => {
    if (!needRefresh()) {
      setDismissed(false);
    }
  });

  onCleanup(() => {
    if (intervalId !== undefined) {
      clearInterval(intervalId);
    }
  });

  return (
    <PwaUpdateModal
      open={needRefresh() && !dismissed()}
      onLater={() => setDismissed(true)}
      onUpdate={() => void updateServiceWorker(true)}
    />
  );
}
