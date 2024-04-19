<script>
  import { createEventDispatcher } from 'svelte';
  export let current_room;
  const dispatch = createEventDispatcher();
  let send = (e) => {
    let value = e.target.value;
    let lastkey = value[value.length-1];
    if (lastkey == '\n') {
      e.target.value = "";
      dispatch('input', value);
    }
  }
</script>
<div class="flex h-60">
  <textarea disabled={current_room.id?false:true} on:input={send} id="message" rows="4" class="block p-2.5 w-full text-sm text-gray-900 bg-gray-50 rounded-lg border border-gray-300 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500" placeholder={current_room.id?"Type here...":"Select or create a room first.."}></textarea>
</div>
