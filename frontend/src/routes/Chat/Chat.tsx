import { ActionIcon, Box, Button, Center, Divider, Flex, Group, Text } from '@mantine/core';
import { IconCircleX, IconLogout, IconMoodSadDizzy, IconTrash } from '@tabler/icons-react';
import { Conversation, Room } from '@types';
import {
  isRouteErrorResponse,
  LoaderFunctionArgs,
  useFetcher,
  useLoaderData,
  useNavigate,
  useParams,
  useRouteError,
} from 'react-router-dom';
import { modals } from '@mantine/modals';
import { useUser } from '@src/context/user';
import { useFetch } from '@mantine/hooks';
import { useEffect, useState } from 'react';

export async function loader({ params }: LoaderFunctionArgs): Promise<Room> {
  const roomId = params.id;

  const res = await fetch(`/api/rooms/${roomId}`);
  const data = await res.json();

  if (res.status !== 200) {
    throw new Response(data.message, { status: res.status });
  }

  return data;
}

const ACTION_TYPES = {
  DELETE_ROOM: 'delete_room',
  LEAVE_ROOM: 'leave_room',
};

type ActionData = { status: number };

export async function action({ request, params }: LoaderFunctionArgs) {
  const room_id = params.id!;
  const formData = await request.formData();
  const formDataObj = Object.fromEntries(formData);

  switch (formDataObj.type) {
    case ACTION_TYPES.DELETE_ROOM: {
      // const res = await fetch(`/api/rooms/${formDataObj.room_id}`, {
      //   method: 'delete',
      // });

      // if (res.status !== 200) {
      //   throw new Response('Failed to delete room', { status: res.status });
      // }

      return;
    }

    case ACTION_TYPES.LEAVE_ROOM: {
      const res = await fetch(`/api/rooms/${room_id}/exit`, {
        method: 'post',
      });

      return {
        status: res.status,
      };
    }
  }
}

export function ErrorBoundary() {
  const error = useRouteError();

  if (isRouteErrorResponse(error)) {
    return (
      <Center h="100%">
        <IconMoodSadDizzy size={50} />
        <Text fz="h3">{error.status === 404 ? 'Room not found' : 'An error occurred'}</Text>
      </Center>
    );
  }

  throw error;
}

export default function Chat() {
  const user = useUser()!;
  const id = useParams().id!;
  const roomData = useLoaderData() as Room;
  const navigate = useNavigate();
  const isOwner = roomData.room.owner_id === user.id;

  const deleteRoomFetcher = useFetcher();
  const leaveRoomFetcher = useFetcher<ActionData>();

  useEffect(() => {
    if (leaveRoomFetcher.data && leaveRoomFetcher.data.status === 200) {
      navigate('/');
    }
  }, [leaveRoomFetcher.data, navigate]);

  const openDeleteModal = () => {
    modals.openConfirmModal({
      title: 'Delete room',
      children: (
        <Text>
          Are you sure you want to delete this room? This action cannot be undone, and all conversations in the room
          will be deleted.
        </Text>
      ),
      labels: {
        confirm: 'Delete',
        cancel: 'Cancel',
      },
      confirmProps: { color: 'red' },
      onConfirm: () => {
        deleteRoomFetcher.submit(
          { type: ACTION_TYPES.DELETE_ROOM },
          {
            method: 'post',
          }
        );
      },
    });
  };

  const [conversations, setConversations] = useState<Conversation[]>([]);

  return (
    <Box>
      <Flex align="center" justify="space-between" p="sm">
        <Text fz="h2">{roomData.room.name}</Text>
        <Group>
          {isOwner ? (
            <Button type="submit" color="red" leftSection={<IconTrash />} onClick={openDeleteModal}>
              <Text>Delete Room</Text>
            </Button>
          ) : null}
          <leaveRoomFetcher.Form method="post">
            <input type="hidden" name="type" value={ACTION_TYPES.LEAVE_ROOM} />
            <Button variant="outline" color="red" size="sm" type="submit" leftSection={<IconLogout />}>
              <Text>Leave Room</Text>
            </Button>
          </leaveRoomFetcher.Form>
        </Group>
      </Flex>
      <Divider />
    </Box>
  );
}
