<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import {
    RESIZE_DIVIDER_LINE_WIDTH,
    RESIZE_HANDLE_HITBOX_WIDTH,
  } from "$lib/tickgit/layout";

  export let active = false;
  export let ariaLabel = "Resize panels";

  const dispatch = createEventDispatcher<{ mousedown: MouseEvent }>();

  const offset = (RESIZE_HANDLE_HITBOX_WIDTH - RESIZE_DIVIDER_LINE_WIDTH) / 2;
</script>

<div
  class="relative self-stretch bg-[#1f2328]"
  style={`width:${RESIZE_DIVIDER_LINE_WIDTH}px; min-width:${RESIZE_DIVIDER_LINE_WIDTH}px;`}
>
  <button
    class={`absolute inset-y-0 cursor-col-resize transition ${
      active ? "bg-[#2f81f7]/20" : "hover:bg-[#2f81f7]/12"
    }`}
    style={`left:-${offset}px; width:${RESIZE_HANDLE_HITBOX_WIDTH}px;`}
    type="button"
    aria-label={ariaLabel}
    on:mousedown={(event) => dispatch("mousedown", event)}
  ></button>
</div>
