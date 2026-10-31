// The only module that talks to the Rust backend. Stores call these functions; components
// never call Tauri directly.

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

import type { Action, EditorState, Environment, Instrument, PlaybackState, PresetInfo } from './types';

import type { ComposeConfig, CompositionReport } from './composition';

export const backend = {
  openDemo: (id: string) => invoke<EditorState>('open_demo', { id }),
  compositionDefaults: () => invoke<ComposeConfig>('composition_defaults'),
  compositionRandom: (seed: number, instrumentsFor: ComposeConfig | null = null) => invoke<ComposeConfig>('composition_random', { seed, instrumentsFor }),
  compositionPreview: (config: ComposeConfig) => invoke<CompositionReport>('composition_preview', { config }),
  composeSong: (config: ComposeConfig, editorId: number, revision: number) => invoke<{ state: EditorState; report: CompositionReport }>('compose_song', { config, editorId, revision }),
  developPattern: (patternId: number, degrees: number, invert: boolean) => invoke<EditorState>('develop_pattern', { patternId, degrees, invert }),
  getState: () => invoke<EditorState>('get_state'),
  getEnvironment: () => invoke<Environment>('get_environment'),
  edit: (action: Action) => invoke<EditorState>('edit', { action }),
  undo: () => invoke<EditorState>('undo'),
  redo: () => invoke<EditorState>('redo'),
  newSong: (title: string) => invoke<EditorState>('new_song', { title }),
  openProject: (path: string) => invoke<EditorState>('open_project', { path }),
  saveProject: (path: string | null) => invoke<EditorState>('save_project', { path }),
  exportWav: (path: string) => invoke<string>('export_wav', { path }),
  play: (fromBar: number, looping: boolean) => invoke<PlaybackState>('play', { fromBar, looping }),
  setLooping: (looping: boolean) => invoke<PlaybackState>('set_looping', { looping }),
  stop: () => invoke<PlaybackState>('stop'),
  playbackState: () => invoke<PlaybackState>('playback_state'),
  listPresets: () => invoke<PresetInfo[]>('list_presets'),
  getPreset: (preset: string) => invoke<Omit<Instrument, 'id'>>('preset_get', { preset }),
  presetLibraryInfo: () => invoke<{ directory: string; warnings: string[] }>('preset_library_info'),
  saveInstrumentPreset: (id: number) => invoke<{ id: string; path: string; instrument: Omit<Instrument, 'id'> }>('instrument_save_preset', { id }),
  /** Called whenever the song changes, including edits made by an AI agent over MCP. */
  onSongChanged: (handler: () => void): Promise<UnlistenFn> => listen('song-changed', handler),
};

/** Turns a rejected backend call into a message for the user. */
export function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
