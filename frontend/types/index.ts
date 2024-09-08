export type User = {
  id: string;
  username: string;
};

export type BaseError = {
  success: false;
  message: string;
};

export type Conversation = {
  id: string;
  user_id: string;
  room_id: string;
  message: string;
  created_at: number;
};

export type ListRoom = {
  room: {
    id: string;
    name: string;
    last_message: string;
    created_at: number;
    owner_id: string;
  };
  users: User[];
};

export type Room = ListRoom & {
  conversations: Conversation[];
};
