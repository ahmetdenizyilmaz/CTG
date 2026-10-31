// Reset points are independent of undo history, which can still undo a reset.
import type { Instrument } from '../ipc/types';

export class InstrumentCheckpoints {
  private editorId: number | null = null;
  private sounds = new Map<number, Instrument>();

  observe(editorId: number, instruments: Instrument[]): void {
    if (editorId !== this.editorId) {
      this.sounds.clear();
      this.editorId = editorId;
    }
    for (const instrument of instruments) {
      if (!this.sounds.has(instrument.id)) this.save(instrument);
    }
  }

  save(instrument: Instrument): void {
    this.sounds.set(instrument.id, structuredClone(instrument));
  }

  get(id: number): Instrument | undefined {
    const original = this.sounds.get(id);
    return original ? structuredClone(original) : undefined;
  }

  replace(editorId: number, instruments: Instrument[]): void {
    this.sounds.clear();
    this.editorId = editorId;
    this.observe(editorId, instruments);
  }
}
