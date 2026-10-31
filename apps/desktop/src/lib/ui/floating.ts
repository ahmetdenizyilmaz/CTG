/** Place a menu in viewport coordinates so dock overflow cannot clip it. */
export function floating(node: HTMLElement, initial: { anchor: HTMLElement | undefined; align: 'left' | 'right' }): { update: (next: typeof initial) => void; destroy: () => void } {
  let options = initial;
  function place(): void {
    if (!options.anchor) return;
    const rect = options.anchor.getBoundingClientRect();
    const width = node.getBoundingClientRect().width;
    const left = options.align === 'right' ? rect.right - width : rect.left;
    node.style.left = `${Math.max(12, Math.min(left, window.innerWidth - width - 12))}px`;
    const below = window.innerHeight - rect.bottom - 14;
    const above = rect.top - 14;
    const upwards = below < 220 && above > below;
    node.style.maxHeight = `${Math.max(80, Math.min(window.innerHeight * .75, upwards ? above : below))}px`;
    node.style.top = upwards ? `${Math.max(12, rect.top - node.getBoundingClientRect().height - 6)}px` : `${rect.bottom + 6}px`;
  }
  const frame = requestAnimationFrame(place);
  window.addEventListener('resize', place);
  // Scrolling the menu itself must not move it. Dock/viewport scrolling does.
  const onScroll = (event: Event): void => { if (!(event.target instanceof Node) || !node.contains(event.target)) place(); };
  window.addEventListener('scroll', onScroll, true);
  return {
    update(next) { options = next; place(); },
    destroy() { cancelAnimationFrame(frame); window.removeEventListener('resize', place); window.removeEventListener('scroll', onScroll, true); },
  };
}
