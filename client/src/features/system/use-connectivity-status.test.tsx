import { createEffect, createRoot } from 'solid-js';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { useConnectivityStatus } from './use-connectivity-status';

type EventHandler = () => void;
const noop = () => {};

function createEventTargetMock() {
  const listeners = new Map<string, Set<EventHandler>>();

  return {
    addEventListener(type: string, handler: EventHandler) {
      if (!listeners.has(type)) {
        listeners.set(type, new Set());
      }

      listeners.get(type)?.add(handler);
    },
    removeEventListener(type: string, handler: EventHandler) {
      listeners.get(type)?.delete(handler);
    },
    dispatch(type: string) {
      listeners.get(type)?.forEach((handler) => handler());
    },
  };
}

function asEventTarget(
  eventTarget: ReturnType<typeof createEventTargetMock>,
): NonNullable<Parameters<typeof useConnectivityStatus>[0]>['eventTarget'] {
  return eventTarget as unknown as NonNullable<
    Parameters<typeof useConnectivityStatus>[0]
  >['eventTarget'];
}

async function flushPromises() {
  await Promise.resolve();
  await Promise.resolve();
}

function mountStatusObserver(options: Parameters<typeof useConnectivityStatus>[0]) {
  const observedStates: string[] = [];
  let dispose = noop;

  createRoot((rootDispose) => {
    dispose = rootDispose;
    const connectivity = useConnectivityStatus(options);

    createEffect(() => {
      observedStates.push(connectivity.status());
    });
  });

  return { dispose, observedStates };
}

function buildFailureCheckHealthMock() {
  return vi
    .fn()
    .mockRejectedValueOnce(new Error('first failure'))
    .mockRejectedValueOnce(new Error('second failure'));
}

function reportsOfflineStateImmediately() {
  const eventTarget = createEventTargetMock();
  const { dispose, observedStates } = mountStatusObserver({
    checkHealth: vi.fn().mockResolvedValue(null),
    onlineSource: () => false,
    eventTarget: asEventTarget(eventTarget),
  });

  expect(observedStates).toContain('offline');
  dispose();
}

async function promotesServiceUnreachableAfterTwoFailures() {
  const eventTarget = createEventTargetMock();
  const checkHealth = buildFailureCheckHealthMock();
  const { dispose, observedStates } = mountStatusObserver({
    checkHealth,
    onlineSource: () => true,
    eventTarget: asEventTarget(eventTarget),
    healthyPollIntervalMs: 30_000,
    retryPollIntervalMs: 10_000,
    failureThreshold: 2,
  });

  await flushPromises();
  await vi.advanceTimersByTimeAsync(10_000);
  await flushPromises();

  expect(checkHealth).toHaveBeenCalledTimes(2);
  expect(observedStates.at(-1)).toBe('service-unreachable');
  dispose();
}

async function returnsOnlineAfterSuccessfulRetry() {
  const eventTarget = createEventTargetMock();
  const checkHealth = buildFailureCheckHealthMock().mockResolvedValueOnce(null);
  const { dispose, observedStates } = mountStatusObserver({
    checkHealth,
    onlineSource: () => true,
    eventTarget: asEventTarget(eventTarget),
    healthyPollIntervalMs: 30_000,
    retryPollIntervalMs: 10_000,
    failureThreshold: 2,
  });

  await flushPromises();
  await vi.advanceTimersByTimeAsync(10_000);
  await flushPromises();
  await vi.advanceTimersByTimeAsync(10_000);
  await flushPromises();

  expect(checkHealth).toHaveBeenCalledTimes(3);
  expect(observedStates.at(-1)).toBe('online');
  dispose();
}

describe('useConnectivityStatus', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it(
    'reports an offline state immediately when the browser is offline',
    reportsOfflineStateImmediately,
  );

  it(
    'promotes the state to service unreachable after two consecutive health check failures',
    promotesServiceUnreachableAfterTwoFailures,
  );

  it(
    'returns to online after a successful retry following an outage',
    returnsOnlineAfterSuccessfulRetry,
  );
});
