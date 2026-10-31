export type WorkspaceView = 'studio' | 'compose' | 'demos' | 'connect';
class Workspace {
  view = $state<WorkspaceView>('studio');
  library = $state(true);
  inspector = $state(true);
}
export const workspace = new Workspace();
