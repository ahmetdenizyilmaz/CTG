// Play/stop and the play head position.

import { backend, errorMessage } from '../ipc/client';
import type { PlaybackState } from '../ipc/types';
import { editor } from './editor.svelte';

export class TransportStore {
  playing = $state(false);
  busy = $state(false);
  /** Play head in bars (fractional), while playing. */
  position = $state<number | null>(null);
  /** Play head in seconds from the start of the song, while playing. */
  seconds = $state<number | null>(null);
  looping = $state(true);
  private epoch = 0;
  private timer: ReturnType<typeof setTimeout> | undefined;

  async play(fromBar = 0): Promise<void> {
    if (!editor.state?.length_bars) {
      editor.notify('Generate or open a song first.');
      return;
    }
    await this.command(() => backend.play(fromBar, this.looping));
  }

  async stop(): Promise<void> {
    await this.command(() => backend.stop());
  }

  async toggle(): Promise<void> {
    await (this.playing ? this.stop() : this.play());
  }

  async toggleLoop(): Promise<void> {
    if (this.busy) return;
    const looping = !this.looping;
    await this.command(async () => {
      const state = await backend.setLooping(looping);
      this.looping = looping;
      return state;
    });
  }

  private async command(run: () => Promise<PlaybackState>): Promise<void> {
    // A held key or double click must not enqueue overlapping transport changes.
    if (this.busy) return;
    this.busy = true;
    const epoch = ++this.epoch;
    clearTimeout(this.timer);
    try {
      this.update(await run());
    } catch (error) {
      editor.notify(errorMessage(error), true);
    } finally {
      this.busy = false;
      if (this.playing) void this.poll(epoch);
    }
  }

  private update(state: PlaybackState): void {
    if (state.error) editor.notify(state.error, true);
    this.playing = state.playing;
    this.position = state.playing ? state.position_bars : null;
    this.seconds = state.playing ? state.position_seconds : null;
  }

  /** Stale replies cannot overwrite a newer Play/Stop; temporary IPC failures retry. */
  private async poll(epoch: number, failures = 0): Promise<void> {
    if (epoch !== this.epoch) return;
    try {
      const state = await backend.playbackState();
      if (epoch !== this.epoch) return;
      this.update(state);
      failures = 0;
    } catch (error) {
      if (epoch !== this.epoch) return;
      failures += 1;
      if (failures === 3) editor.notify(`Playback status unavailable: ${errorMessage(error)}`, true);
    }
    if (this.playing && epoch === this.epoch) {
      this.timer = setTimeout(() => void this.poll(epoch, failures), failures ? 250 : 32);
    }
  }
}

export const transport = new TransportStore();
