import { ActionIcon, Box, Button, Flex, Grid, Group, Modal, ScrollArea, Text, TextInput, Title } from '@mantine/core';
import { useDisclosure } from '@mantine/hooks';
import { IconBug, IconUsers } from '@tabler/icons-react';
import { ListRoom, User } from '@types';
import { ActionFunctionArgs, Link, Outlet, useFetcher, useLoaderData, useNavigate } from 'react-router-dom';
import { z } from 'zod';
import { get } from 'radash';
import { useEffect, useState } from 'react';
import { createInitValue, useWS, WsContextValue, WSProvider, wsSchema } from '@src/context/ws';
import { useUser } from '@src/context/user';

export async function loader(): Promise<ListRoom[]> {
  const res = await fetch('/api/rooms').then(res => res.json());
  return res;
}

const ACTION_TYPES = {
  LOGOUT: 'logout',
  CREATE_ROOM: 'create_room',
  JOIN_ROOM: 'join_room',
} as const;

const logoutActionSchema = z.object({
  type: z.literal(ACTION_TYPES.LOGOUT),
});

const createRoomActionSchema = z.object({
  type: z.literal(ACTION_TYPES.CREATE_ROOM),
  room_name: z.string().min(1),
});

const joinRoomActionSchema = z.object({
  type: z.literal(ACTION_TYPES.JOIN_ROOM),
  conn_id: z.string(),
  room_id: z.string(),
});

const actionSchema = z.union([logoutActionSchema, createRoomActionSchema, joinRoomActionSchema]);

export async function action({ request }: ActionFunctionArgs) {
  const formData = await request.formData();
  const formDataObj = Object.fromEntries(formData);

  const result = actionSchema.safeParse(formDataObj);

  if (!result.success) {
    return {
      status: 400,
      data: {
        success: false,
        message: {
          room_name: result.error.formErrors.fieldErrors.room_name?.[0],
        },
      },
    };
  }

  switch (result.data.type) {
    case ACTION_TYPES.LOGOUT:
      await fetch('/api/auth/logout', {
        method: 'post',
      });
      return {
        status: 200,
        data: {
          success: true,
        },
      };

    case ACTION_TYPES.CREATE_ROOM: {
      const res = await fetch('/api/rooms', {
        method: 'post',
        body: JSON.stringify(result.data),
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (res.status !== 200) {
        return {
          status: res.status,
          error: await res.text(),
        };
      }

      return {
        status: res.status,
        data: await res.json(),
      };
    }

    case ACTION_TYPES.JOIN_ROOM: {
      const data = result.data as z.infer<typeof joinRoomActionSchema>;
      const res = await fetch(`/api/rooms/${data.room_id}/join`, {
        method: 'post',
        headers: {
          'Conn-Id': data.conn_id,
        },
      });

      return {
        status: res.status,
        room_id: data.room_id,
      };
    }
  }
}

function CreateRoomModalForm({ close }: { close: () => void }) {
  const wsContext = useWS();
  const createRoomFetcher = useFetcher();
  useEffect(() => {
    if (createRoomFetcher.data) {
      if (createRoomFetcher.data.status === 200) {
        close();
      }
    }
  }, [createRoomFetcher.data, close]);

  return (
    <createRoomFetcher.Form method="post">
      <input type="hidden" name="type" value={ACTION_TYPES.CREATE_ROOM} />
      <input type="hidden" name="conn_id" value={wsContext.conn_id || ''} />
      <TextInput
        type="text"
        placeholder="Room Name"
        name="room_name"
        error={
          get(createRoomFetcher, 'data.data.message.room_name', null) || get(createRoomFetcher, 'data.error', null)
        }
      />
      <Flex justify="end" mt="xl" gap="md">
        <Button onClick={close} variant="outline">
          Close
        </Button>
        <Button type="submit">Create</Button>
      </Flex>
    </createRoomFetcher.Form>
  );
}

export type RouteContext = {
  rooms: ListRoom[];
};

export default function ChatRoom() {
  const user = useUser();
  const initRooms = useLoaderData() as ListRoom[];
  const [rooms, setRooms] = useState(initRooms);
  const navigate = useNavigate();
  const logoutFetcher = useFetcher();

  const [createRoomModalOpened, { open, close }] = useDisclosure(false);

  const createRoomFetcher = useFetcher();
  useEffect(() => {
    if (createRoomFetcher.data) {
      if (createRoomFetcher.data.status === 200) {
        close();
      }
    }
  }, [createRoomFetcher.data, close]);

  const joinRoomFetcher = useFetcher();
  useEffect(() => {
    if (joinRoomFetcher.data && joinRoomFetcher.data.status === 200) {
      navigate(`/room/${joinRoomFetcher.data.room_id}`);
    }
  }, [joinRoomFetcher.data, navigate]);

  const [wsContext, setWsContext] = useState<WsContextValue>(createInitValue());
  useEffect(() => {
    const _ws = new WebSocket('/ws');
    _ws.onopen = () => {
      console.log('Connected to WS');
      _ws.send(JSON.stringify({ message: 'hello' }));
    };
    _ws.onclose = () => {
      console.log('Disconnected from WS');
    };
    _ws.onerror = e => {
      console.error('WS error:', e);
    };

    setWsContext(prev => {
      return {
        ...prev,
        ws: _ws,
      };
    });

    return () => {
      console.log('Closing WS');
      _ws.close();
    };
  }, []);

  useEffect(() => {
    if (!wsContext.ws) {
      return;
    }
    const ws = wsContext.ws;

    const handleMessage = (event: MessageEvent) => {
      const data = JSON.parse(event.data);

      const result = wsSchema.safeParse(data);
      if (!result.success) {
        return;
      }

      switch (result.data.type) {
        case 'init': {
          const conn_id = result.data.data.conn_id;
          setWsContext(prev => {
            return {
              ...prev,
              conn_id,
            };
          });
          break;
        }
        case 'join_room': {
          const { room_id, user } = result.data.data;

          setRooms(prev => {
            return prev.map(room => {
              if (room.room.id === room_id) {
                if (room.users.find(u => u.id === user.id)) {
                  return room;
                }
                return {
                  ...room,
                  users: [...room.users, user],
                };
              }
              return room;
            });
          });

          break;
        }

        case 'exit_room': {
          const { room_id, user_id } = result.data.data;
          setRooms(prev => {
            return prev.map(room => {
              if (room.room.id === room_id) {
                return {
                  ...room,
                  users: room.users.filter(user => user.id !== user_id),
                };
              }
              return room;
            });
          });
          break;
        }

        case 'create_room': {
          const roomResponse = result.data.data;
          setRooms(prev => {
            return [
              ...prev,
              {
                room: roomResponse.room,
                users: roomResponse.users,
              },
            ];
          });
          break;
        }

        case 'delete_room': {
          const { room_id } = result.data.data;
          setRooms(prev => {
            return prev.filter(room => room.room.id !== room_id);
          });
          break;
        }
      }
    };

    ws.addEventListener('message', handleMessage);

    return () => {
      ws.removeEventListener('message', handleMessage);
    };
  }, [wsContext.ws]);

  return (
    <WSProvider value={wsContext}>
      <Grid
        gutter={0}
        className="min-h-screen"
        classNames={{
          inner: 'min-h-screen',
        }}
      >
        <Grid.Col p="md" span={3}>
          <Flex direction="column" h="100%" justify="space-between">
            <ScrollArea>
              <Button w="100%" onClick={open}>
                Create New Room
              </Button>
              <Box mt="md">
                <form
                  onSubmit={e => {
                    e.preventDefault();
                    const formData = Object.fromEntries(new FormData(e.target as HTMLFormElement));
                    wsContext.ws?.send(formData.command);
                  }}
                >
                  <TextInput
                    autoComplete="off"
                    type="text"
                    name="command"
                    placeholder="Debug command"
                    rightSection={
                      <ActionIcon variant="outline" size="sm" type="submit">
                        <IconBug size={16} />
                      </ActionIcon>
                    }
                  />
                </form>
              </Box>
              <Box>
                <joinRoomFetcher.Form method="post">
                  {rooms.map(room => (
                    <Box
                      mt="sm"
                      className="block cursor-pointer rounded-md border hover:bg-gray-100 transition duration-150 ease-in-out"
                      p="xs"
                      key={room.room.id}
                      h="6rem"
                      onClick={() => {
                        const inRoom = room.users.find(u => u.id === user?.id);
                        if (inRoom) {
                          navigate(`/room/${room.room.id}`);
                          return;
                        }

                        joinRoomFetcher.submit(
                          { type: 'join_room', room_id: room.room.id, conn_id: wsContext.conn_id || '' },
                          { method: 'post' }
                        );
                      }}
                    >
                      <Group gap="0" justify="space-between">
                        <Title order={4}>{room.room.name}</Title>
                        <Group gap="0">
                          <IconUsers className="ml-2" size={14} />
                          <Text c="gray" size="sm" className="ml-1">
                            {room.users.length}
                          </Text>
                        </Group>
                      </Group>
                      <Text c="gray">{room.room.last_message}</Text>
                    </Box>
                  ))}
                </joinRoomFetcher.Form>
              </Box>
            </ScrollArea>
            <logoutFetcher.Form method="post">
              <input type="hidden" name="type" value={ACTION_TYPES.LOGOUT} />
              <Button w="100%" type="submit" variant="outline">
                Logout
              </Button>
            </logoutFetcher.Form>
          </Flex>
        </Grid.Col>
        <Grid.Col span={9} p="0" className="border-l max-h-screen">
          <Outlet
            context={{
              rooms,
            }}
          />
        </Grid.Col>
      </Grid>
      <Modal opened={createRoomModalOpened} onClose={close} title="Create New Room">
        <CreateRoomModalForm close={close} />
      </Modal>
    </WSProvider>
  );
}
