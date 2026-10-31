import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { PlaybackState } from '../ipc/types';

const mocks = vi.hoisted(() => ({
  backend: { play: vi.fn(), stop: vi.fn(), playbackState: vi.fn(), setLooping: vi.fn() },
  editor: { state: { length_bars: 4 }, notify: vi.fn() },
}));
vi.mock('../ipc/client', () => ({ backend: mocks.backend, errorMessage: String }));
vi.mock('./editor.svelte', () => ({ editor: mocks.editor }));
import { TransportStore } from './transport.svelte';

const stopped: PlaybackState = { playing: false, position_bars: null, position_seconds: null };
const playing: PlaybackState = { playing: true, position_bars: 1, position_seconds: 2 };
function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => { resolve = done; });
  return { promise, resolve };
}

describe('transport reliability', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.resetAllMocks();
    mocks.backend.play.mockResolvedValue(playing);
    mocks.backend.stop.mockResolvedValue(stopped);
    mocks.backend.playbackState.mockResolvedValue(playing);
    mocks.backend.setLooping.mockResolvedValue(playing);
  });
  afterEach(() => vi.useRealTimers());

  it('ignores an old stopped reply after a new Play', async () => {
    const t = new TransportStore();
    const stale = deferred<PlaybackState>();
    mocks.backend.playbackState.mockReturnValueOnce(stale.promise);
    await t.play();
    await t.stop();
    await t.play();
    stale.resolve(stopped);
    await Promise.resolve();
    expect(t.playing).toBe(true);
    await vi.advanceTimersByTimeAsync(64);
    expect(mocks.backend.playbackState).toHaveBeenCalledTimes(4);
  });

  it('ignores an old playing reply after Stop', async () => {
    const t = new TransportStore();
    const stale = deferred<PlaybackState>();
    mocks.backend.playbackState.mockReturnValueOnce(stale.promise);
    await t.play();
    await t.stop();
    stale.resolve(playing);
    await Promise.resolve();
    expect(t.playing).toBe(false);
    expect(t.seconds).toBeNull();
  });

  it('does not enqueue duplicate Play requests while starting', async () => {
    const t = new TransportStore();
    const pending = deferred<PlaybackState>();
    mocks.backend.play.mockReturnValue(pending.promise);
    const start = t.toggle();
    await t.toggle();
    expect(t.busy).toBe(true);
    expect(mocks.backend.play).toHaveBeenCalledTimes(1);
    pending.resolve(playing);
    await start;
    expect(t.busy).toBe(false);
  });

  it('retries temporary status failures and reports persistent errors', async () => {
    const t = new TransportStore();
    mocks.backend.playbackState.mockRejectedValueOnce('lost IPC').mockRejectedValueOnce('lost IPC').mockRejectedValueOnce('lost IPC');
    await t.play();
    await vi.advanceTimersByTimeAsync(500);
    expect(t.playing).toBe(true);
    expect(mocks.editor.notify).toHaveBeenCalledWith('Playback status unavailable: lost IPC', true);
    await vi.advanceTimersByTimeAsync(250);
    expect(t.seconds).toBe(2);
  });

  it('shows device failures and permits a fresh Play attempt', async () => {
    const t = new TransportStore();
    mocks.backend.playbackState.mockResolvedValueOnce({ ...stopped, error: 'Device lost. Press Play to reconnect.' });
    await t.play();
    expect(t.playing).toBe(false);
    expect(t.busy).toBe(false);
    expect(mocks.editor.notify).toHaveBeenCalledWith('Device lost. Press Play to reconnect.', true);
    await t.play();
    expect(t.playing).toBe(true);
    expect(mocks.backend.play).toHaveBeenCalledTimes(2);
  });

  it('changes the running loop without restarting and keeps its setting on failure', async () => {
    const t = new TransportStore();
    await t.play();
    await t.toggleLoop();
    expect(mocks.backend.setLooping).toHaveBeenCalledWith(false);
    expect(mocks.backend.play).toHaveBeenCalledTimes(1);
    expect(t.looping).toBe(false);
    mocks.backend.setLooping.mockRejectedValueOnce('queue full');
    await t.toggleLoop();
    expect(t.looping).toBe(false);
    expect(mocks.editor.notify).toHaveBeenCalledWith('queue full', true);
  });
});
