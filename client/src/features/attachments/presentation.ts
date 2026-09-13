export function isPreviewableAttachment(contentType: string | null | undefined) {
  if (!contentType) return false;
  return new Set([
    'image/jpeg',
    'image/png',
    'image/gif',
    'image/webp',
    'image/avif',
    'audio/mpeg',
    'audio/ogg',
    'video/mp4',
    'video/webm',
    'application/pdf',
    'text/plain',
  ]).has(contentType.split(';')[0]!.trim());
}
