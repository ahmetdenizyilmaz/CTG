// A Svelte action that calls `onClose` on a pointer press outside the element or on Escape,
// used by menus and popovers.

export function clickOutside(node: HTMLElement, onClose: () => void): { update: (next: () => void) => void; destroy: () => void } {
  let close = onClose;
  const onPointer = (event: PointerEvent): void => {
    if (event.composedPath().includes(node)) return;
    // Native select popups can retarget their pointer press to the document root.
    const active = document.activeElement;
    if (active instanceof HTMLSelectElement && node.contains(active)
      && (event.target === document.documentElement || event.target === document.body)) return;
    if (event.target instanceof Node && !node.contains(event.target)) close();
  };
  const onKey = (event: KeyboardEvent): void => {
    if (event.key === 'Escape') close();
  };
  window.addEventListener('pointerdown', onPointer, true);
  window.addEventListener('keydown', onKey);
  return {
    update(next) {
      close = next;
    },
    destroy() {
      window.removeEventListener('pointerdown', onPointer, true);
      window.removeEventListener('keydown', onKey);
    },
  };
}
