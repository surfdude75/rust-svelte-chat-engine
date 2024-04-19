let onopen = function (event) {
  console.log("WebSocket is open now.", this);
};
let onmessage = function ({ message, sender, room }) {
  console.log(`message ${message} in room ${room} from ${sender}.`);
  this.dispatchEvent(
    new CustomEvent("message", { detail: { message, sender, room } }),
  );
};
let onroomslist = function (rooms) {
  console.log("rooms list", rooms);
  this.dispatchEvent(new CustomEvent("roomslist", { detail: rooms }));
};
let onroomclientslist = function (clients) {
  console.log("room clients list", clients);
  this.dispatchEvent(
    new CustomEvent("roomclientslist", { detail: { clients, room_id } }),
  );
};
let onroomremoval = function (room_id) {
  console.log("rooms removal", room_id);
  this.dispatchEvent(new CustomEvent("roomremoval", { detail: room_id }));
};
let onroomcreation = function (room_id) {
  console.log("rooms creation", room_id);
  this.dispatchEvent(new CustomEvent("roomcreation", { detail: room_id }));
};
let onroomjoin = function ({ room_id, client_id }) {
  console.log(`room ${room_id} join ${client_id}`);
  this.dispatchEvent(
    new CustomEvent("roomjoin", { detail: { room_id, client_id } }),
  );
};
let onroomexit = function ({ room_id, client_id }) {
  console.log(`room ${room_id} exit ${client_id}`);
  this.dispatchEvent(
    new CustomEvent("roomexit", { detail: { room_id, client_id } }),
  );
};
let onjoin = function (event) {
  let client_id = event.client_id;
  console.log(`your client id is ${client_id}.`);
  this.dispatchEvent(new CustomEvent("join", { detail: { client_id } }));
  // this.create_room();
};
let onreceive = function (event) {
  console.log("Message received: ", event.data);
  let data = JSON.parse(event.data);
  if (data.type == "EVENT") {
    let event = data.event;
    switch (event.type) {
      case "ROOM_EXIT": {
        let { room_id, client_id } = event;
        onroomexit.call(this, { room_id, client_id });
        break;
      }
      case "CLIENT_JOIN":
        onjoin.call(this, event);
        break;
      case "ROOMS_LIST":
        onroomslist.call(this, event.rooms);
        break;
      case "ROOM_CLIENTS_LIST":
        onroomclientslist.call(this, event.clients, event.room_id);
        break;
      case "ROOM_JOIN": {
        let { room_id, client_id } = event;
        onroomjoin.call(this, { room_id, client_id });
        break;
      }
      case "ROOM_REMOVAL":
        onroomremoval.call(this, event.room_id);
        break;
      case "ROOM_CREATION":
        onroomcreation.call(this, event.room_id);
        break;
    }
  } else if (data.type == "MESSAGE") {
    console.log("DEBUG", data);
    let { sender, message, room } = data;
    onmessage.call(this, { sender, message, room });
  }
};
let send = function (data) {
  if (typeof data == "object") data = JSON.stringify(data);
  this.socket.send(data);
};

export default class Client extends EventTarget {
  constructor(url) {
    super();
    this.url = url;
    this.socket = new WebSocket(url);
    this.socket.onopen = onopen.bind(this);
    this.socket.onmessage = onreceive.bind(this);
  }
  create_room = function () {
    send.call(this, {
      action: "ROOM_CREATE",
    });
  };
  join_room = function (room_id) {
    send.call(this, {
      action: "ROOM_JOIN",
      room_id: room_id,
    });
  };
  subscribe_rooms = function () {
    send.call(this, {
      action: "ROOMS_SUBSCRIBE",
    });
  };
  unsubscribe_rooms = function () {
    send.call(this, {
      action: "ROOMS_UNSUBSCRIBE",
    });
  };
  get_room_clients_list = function (room_id) {
    send.call(this, {
      action: "ROOM_CLIENTS_LIST",
      room_id: room_id,
    });
  };
  get_rooms_list = function () {
    send.call(this, {
      action: "ROOMS_LIST",
    });
  };
  send_message_to_room = function (room_id, message) {
    let value = {
      action: "BROADCAST",
      target: {
        type: "ROOM",
        id: room_id,
      },
      data: {
        type: "MESSAGE",
        message: message,
      },
    };
    send.call(this, value);
  };
}
console.log("Client is loaded.");
