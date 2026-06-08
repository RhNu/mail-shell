import { createEffect, createMemo, createSignal, on, Show } from 'solid-js';
import { useParams } from '@solidjs/router';
import { ArrowLeft, Calendar, User } from 'lucide-solid';
import { useMessageDetail } from '../../features/messages/queries';
import { ErrorBanner, EmptyState, Skeleton } from '../../components/ui';
import { AttachmentList } from '../../components/attachment-list';
import { MessageActionMenu } from '../../components/message-action-menu';
import { sanitizeEmailHtml } from '../../lib/html-sanitize';
import { HttpResponseError } from '../../api/core/errors';

function MessageMeta(props: { from: string; to: string; createdAt: string }) {
  return (
    <div class="flex flex-col gap-2 text-sm">
      <div class="flex items-start gap-2">
        <User
          size={16}
          class="mt-0.5 shrink-0 text-zinc-400 dark:text-zinc-500"
          aria-hidden="true"
        />
        <div class="flex flex-col gap-0.5">
          <div class="text-zinc-900 dark:text-zinc-100">
            <span class="text-zinc-500 dark:text-zinc-400">发件人：</span> {props.from}
          </div>
          <div class="text-zinc-900 dark:text-zinc-100">
            <span class="text-zinc-500 dark:text-zinc-400">收件人：</span> {props.to}
          </div>
        </div>
      </div>
      <div class="flex items-center gap-2">
        <Calendar size={16} class="shrink-0 text-zinc-400 dark:text-zinc-500" aria-hidden="true" />
        <time class="text-zinc-700 dark:text-zinc-300" datetime={props.createdAt}>
          {new Date(props.createdAt).toLocaleString(undefined, {
            year: 'numeric',
            month: 'long',
            day: 'numeric',
            hour: '2-digit',
            minute: '2-digit',
          })}
        </time>
      </div>
    </div>
  );
}

function DetailSkeleton() {
  return (
    <div class="flex flex-col gap-4">
      <Skeleton class="h-6 w-3/4 rounded-sm" />
      <Skeleton class="h-4 w-1/2 rounded-sm" />
      <Skeleton class="h-4 w-1/3 rounded-sm" />
      <div class="mt-4 border-t border-zinc-200 pt-4 dark:border-zinc-800">
        <Skeleton class="h-32 w-full rounded-sm" />
      </div>
    </div>
  );
}

function MessageBody(props: { html: string; bodyText: string | null | undefined }) {
  return (
    <div class="border-t border-zinc-200 pt-6 dark:border-zinc-800">
      <Show
        when={props.html}
        fallback={
          <div class="whitespace-pre-wrap text-sm leading-relaxed text-zinc-800 dark:text-zinc-200">
            {props.bodyText ?? '（无内容）'}
          </div>
        }
      >
        <div class="overflow-x-auto rounded-sm border border-zinc-200 bg-white p-4 dark:border-zinc-800 dark:bg-white">
          {/* eslint-disable-next-line solid/no-innerhtml */}
          <div class="prose prose-sm max-w-none text-zinc-900" innerHTML={props.html} />
        </div>
      </Show>
    </div>
  );
}

function RemoteResourcesNotice(props: { onLoadRemoteResources: () => void }) {
  return (
    <div class="flex flex-wrap items-center gap-3 rounded-sm border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-900 dark:border-amber-900/40 dark:bg-amber-950/20 dark:text-amber-200">
      <p class="flex-1">远程资源因安全原因已被阻止。</p>
      <button
        type="button"
        onClick={() => props.onLoadRemoteResources()}
        class="inline-flex items-center rounded-sm bg-amber-900 px-3 py-1.5 font-medium text-amber-50 transition-colors hover:bg-amber-800 dark:bg-amber-200 dark:text-amber-950 dark:hover:bg-amber-100"
      >
        加载远程资源
      </button>
    </div>
  );
}

function MessageLoadedState(props: {
  query: ReturnType<typeof useMessageDetail>;
  html: string;
  bodyText: string | null | undefined;
  remoteResourcesBlocked: boolean;
  onLoadRemoteResources: () => void;
}) {
  return (
    <>
      <div class="flex flex-col gap-4">
        <div class="flex items-start justify-between gap-3">
          <h1
            id="message-detail-heading"
            class="text-xl font-semibold text-zinc-900 break-words dark:text-zinc-100"
          >
            {props.query.data!.subject ?? '（无主题）'}
          </h1>
          <MessageActionMenu messageId={props.query.data!.id} />
        </div>
        <MessageMeta
          from={props.query.data!.from_address}
          to={props.query.data!.to_address}
          createdAt={props.query.data!.created_at}
        />
      </div>
      <Show when={props.remoteResourcesBlocked}>
        <RemoteResourcesNotice onLoadRemoteResources={props.onLoadRemoteResources} />
      </Show>
      <MessageBody html={props.html} bodyText={props.bodyText} />
      <Show when={props.query.data!.attachments.length > 0}>
        <AttachmentList attachments={props.query.data!.attachments} />
      </Show>
    </>
  );
}

function MessageDetailContent(props: {
  query: ReturnType<typeof useMessageDetail>;
  html: () => string;
  bodyText: () => string | null | undefined;
  notFound: boolean;
  remoteResourcesBlocked: boolean;
  onLoadRemoteResources: () => void;
}) {
  return (
    <>
      {props.query.isLoading ? (
        <DetailSkeleton />
      ) : props.query.data ? (
        <MessageLoadedState
          query={props.query}
          html={props.html()}
          bodyText={props.bodyText()}
          remoteResourcesBlocked={props.remoteResourcesBlocked}
          onLoadRemoteResources={props.onLoadRemoteResources}
        />
      ) : props.notFound ? (
        <EmptyState title="邮件未找到" description="您查找的邮件不存在或已被移除。" />
      ) : (
        <></>
      )}
    </>
  );
}

export function MessageDetailRoute() {
  const params = useParams<{ messageId: string }>();
  const query = useMessageDetail(() => params.messageId);
  const [allowRemoteResources, setAllowRemoteResources] = createSignal(false);
  const sanitizedHtml = createMemo(() =>
    sanitizeEmailHtml(query.data?.body_html ?? '', {
      allowRemoteResources: allowRemoteResources(),
    }),
  );
  const bodyText = () => query.data?.body_text;
  const isNotFound = () =>
    query.isError && query.error instanceof HttpResponseError && query.error.status === 404;

  createEffect(
    on(
      () => params.messageId,
      () => setAllowRemoteResources(false),
    ),
  );

  return (
    <section aria-labelledby="message-detail-heading" class="flex flex-col gap-6">
      <a
        href="#/"
        class="inline-flex w-fit items-center gap-1.5 text-sm text-zinc-500 transition-colors hover:text-zinc-900 dark:text-zinc-400 dark:hover:text-zinc-100"
      >
        <ArrowLeft size={16} /> 返回收件箱
      </a>
      {query.isError && !isNotFound() && (
        <ErrorBanner
          message={query.error?.message ?? '加载邮件失败'}
          onRetry={() => query.refetch()}
        />
      )}
      <MessageDetailContent
        query={query}
        html={() => sanitizedHtml().html}
        bodyText={bodyText}
        notFound={isNotFound()}
        remoteResourcesBlocked={sanitizedHtml().hasBlockedRemoteResources}
        onLoadRemoteResources={() => setAllowRemoteResources(true)}
      />
    </section>
  );
}
