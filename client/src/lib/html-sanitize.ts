import DOMPurify from 'dompurify';

const ALLOWED_TAGS = [
  'p',
  'br',
  'hr',
  'div',
  'span',
  'h1',
  'h2',
  'h3',
  'h4',
  'h5',
  'h6',
  'ul',
  'ol',
  'li',
  'dl',
  'dt',
  'dd',
  'table',
  'thead',
  'tbody',
  'tfoot',
  'tr',
  'td',
  'th',
  'caption',
  'colgroup',
  'col',
  'a',
  'img',
  'strong',
  'b',
  'em',
  'i',
  'u',
  's',
  'strike',
  'del',
  'sub',
  'sup',
  'blockquote',
  'pre',
  'code',
  'font',
];

const ALLOWED_ATTR = [
  'href',
  'title',
  'alt',
  'src',
  'width',
  'height',
  'style',
  'class',
  'id',
  'name',
  'target',
  'rel',
  'border',
  'cellpadding',
  'cellspacing',
  'bgcolor',
  'color',
  'align',
  'valign',
  'dir',
];

type SanitizeEmailHtmlOptions = {
  allowRemoteResources?: boolean;
};

export type SanitizedEmailHtml = {
  html: string;
  hasBlockedRemoteResources: boolean;
};

const SAFE_RESOURCE_PREFIXES = ['cid:', 'data:', 'blob:', '/', '#'];
const REMOTE_STYLE_URL_PATTERN = /url\s*\(\s*(['"]?)(.*?)\1\s*\)/giu;

function isSafeResourceUrl(url: string): boolean {
  const normalized = url.trim().toLowerCase();
  return SAFE_RESOURCE_PREFIXES.some((prefix) => normalized.startsWith(prefix));
}

function blockRemoteResources(html: string): SanitizedEmailHtml {
  const document = new DOMParser().parseFromString(html, 'text/html');
  let hasBlockedRemoteResources = false;

  for (const image of document.querySelectorAll('img[src]')) {
    const src = image.getAttribute('src');
    if (src !== null && !isSafeResourceUrl(src)) {
      image.remove();
      hasBlockedRemoteResources = true;
    }
  }

  for (const element of document.querySelectorAll('[style]')) {
    const style = element.getAttribute('style');
    if (style === null) {
      continue;
    }

    const hasRemoteStyleResource = Array.from(style.matchAll(REMOTE_STYLE_URL_PATTERN)).some(
      ([, , resourceUrl]) => !isSafeResourceUrl(resourceUrl),
    );

    if (hasRemoteStyleResource) {
      element.removeAttribute('style');
      hasBlockedRemoteResources = true;
    }
  }

  return {
    html: document.body.innerHTML,
    hasBlockedRemoteResources,
  };
}

export function sanitizeEmailHtml(
  html: string,
  options: SanitizeEmailHtmlOptions = {},
): SanitizedEmailHtml {
  const sanitizedHtml = DOMPurify.sanitize(html, {
    ALLOWED_TAGS,
    ALLOWED_ATTR,
    ALLOW_DATA_ATTR: false,
  });

  if (options.allowRemoteResources) {
    return {
      html: sanitizedHtml,
      hasBlockedRemoteResources: false,
    };
  }

  return blockRemoteResources(sanitizedHtml);
}
