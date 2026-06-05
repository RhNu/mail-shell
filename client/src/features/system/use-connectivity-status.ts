import { createSignal, onCleanup, type Setter } from 'solid-js';
import { getHealth } from '../health/api';

export type ConnectivityStatus = 'online' | 'offline' | 'service-unreachable';

type EventTargetLike = Pick<Window, 'addEventListener' | 'removeEventListener'>;

type ConnectivityStatusOptions = {
  checkHealth?: () => Promise<unknown>;
  eventTarget?: EventTargetLike;
  onlineSource?: () => boolean;
  healthyPollIntervalMs?: number;
  retryPollIntervalMs?: number;
  failureThreshold?: number;
};

const DEFAULT_HEALTHY_POLL_INTERVAL_MS = 30_000;
const DEFAULT_RETRY_POLL_INTERVAL_MS = 10_000;
const DEFAULT_FAILURE_THRESHOLD = 2;

type ResolvedConnectivityStatusOptions = {
  checkHealth: () => Promise<unknown>;
  eventTarget?: EventTargetLike;
  onlineSource: () => boolean;
  healthyPollIntervalMs: number;
  retryPollIntervalMs: number;
  failureThreshold: number;
};

type ConnectivityRuntimeState = {
  timerId: ReturnType<typeof setTimeout> | undefined;
  consecutiveFailures: number;
  runId: number;
  disposed: boolean;
};

function getDefaultEventTarget(): EventTargetLike | undefined {
  return typeof window === 'undefined' ? undefined : window;
}

function getDefaultOnlineSource(): () => boolean {
  return typeof navigator === 'undefined' ? () => true : () => navigator.onLine;
}

function resolveOptions(options: ConnectivityStatusOptions): ResolvedConnectivityStatusOptions {
  return {
    checkHealth: options.checkHealth ?? getHealth,
    eventTarget: options.eventTarget ?? getDefaultEventTarget(),
    onlineSource: options.onlineSource ?? getDefaultOnlineSource(),
    healthyPollIntervalMs: options.healthyPollIntervalMs ?? DEFAULT_HEALTHY_POLL_INTERVAL_MS,
    retryPollIntervalMs: options.retryPollIntervalMs ?? DEFAULT_RETRY_POLL_INTERVAL_MS,
    failureThreshold: options.failureThreshold ?? DEFAULT_FAILURE_THRESHOLD,
  };
}

function clearScheduledCheck(state: ConnectivityRuntimeState) {
  if (state.timerId !== undefined) {
    clearTimeout(state.timerId);
    state.timerId = undefined;
  }
}

function createOfflineMarker(
  state: ConnectivityRuntimeState,
  setStatus: Setter<ConnectivityStatus>,
) {
  return () => {
    clearScheduledCheck(state);
    state.consecutiveFailures = 0;
    state.runId += 1;
    setStatus('offline');
  };
}

function createCheckScheduler(
  state: ConnectivityRuntimeState,
  runHealthCheck: () => Promise<void>,
) {
  return (delayMs: number) => {
    if (state.disposed) {
      return;
    }

    clearScheduledCheck(state);
    state.timerId = setTimeout(() => {
      void runHealthCheck();
    }, delayMs);
  };
}

function createHealthCheckRunner(
  options: ResolvedConnectivityStatusOptions,
  state: ConnectivityRuntimeState,
  setStatus: Setter<ConnectivityStatus>,
) {
  const markOffline = createOfflineMarker(state, setStatus);

  const runHealthCheck = async () => {
    if (state.disposed) {
      return;
    }

    if (!options.onlineSource()) {
      markOffline();
      return;
    }

    const currentRunId = ++state.runId;
    const scheduleNextCheck = createCheckScheduler(state, runHealthCheck);

    try {
      await options.checkHealth();
      if (state.disposed || currentRunId !== state.runId) {
        return;
      }

      state.consecutiveFailures = 0;
      setStatus('online');
      scheduleNextCheck(options.healthyPollIntervalMs);
    } catch {
      if (state.disposed || currentRunId !== state.runId) {
        return;
      }

      state.consecutiveFailures += 1;
      if (state.consecutiveFailures >= options.failureThreshold) {
        setStatus('service-unreachable');
      }
      scheduleNextCheck(options.retryPollIntervalMs);
    }
  };

  return {
    markOffline,
    runHealthCheck,
  };
}

export function useConnectivityStatus(options: ConnectivityStatusOptions = {}) {
  const resolvedOptions = resolveOptions(options);
  const [status, setStatus] = createSignal<ConnectivityStatus>(
    resolvedOptions.onlineSource() ? 'online' : 'offline',
  );
  const state: ConnectivityRuntimeState = {
    timerId: undefined,
    consecutiveFailures: 0,
    runId: 0,
    disposed: false,
  };
  const { markOffline, runHealthCheck } = createHealthCheckRunner(
    resolvedOptions,
    state,
    setStatus,
  );
  const handleOnline = () => {
    state.consecutiveFailures = 0;
    setStatus('online');
    clearScheduledCheck(state);
    void runHealthCheck();
  };

  resolvedOptions.eventTarget?.addEventListener('offline', markOffline);
  resolvedOptions.eventTarget?.addEventListener('online', handleOnline);

  onCleanup(() => {
    state.disposed = true;
    clearScheduledCheck(state);
    resolvedOptions.eventTarget?.removeEventListener('offline', markOffline);
    resolvedOptions.eventTarget?.removeEventListener('online', handleOnline);
  });

  if (resolvedOptions.onlineSource()) {
    void runHealthCheck();
  }

  return { status };
}
