import { Show } from 'solid-js';
import { useParams } from '@solidjs/router';
import { ArrowLeft, Calendar, User } from 'lucide-solid';
import { useMessageDetail } from '../../features/messages/queries';
import { ErrorBanner } from '../../components/ui/error-banner';
import { EmptyState } from '../../components/ui/empty-state';
import { Skeleton } from '../../components/ui/skeleton';
import { AttachmentList } from '../../components/attachment-list';
import { sanitizeEmailHtml } from '../../lib/html-sanitize';

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
            <span class="text-zinc-500 dark:text-zinc-400">From:</span> {props.from}
          </div>
          <div class="text-zinc-900 dark:text-zinc-100">
            <span class="text-zinc-500 dark:text-zinc-400">To:</span> {props.to}
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
            {props.bodyText ?? '(no content)'}
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

function MessageDetailContent(props: {
  query: ReturnType<typeof useMessageDetail>;
  html: () => string;
  bodyText: () => string | null | undefined;
}) {
  return (
    <>
      {props.query.isLoading ? (
        <DetailSkeleton />
      ) : props.query.data ? (
        <>
          <div class="flex flex-col gap-4">
            <h1
              id="message-detail-heading"
              class="text-xl font-semibold text-zinc-900 break-words dark:text-zinc-100"
            >
              {props.query.data.subject ?? '(no subject)'}
            </h1>
            <MessageMeta
              from={props.query.data.from_address}
              to={props.query.data.to_address}
              createdAt={props.query.data.created_at}
            />
          </div>
          <MessageBody html={props.html()} bodyText={props.bodyText()} />
          <Show when={props.query.data.attachments.length > 0}>
            <AttachmentList attachments={props.query.data.attachments} />
          </Show>
        </>
      ) : (
        <EmptyState
          title="Message not found"
          description="The message you are looking for does not exist or has been removed."
        />
      )}
    </>
  );
}

export function MessageDetailRoute() {
  const params = useParams<{ messageId: string }>();
  const query = useMessageDetail(() => params.messageId);
  const html = () => sanitizeEmailHtml(query.data?.body_html ?? '');
  const bodyText = () => query.data?.body_text;

  return (
    <section aria-labelledby="message-detail-heading" class="flex flex-col gap-6">
      <a
        href="#/"
        class="inline-flex w-fit items-center gap-1.5 text-sm text-zinc-500 transition-colors hover:text-zinc-900 dark:text-zinc-400 dark:hover:text-zinc-100"
      >
        <ArrowLeft size={16} /> Back to inbox
      </a>
      {query.isError && (
        <ErrorBanner
          message={query.error?.message ?? 'Failed to load message'}
          onRetry={() => query.refetch()}
        />
      )}
      <MessageDetailContent query={query} html={html} bodyText={bodyText} />
    </section>
  );
}
