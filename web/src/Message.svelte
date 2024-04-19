<script>
  export let message;
</script>

<blockquote class=" mb-2">
  <p class="text-gray-800 sm:text-xl dark:text-white">{message.message}</p>
  <div class="text-base font-semibold text-gray-800 dark:text-neutral-400">{ message.sender_id }</div>
  <div class="text-xs text-gray-500 dark:text-neutral-500">{new Date(message.received_at)}</div>
</blockquote>


