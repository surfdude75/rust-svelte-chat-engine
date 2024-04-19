<script>
  import { createEventDispatcher } from 'svelte';

  export let rooms;
  export let current_room;

  const dispatch = createEventDispatcher();

  function roomselect(room_id) {
    dispatch('roomselect', room_id);
  }
</script>

<ul class="flex flex-col">
  <li class="inline-flex items-center gap-x-2 py-3 px-4 text-sm font-bold bg-white border border-gray-200 text-gray-800 -mt-px first:rounded-t-lg first:mt-0 last:rounded-b-lg dark:bg-slate-900 dark:border-gray-700 dark:text-white">
    Rooms
  </li>

{#each Object.values(rooms) as room (room.id) }
  <li on:click={() => { roomselect(room.id) }} class="{(current_room.id == room.id)?"bg-blue-200":(room.connected?"bg-green-200":"")} inline-flex items-center gap-x-2 py-3 px-4 text-sm font-medium border border-gray-200 text-gray-800 -mt-px first:rounded-t-lg first:mt-0 last:rounded-b-lg dark:bg-slate-900 dark:border-gray-700 dark:text-white cursor-pointer">
    {room.id}
  </li>
{/each}

</ul>
