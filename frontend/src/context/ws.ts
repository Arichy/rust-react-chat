import { createContext, useContext } from 'react';
import { z } from 'zod';

export type WsContextValue = {
  ws: WebSocket | null;
  conn_id: string | null;
};

export function createInitValue(): WsContextValue {
  return {
    ws: null,
    conn_id: null,
  };
}

const WSContext = createContext<WsContextValue>(createInitValue());

export function useWS() {
  return useContext(WSContext);
}

export const WSProvider = WSContext.Provider;

const wsInitSchema = z.object({
  type: z.literal('init'),
  data: z.object({ conn_id: z.string() }),
});

const wsCreateMessageSchema = z.object({
  type: z.literal('message'),
  data: z.object({
    id: z.string(),
    user_id: z.string(),
    room_id: z.string(),
    message: z.string(),
    created_at: z.string(),
  }),
});

const wsCreateRoomSchema = z.object({
  type: z.literal('create_room'),
  data: z.object({
    id: z.string(),
    name: z.string(),
    owner_id: z.string(),
    created_at: z.string(),
  }),
});

export const wsSchema = z.discriminatedUnion('type', [wsInitSchema, wsCreateMessageSchema, wsCreateRoomSchema]);
