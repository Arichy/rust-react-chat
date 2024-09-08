import {
  Box,
  Button,
  Flex,
  Grid,
  Group,
  Modal,
  ScrollArea,
  Text,
  TextInput,
  Title,
  UnstyledButton,
} from '@mantine/core';
import { useDisclosure } from '@mantine/hooks';
import { IconUsers } from '@tabler/icons-react';
import { ListRoom, User } from '@types';
import { ActionFunctionArgs, Link, Outlet, useFetcher, useLoaderData, useNavigate } from 'react-router-dom';
import { z } from 'zod';
import { get } from 'radash';
import { useEffect } from 'react';

export async function loader(): Promise<ListRoom[]> {
  const res = await fetch('/api/rooms').then(res => res.json());
  return res;
}

const ACTION_TYPES = {
  LOGOUT: 'logout',
  CREATE_ROOM: 'create_room',
  JOIN_ROOM: 'join_room',
};

const logoutActionSchema = z.object({
  type: z.literal(ACTION_TYPES.LOGOUT),
});

const createRoomActionSchema = z.object({
  type: z.literal(ACTION_TYPES.CREATE_ROOM),
  room_name: z.string().min(1),
});

const joinRoomActionSchema = z.object({
  type: z.literal(ACTION_TYPES.JOIN_ROOM),
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

  console.log(result.data);

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

      return {
        status: res.status,
        data: await res.json(),
      };
    }

    case ACTION_TYPES.JOIN_ROOM: {
      const data = result.data as z.infer<typeof joinRoomActionSchema>;
      const res = await fetch(`/api/rooms/${data.room_id}/join`, {
        method: 'post',
      });

      return {
        status: res.status,
        room_id: data.room_id,
      };
    }
  }
}

export default function ChatRoom() {
  const rooms = useLoaderData() as ListRoom[];
  const navigate = useNavigate();
  const logoutFetcher = useFetcher();

  const [createRoomModalOpened, { open, close }] = useDisclosure(false);

  const createRoomFetcher = useFetcher();

  const joinRoomFetcher = useFetcher();
  useEffect(() => {
    if (joinRoomFetcher.data && joinRoomFetcher.data.status === 200) {
      navigate(`/room/${joinRoomFetcher.data.room_id}`);
    }
  }, [joinRoomFetcher.data, navigate]);

  return (
    <>
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
                        joinRoomFetcher.submit({ type: 'join_room', room_id: room.room.id }, { method: 'post' });
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
        <Grid.Col span={9} p="0" className="border-l">
          <Outlet />
        </Grid.Col>
      </Grid>
      <Modal opened={createRoomModalOpened} onClose={close} title="Create New Room">
        <createRoomFetcher.Form method="post">
          <input type="hidden" name="type" value={ACTION_TYPES.CREATE_ROOM} />
          <TextInput
            type="text"
            placeholder="Room Name"
            name="room_name"
            error={get(createRoomFetcher, 'data.data.message.room_name', null)}
          />
          <Flex justify="end" mt="xl" gap="md">
            <Button onClick={close} variant="outline">
              Close
            </Button>
            <Button type="submit">Create</Button>
          </Flex>
        </createRoomFetcher.Form>
      </Modal>
    </>
  );
}
