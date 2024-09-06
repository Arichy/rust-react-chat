import { Box, Button, Divider, Flex, Grid, Group, Modal, ScrollArea, Text, TextInput, Title } from '@mantine/core';
import { useDisclosure } from '@mantine/hooks';
import { IconUsers } from '@tabler/icons-react';
import { Room, User } from '@types';
import { ActionFunctionArgs, Link, LoaderFunctionArgs, Outlet, useFetcher, useLoaderData } from 'react-router-dom';
import { z } from 'zod';

export async function loader(): Promise<Room[]> {
  const res = await fetch('/api/rooms').then(res => res.json());
  return res;
}

const ACTION_TYPES = {
  LOGOUT: 'logout',
  CREATE_ROOM: 'create_room',
};

const actionSchema = z.object({
  type: z.enum([ACTION_TYPES.LOGOUT]),
});

export async function action({ request }: ActionFunctionArgs) {
  const formData = await request.formData();
  const formDataObj = Object.fromEntries(formData);

  const result = actionSchema.safeParse(formDataObj);
  if (!result.success) {
    return {
      status: 400,
      data: {
        success: false,
        message: 'Invalid form data',
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

      return {
        status: res.status,
        data: await res.json(),
      };
    }
  }
}

export default function ChatRoom() {
  const rooms = useLoaderData() as { room: Room; users: User[] }[];
  const logoutFetcher = useFetcher();

  const [createRoomModalOpened, { open, close }] = useDisclosure(false);

  const createRoomFetcher = useFetcher();

  return (
    <>
      <Grid
        className="min-h-screen"
        classNames={{
          inner: 'min-h-screen',
        }}
        styles={{
          inner: {
            margin: 0,
          },
        }}
        overflow="hidden"
      >
        <Grid.Col p="md" span={3}>
          <Flex direction="column" h="100%" justify="space-between">
            <ScrollArea>
              <Button w="100%" onClick={open}>
                Create New Room
              </Button>
              <Box>
                {rooms.map(room => (
                  <Box
                    mt="sm"
                    component={Link}
                    to={`/room/${room.room.id}`}
                    className="block rounded-md border hover:bg-gray-100 transition duration-150 ease-in-out"
                    p="xs"
                    key={room.room.id}
                    h="6rem"
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
              </Box>
            </ScrollArea>
            <logoutFetcher.Form method="post">
              <input type="hidden" name="type" value="logout" />
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
          <TextInput type="text" placeholder="Room Name" name="room_name" />
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
