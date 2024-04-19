<script>
import Message from './Message.svelte';

export let current_room;
</script>

<div class="flex h-full flex-col bg-slate-100 p-4">
  {#if current_room.id}
    {#each current_room.messages as message}
      <Message message={message} />
    {/each}

  {/if}
</div>
