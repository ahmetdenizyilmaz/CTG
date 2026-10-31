// Drawing the pattern editor: pitches as rows, steps as columns.

import type { Note } from '../ipc/types';
import type { CanvasColors } from '../theme/themes';
import { isBlackKey, pitchClass, pitchName } from '../music/pitch';
import type { Grid } from './grid';

/** Highest and lowest pitch shown (C7 down to C1). */
export const TOP_PITCH = 96;
export const BOTTOM_PITCH = 24;

/** Narrowest and widest step column, in pixels. */
export const MIN_STEP_WIDTH = 18;
export const MAX_STEP_WIDTH = 96;

/** The grid for `steps` columns, widened to fill `availableWidth` when there is room. */
export function rollGrid(steps: number, availableWidth = 0): Grid {
  const labelWidth = 52;
  const fitted = Math.floor((availableWidth - labelWidth - 2) / Math.max(steps, 1));
  return {
    labelWidth,
    // Step numbers are drawn above the scroll area, so they stay visible while scrolling.
    headerHeight: 0,
    cellWidth: Math.min(MAX_STEP_WIDTH, Math.max(MIN_STEP_WIDTH, fitted)),
    cellHeight: 18,
    columns: steps,
    rows: TOP_PITCH - BOTTOM_PITCH + 1,
  };
}

export const pitchAtRow = (row: number): number => TOP_PITCH - row;
export const rowOfPitch = (pitch: number): number => TOP_PITCH - pitch;

export interface RollView {
  grid: Grid;
  notes: Note[];
  stepsPerBeat: number;
  scaleClasses: number[];
  color: string;
  /** Step under the play head, while this pattern is playing. */
  playStep: number | null;
  colors: CanvasColors;
}

export function drawPianoRoll(context: CanvasRenderingContext2D, view: RollView): void {
  const { grid, colors } = view;
  const width = grid.labelWidth + grid.columns * grid.cellWidth;
  const height = grid.headerHeight + grid.rows * grid.cellHeight;
  context.fillStyle = colors.background;
  context.fillRect(0, 0, width, height);
  context.font = '10px ui-monospace, monospace';
  context.textBaseline = 'middle';

  for (let row = 0; row < grid.rows; row++) {
    const pitch = pitchAtRow(row);
    const y = grid.headerHeight + row * grid.cellHeight;
    const inScale = view.scaleClasses.includes(pitchClass(pitch));
    context.fillStyle = inScale ? colors.scaleRow : isBlackKey(pitch) ? colors.blackRow : colors.whiteRow;
    context.fillRect(grid.labelWidth, y, grid.columns * grid.cellWidth, grid.cellHeight - 1);
    context.fillStyle = isBlackKey(pitch) ? colors.keyBlack : colors.row;
    context.fillRect(0, y, grid.labelWidth - 2, grid.cellHeight - 1);
    context.fillStyle = pitchClass(pitch) === 0 ? colors.playhead : colors.label;
    context.fillText(pitchName(pitch), 6, y + grid.cellHeight / 2);
    if (pitchClass(pitch) === 0) {
      context.fillStyle = colors.lineStrong;
      context.fillRect(grid.labelWidth, y + grid.cellHeight - 1, width - grid.labelWidth, 1);
    }
  }

  drawColumns(context, view, height);
  drawNotes(context, view);
}

function drawColumns(context: CanvasRenderingContext2D, view: RollView, height: number): void {
  const { grid, colors } = view;
  for (let step = 0; step <= grid.columns; step++) {
    const x = grid.labelWidth + step * grid.cellWidth;
    context.fillStyle = step % view.stepsPerBeat === 0 ? colors.lineStrong : colors.line;
    context.fillRect(x, grid.headerHeight, 1, height - grid.headerHeight);
  }
  if (view.playStep !== null) {
    context.fillStyle = colors.playheadFill;
    const x = grid.labelWidth + view.playStep * grid.cellWidth;
    context.fillRect(x, grid.headerHeight, grid.cellWidth, height - grid.headerHeight);
  }
}

function drawNotes(context: CanvasRenderingContext2D, view: RollView): void {
  const { grid, colors } = view;
  for (const note of view.notes) {
    const row = rowOfPitch(note.pitch);
    if (row < 0 || row >= grid.rows) continue;
    const x = grid.labelWidth + note.start * grid.cellWidth + 2;
    const y = grid.headerHeight + row * grid.cellHeight + 2;
    const width = Math.max(2, note.length * grid.cellWidth - 4);
    const height = grid.cellHeight - 4;
    context.globalAlpha = 0.65 + 0.35 * (note.velocity / 127);
    context.fillStyle = view.color;
    context.beginPath();
    context.roundRect(x, y, width, height, 3);
    context.fill();
    context.globalAlpha = 1;
    context.fillStyle = colors.noteEdge;
    context.fillRect(x + 3, y + height - 3, Math.max(1, (width - 6) * note.velocity / 127), 1);
    if (width >= 32) {
      context.font = '500 8px "JetBrains Mono", monospace';
      context.textBaseline = 'middle';
      context.fillStyle = '#07131b';
      const expression = note.slide_semitones ? (note.glissando ? ' steps' : ' slide') : note.zzfx_effect ? ` ${note.zzfx_effect.code}` : '';
      context.save();
      context.beginPath();context.rect(x,y,width,height);context.clip();
      context.fillText(pitchName(note.pitch)+expression, x + 5, y + height / 2 - 1);
      context.restore();
    }
  }
}
