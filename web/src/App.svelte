<script>
  import Client from './lib/client.js';
  import RoomsList from './RoomsList.svelte';
  import ChatArea from './ChatArea.svelte';
  import TextInput from './TextInput.svelte';

  let connection_id = "";

  let connected = false;

  let current_room = {};

  let rooms = {};

  let utils = {
    create_room: function(room_id) {
      console.log('create');
      let room = {
        id: room_id,
        clients: [],
        connected: false,
        messages: []
      };
      rooms[room_id] = room;
      return room;
    }
  };

  let onroomcreation = function(event) {
    let room_id = event.detail;
    let room = rooms[room_id];
    if (!room) {
      utils.create_room(room_id);
      rooms = rooms;
    };
    console.log("creation ", room_id, rooms);
  }
  let onroomremoval = function(event) {
    let room_id = event.detail;
    let room = rooms[room_id];
    if ( room ) {
      //TODO: If current handle it
      delete rooms[room_id];
      rooms = rooms;
    };
  }
  let onroomslist = function(event) {
    let _rooms = event.detail;
    let updated = false;
    for (let room_id of _rooms) {
      if (!rooms[room_id]) {
        utils.create_room(room_id);
        updated = true;
      }
    }
    if (updated) rooms = rooms;

    console.log("ON ROOMS LIST", rooms);
  }
  let onroomclientslist = function(event) {
    let clients = event.detail.clients;
    let room_id = event.detail.room_id;
    let room = rooms[room_id];
    if (room) {
      room.clients = clients;
      rooms = rooms;
    }

  }
  let onroomjoin = function(event) {
    let client_id = event.detail.client_id;
    let room_id = event.detail.room_id;
    let room = rooms[room_id];

    if (room) {
      room.clients.push(client_id);
    } else {
        room = utils.create_room(room_id);
        room.clients.push(client_id);
    }
    if (client_id == connection_id) {
      room.connected = true;
      current_room = room;
      client.get_room_clients_list();
    }
    rooms = rooms;
    console.log("ON ROOM JOIN", rooms);
  }
  let onmessage = function(event) {
    let message = event.detail.message;
    let room_id = event.detail.room; //TODO: Maybe refactor it
    let sender_id = event.detail.sender;
    let room = rooms[room_id];
    if (room) {
      room.messages.push({
        message, sender_id, received_at: Date.now()
      });
      if (current_room.id == room_id) {
        current_room = current_room;
      }
    }
    console.log("DEBUG MESSAGE", current_room);
  }
  let roomselect = function (event) {
    let room_id = event.detail;
    console.log("room select2333", room_id); 
    let room = rooms[room_id];
    if (room) {
      if (!room.connected) {
        console.log("select join",room_id);
        client.join_room(room.id);
      } else {
        current_room = room;
      }
    }
  }
  let client = new Client("ws://localhost:3030/ws");
  client.addEventListener('join', function(evt){
    let client_id = evt.detail.client_id;
    connection_id = client_id;
    client.addEventListener('roomslist', onroomslist);
    client.addEventListener('roomremoval', onroomremoval);
    client.addEventListener('roomcreation', onroomcreation);
    client.addEventListener('roomclientslist', onroomclientslist);
    client.addEventListener('roomjoin', onroomjoin);
    client.addEventListener('message', onmessage);
    client.subscribe_rooms();
    ;
    connected = true;
  }, { once: true })

  let create_room = function() {
    client.create_room();
  }

  let send = function(e) {
    let value = e.detail;
    if (current_room) {
      console.log("Send", value);
      client.send_message_to_room(current_room.id, value);
    }
  }

</script>

<div class="h-[96vh] m-[2vh]">

<div class="flex flex-row gap-4 h-full">

  <div class="flex gap-4 flex-col flex-none">
    <ul class="flex flex-col">
      <li class="inline-flex items-center gap-x-2 py-3 px-4 text-sm font-bold bg-white border border-gray-200 text-gray-800 -mt-px first:rounded-t-lg first:mt-0 last:rounded-b-lg dark:bg-slate-900 dark:border-gray-700 dark:text-white">
      Your ID 
      </li>
      <li class="inline-flex items-center gap-x-2 py-3 px-4 text-sm font-medium bg-white border border-gray-200 text-gray-800 -mt-px first:rounded-t-lg first:mt-0 last:rounded-b-lg dark:bg-slate-900 dark:border-gray-700 dark:text-white">
      {connection_id}
      </li>

    </ul>

    <button disabled={!connected} on:click={create_room} type="button" class="py-3 px-4 inline-flex justify-center items-center gap-x-2 text-sm font-semibold rounded-lg border border-transparent bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-50 disabled:pointer-events-none">
      Create new room
    </button>

    <RoomsList rooms={rooms} current_room={current_room} on:roomselect={roomselect}/>
  </div>

  <div class="flex flex-col gap-4 flex-1"> 
    <ChatArea current_room={current_room} />
    <TextInput current_room={current_room} on:input={send} />
  </div>

</div>

</div>
<style lang="postcss">

</style>
