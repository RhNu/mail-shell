import { cleanup, fireEvent, render, screen } from '@solidjs/testing-library';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { PwaUpdateController } from './pwa-update-controller';

const registerSwState = vi.hoisted(() => ({
  needRefresh: false,
  offlineReady: false,
  updateServiceWorker: vi.fn(),
}));

vi.mock('./pwa-register', () => ({
  useRegisterSW: () => ({
    offlineReady: [() => registerSwState.offlineReady, vi.fn()] as const,
    needRefresh: [() => registerSwState.needRefresh, vi.fn()] as const,
    updateServiceWorker: registerSwState.updateServiceWorker,
  }),
}));

describe('PwaUpdateController', () => {
  afterEach(() => {
    cleanup();
  });

  beforeEach(() => {
    registerSwState.needRefresh = false;
    registerSwState.offlineReady = false;
    registerSwState.updateServiceWorker.mockReset();
  });

  it('shows an update modal when a refreshed service worker is waiting', () => {
    registerSwState.needRefresh = true;

    render(() => <PwaUpdateController />);

    expect(screen.getByRole('dialog', { name: '发现新版本' })).toBeInTheDocument();
    expect(screen.getByText('有新内容可用。立即更新将刷新当前页面。')).toBeInTheDocument();
  });

  it('suppresses the modal for the current session after choosing later', async () => {
    registerSwState.needRefresh = true;

    render(() => <PwaUpdateController />);

    await fireEvent.click(screen.getByRole('button', { name: '稍后' }));

    expect(screen.queryByRole('dialog', { name: '发现新版本' })).not.toBeInTheDocument();
  });

  it('applies the waiting service worker when update now is clicked', async () => {
    registerSwState.needRefresh = true;

    render(() => <PwaUpdateController />);

    await fireEvent.click(screen.getByRole('button', { name: '立即更新' }));

    expect(registerSwState.updateServiceWorker).toHaveBeenCalledWith(true);
  });
});
