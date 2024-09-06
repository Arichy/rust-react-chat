export type User = {
  id: string;
  username: string;
};

export type BaseError = {
  success: false;
  message: string;
};

export type Room = {
  id: string;
  name: string;
  last_message: string;
  users: User[];
  created_at: number;
};
