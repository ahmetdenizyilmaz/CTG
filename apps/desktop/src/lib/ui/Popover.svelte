<script lang="ts">
  import type { Snippet } from 'svelte';

  import { clickOutside } from './clickOutside';
  import { floating } from './floating';

  let {
    trigger,
    children,
    title,
    align = 'left',
  }: { trigger: Snippet; children: Snippet; title: string; align?: 'left' | 'right' } = $props();
  let open = $state(false);
  let anchor = $state<HTMLButtonElement>();
</script>

<div class="popover" use:clickOutside={() => (open = false)}>
  <button bind:this={anchor} class="ghost trigger" class:open {title} aria-expanded={open} onclick={() => (open = !open)}>
    {@render trigger()}
  </button>
  {#if open}
    <div class="content" use:floating={{ anchor, align }} role="dialog" aria-label={title}>
      {@render children()}
    </div>
  {/if}
</div>

<style>
  .popover {
    position: relative;
  }

  .trigger {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .content {
    position: fixed;
    z-index: 100;
    overflow-y: auto;
    max-width: calc(100vw - 24px);
    padding: 14px;
    background: var(--panel-raised);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: var(--shadow);
  }

</style>
