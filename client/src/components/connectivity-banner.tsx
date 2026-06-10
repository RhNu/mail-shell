import type { JSX } from 'solid-js';
import { Match, Switch } from 'solid-js';
import { AlertTriangle } from 'lucide-solid';
import type { ConnectivityStatus } from '../features/system/use-connectivity-status';

type ConnectivityBannerProps = {
  status: ConnectivityStatus;
};

export function ConnectivityBanner(props: ConnectivityBannerProps): JSX.Element {
  return (
    <Switch>
      <Match when={props.status === 'offline'}>
        <Banner>网络已断开，应用当前无法连接服务器。</Banner>
      </Match>
      <Match when={props.status === 'service-unreachable'}>
        <Banner>暂时无法连接 Mail Shell 服务，正在重试。</Banner>
      </Match>
      <Match when={props.status === 'online'}>
        <></>
      </Match>
    </Switch>
  );
}

function Banner(props: { children: JSX.Element }): JSX.Element {
  return (
    <div
      role="alert"
      class="animate-slide-down mx-auto mt-4 flex w-full max-w-5xl items-start gap-3 rounded-sm border border-amber-200 bg-amber-50 px-4 py-3 text-amber-900 dark:border-amber-900/40 dark:bg-amber-950/30 dark:text-amber-200"
    >
      <AlertTriangle size={18} class="mt-0.5 shrink-0" aria-hidden="true" />
      <p class="text-sm font-medium">{props.children}</p>
    </div>
  );
}
