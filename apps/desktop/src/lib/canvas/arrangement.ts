// Drawing the arrangement: tracks as rows, bars as columns, clips as blocks with a preview of
// their notes. The bar ruler and track headers are DOM elements next to the canvas.

import type { Note, Track } from '../ipc/types';
import type { CanvasColors } from '../theme/themes';
import type { Cell, Grid } from './grid';

/** Height of one track row; the track headers use the same value. */
export const TRACK_HEIGHT = 52;
/** Width of one bar; the ruler uses the same value. */
export const BAR_WIDTH = 80;

export function arrangementGrid(bars: number, tracks: number): Grid {
  return {
    labelWidth: 0,
    headerHeight: 0,
    cellWidth: BAR_WIDTH,
    cellHeight: TRACK_HEIGHT,
    columns: bars,
    rows: Math.max(tracks, 1),
  };
}

/** How a pattern looks in a clip. */
export interface ClipLook {
  name: string;
  color: string;
  notes: Note[];
  /** Lowest and highest pitch, to scale the note preview. */
  low: number;
  high: number;
}

export function clipLook(name: string, color: string, notes: Note[]): ClipLook {
  const pitches = notes.map((note) => note.pitch);
  return {
    name,
    color,
    notes,
    low: pitches.length ? Math.min(...pitches) : 60,
    high: pitches.length ? Math.max(...pitches) : 60,
  };
}

/** Shortens `text` with an ellipsis until `measure` says it fits in `maxWidth`. */
export function fitText(text: string, maxWidth: number, measure: (text: string) => number): string {
  if (measure(text) <= maxWidth) return text;
  for (let length = text.length - 1; length > 0; length--) {
    const candidate = `${text.slice(0, length).trimEnd()}…`;
    if (measure(candidate) <= maxWidth) return candidate;
  }
  return '';
}

/** `#rrggbb` with an alpha channel, as a CSS color. */
export function withAlpha(hex: string, alpha: number): string {
  const value = Number.parseInt(hex.slice(1), 16);
  return `rgba(${(value >> 16) & 255}, ${(value >> 8) & 255}, ${value & 255}, ${alpha})`;
}

export interface ArrangementView {
  grid: Grid;
  tracks: Track[];
  looks: Map<number, ClipLook>;
  stepsPerBar: number;
  selectedPattern: number | null;
  /** Cell under the pointer, where a click would place the selected pattern. */
  hover: Cell | null;
  loop: { start: number; end: number } | null;
  /** Play head position in bars, while playing. */
  playBars: number | null;
  colors: CanvasColors;
}

export function drawArrangement(context: CanvasRenderingContext2D, view: ArrangementView): void {
  const { grid, colors } = view;
  const width = grid.columns * grid.cellWidth;
  const height = grid.rows * grid.cellHeight;
  context.fillStyle = colors.background;
  context.fillRect(0, 0, width, height);

  for (let row = 0; row < grid.rows; row++) {
    context.fillStyle = row % 2 === 0 ? colors.row : colors.rowAlt;
    context.fillRect(0, row * grid.cellHeight, width, grid.cellHeight);
  }
  if (view.loop) {
    context.fillStyle = colors.loopTint;
    context.fillRect(view.loop.start * grid.cellWidth, 0, (view.loop.end - view.loop.start) * grid.cellWidth, height);
  }
  for (let bar = 0; bar <= grid.columns; bar++) {
    context.fillStyle = bar % 4 === 0 ? colors.lineStrong : colors.line;
    context.fillRect(bar * grid.cellWidth, 0, 1, height);
  }

  view.tracks.forEach((track, row) => {
    for (const clip of track.clips) {
      const look = view.looks.get(clip.pattern);
      if (look) drawClip(context, view, clip.bar, row, look, { muted: track.muted, ghost: false });
    }
  });
  drawGhost(context, view);

  if (view.playBars !== null) {
    context.fillStyle = colors.playhead;
    context.fillRect(Math.round(view.playBars * grid.cellWidth), 0, 2, height);
  }
}

/** Previews the selected pattern in an empty cell under the pointer. */
function drawGhost(context: CanvasRenderingContext2D, view: ArrangementView): void {
  const { hover, selectedPattern } = view;
  const look = selectedPattern === null ? undefined : view.looks.get(selectedPattern);
  const track = hover ? view.tracks[hover.row] : undefined;
  if (!hover || !look || !track || track.clips.some((clip) => clip.bar === hover.column)) return;
  drawClip(context, view, hover.column, hover.row, look, { muted: false, ghost: true });
}

function drawClip(
  context: CanvasRenderingContext2D,
  view: ArrangementView,
  bar: number,
  row: number,
  look: ClipLook,
  state: { muted: boolean; ghost: boolean },
): void {
  const { grid } = view;
  const x = bar * grid.cellWidth + 3;
  const y = row * grid.cellHeight + 4;
  const width = grid.cellWidth - 6;
  const height = grid.cellHeight - 8;
  const selected = !state.ghost && view.selectedPattern !== null && look === view.looks.get(view.selectedPattern);

  context.globalAlpha = state.ghost ? 0.45 : state.muted ? 0.3 : 1;
  context.beginPath();
  context.roundRect(x, y, width, height, 6);
  const surface = context.createLinearGradient(0, y, 0, y + height);
  surface.addColorStop(0, withAlpha(look.color, selected ? 0.38 : 0.24));
  surface.addColorStop(1, withAlpha(look.color, 0.08));
  context.fillStyle = surface;
  context.fill();
  context.lineWidth = selected ? 2 : 1;
  context.strokeStyle = selected ? look.color : withAlpha(look.color, 0.35);
  context.setLineDash(state.ghost ? [3, 3] : []);
  context.stroke();
  context.setLineDash([]);

  // A colored onset strip makes clip boundaries legible without a heavy box.
  context.fillStyle = look.color;
  context.beginPath();
  context.roundRect(x + 1, y + 7, 2, height - 14, 1);
  context.fill();
  context.font = '500 9px Geist, system-ui, sans-serif';
  context.textBaseline = 'top';
  context.fillStyle = selected ? view.colors.selection : look.color;
  context.fillText(fitText(look.name, width - 13, (text) => context.measureText(text).width), x + 8, y + 5);
  drawPreview(context, view, look, { x: x + 8, y: y + 21, width: width - 13, height: height - 26 });
  context.globalAlpha = 1;
}

/** A tiny piano roll of the pattern's notes. */
function drawPreview(
  context: CanvasRenderingContext2D,
  view: ArrangementView,
  look: ClipLook,
  area: { x: number; y: number; width: number; height: number },
): void {
  const steps = Math.max(view.stepsPerBar, 1);
  const span = look.high - look.low;
  context.fillStyle = look.color;
  for (const note of look.notes) {
    const x = area.x + (note.start / steps) * area.width;
    const width = Math.max(1, (note.length / steps) * area.width - 1);
    const y = span === 0 ? area.y + area.height / 2 : area.y + ((look.high - note.pitch) / span) * (area.height - 2);
    context.fillRect(x, Math.round(y), width, 2);
  }
}
