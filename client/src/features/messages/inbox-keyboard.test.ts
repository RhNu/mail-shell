import { describe, expect, it } from 'vitest';
import { inboxShortcutFor, nextMessageIndex } from './inbox-keyboard';

describe('inbox keyboard commands', () => {
  it('maps navigation and action keys', () => {
    expect(inboxShortcutFor('j', { modified: false, editing: false })).toBe('next');
    expect(inboxShortcutFor('K', { modified: false, editing: false })).toBe('previous');
    expect(inboxShortcutFor('/', { modified: false, editing: false })).toBe('search');
    expect(inboxShortcutFor('Delete', { modified: false, editing: false })).toBe('trash');
  });

  it('leaves modified keys and text editing alone', () => {
    expect(inboxShortcutFor('j', { modified: true, editing: false })).toBeUndefined();
    expect(inboxShortcutFor('s', { modified: false, editing: true })).toBeUndefined();
  });

  it('clamps focus navigation to the available rows', () => {
    expect(nextMessageIndex(-1, 3, 1)).toBe(0);
    expect(nextMessageIndex(-1, 3, -1)).toBe(2);
    expect(nextMessageIndex(2, 3, 1)).toBe(2);
    expect(nextMessageIndex(0, 3, -1)).toBe(0);
    expect(nextMessageIndex(-1, 0, 1)).toBe(-1);
  });
});
