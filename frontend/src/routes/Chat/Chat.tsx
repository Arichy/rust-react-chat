import {
  Box,
  Button,
  Center,
  Divider,
  Flex,
  Group,
  Paper,
  ScrollArea,
  Stack,
  Text,
  Textarea,
  Title,
} from '@mantine/core';
import { IconLogout, IconMoodSadDizzy, IconTrash } from '@tabler/icons-react';
import { Conversation, Room } from '@types';
import { v4 as uuidv4 } from 'uuid';
import {
  isRouteErrorResponse,
  LoaderFunctionArgs,
  useFetcher,
  useLoaderData,
  useNavigate,
  useOutletContext,
  useParams,
  useRouteError,
} from 'react-router-dom';
import { modals } from '@mantine/modals';
import { useUser } from '@src/context/user';
import { useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react';
import { objectify, omit } from 'radash';
import { formatDate } from '@src/utils';
import { useWS, wsSchema } from '@src/context/ws';
import clsx from 'clsx';
import { z } from 'zod';
import { RouteContext } from '../ChatRoom/ChatRoom';
import { notifications } from '@mantine/notifications';

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
  SEND_MESSAGE: 'send_message',
} as const;

type ActionData = { status: number };

const actionSchema = z.discriminatedUnion('type', [
  z.object({
    type: z.literal(ACTION_TYPES.DELETE_ROOM),
  }),
  z.object({
    type: z.literal(ACTION_TYPES.LEAVE_ROOM),
    conn_id: z.string(),
  }),
  z.object({
    type: z.literal(ACTION_TYPES.SEND_MESSAGE),
    fake_id: z.string(),
    conn_id: z.string(),
    room_id: z.string(),
    message: z.string(),
  }),
]);

type SendMessageResponse = {
  status: number;
  oldId: string;
  data: Conversation;
};

export async function action({ request, params }: LoaderFunctionArgs) {
  const room_id = params.id!;
  const formData = await request.formData();
  const formDataObj = Object.fromEntries(formData);
  const result = actionSchema.safeParse(formDataObj);
  if (!result.success) {
    return {
      status: 400,
      data: {
        success: false,
        message: result.error.errors,
      },
    };
  }

  const data = result.data;

  switch (data.type) {
    case ACTION_TYPES.DELETE_ROOM: {
      const res = await fetch(`/api/rooms/${room_id}`, {
        method: 'delete',
      });

      return {
        status: res.status,
      };
    }

    case ACTION_TYPES.LEAVE_ROOM: {
      const res = await fetch(`/api/rooms/${room_id}/exit`, {
        method: 'post',
        headers: {
          'Conn-Id': data.conn_id,
        },
      });

      return {
        status: res.status,
      };
    }

    case ACTION_TYPES.SEND_MESSAGE: {
      const res = await fetch(`/api/conversations`, {
        method: 'post',
        body: JSON.stringify(omit(data, ['type', 'fake_id'])),
        headers: {
          'Content-Type': 'application/json',
          'Conn-Id': data.conn_id,
        },
      });

      return {
        status: res.status,
        oldId: data.fake_id,
        data: await res.json(),
      } satisfies SendMessageResponse;
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
  const rooms = useOutletContext<RouteContext>().rooms;
  const navigate = useNavigate();

  const roomData = useLoaderData() as Room;

  const isOwner = roomData.room.owner_id === user.id;

  useEffect(() => {
    const room = rooms.find(room => room.room.id === id);
    // room has been deleted by owner
    if (!room) {
      notifications.show({
        title: 'Room deleted',
        message: `Room ${roomData.room.name} has been deleted.`,
        position: 'top-center',
        color: 'orange',
      });
      navigate('/');
    }
  }, [rooms, id, navigate, roomData.room.name]);

  const deleteRoomFetcher = useFetcher();
  useEffect(() => {
    if (deleteRoomFetcher.data && deleteRoomFetcher.data.status === 200) {
      navigate('/');
    }
  }, [deleteRoomFetcher.data, navigate]);

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

  const [conversations, setConversations] = useState(roomData.conversations);

  const sendMessageFetcher = useFetcher<SendMessageResponse>();

  const userHash = useMemo(() => {
    return objectify([...roomData.users, ...roomData.exited_users], user => user.id);
  }, [roomData]);

  const { ws, conn_id } = useWS();

  const [message, setMessage] = useState('');

  const idRef = useRef(id);
  if (idRef.current !== id) {
    idRef.current = id;
    setConversations(roomData.conversations);
  }

  const sendMessage = () => {
    const fakeId = uuidv4();
    setConversations(prev => {
      return [...prev, { id: fakeId, user_id: user.id, room_id: id, message, created_at: new Date().toISOString() }];
    });
    sendMessageFetcher.submit(
      { type: ACTION_TYPES.SEND_MESSAGE, fake_id: fakeId, conn_id, room_id: id, message },
      { method: 'post' }
    );
  };

  useEffect(() => {
    if (sendMessageFetcher.data && sendMessageFetcher.data.status === 200) {
      setMessage('');
      const { oldId, data } = sendMessageFetcher.data;
      setConversations(prev => {
        return prev.map(conversation => {
          if (conversation.id === oldId) {
            return data;
          }
          return conversation;
        });
      });
    }
  }, [sendMessageFetcher.data]);

  const viewportRef = useRef<HTMLDivElement>(null);
  const scrollToBottom = () => {
    viewportRef.current?.scrollTo({
      top: viewportRef.current.scrollHeight,
    });
  };
  useLayoutEffect(() => {
    scrollToBottom();
  }, [conversations]);

  useEffect(() => {
    if (!ws) {
      return;
    }
    const handleMessage = (event: MessageEvent) => {
      const data = JSON.parse(event.data);

      const result = wsSchema.safeParse(data);
      if (!result.success) {
        return;
      }

      switch (result.data.type) {
        case 'message': {
          if (result.data.data.room_id === id) {
            const newMessage = result.data.data;
            setConversations(prev => [...prev, newMessage]);
          }
          break;
        }
      }
    };
    ws.addEventListener('message', handleMessage);

    return () => {
      ws.removeEventListener('message', handleMessage);
    };
  }, [ws, id]);

  return (
    <Stack h="100%">
      <Flex align="center" justify="space-between" p="sm">
        <Title order={3}>{roomData.room.name}</Title>
        <Group>
          {isOwner ? (
            <Button type="submit" size="sm" color="red" leftSection={<IconTrash />} onClick={openDeleteModal}>
              <Text size="sm">Delete</Text>
            </Button>
          ) : null}
          <leaveRoomFetcher.Form method="post">
            <input type="hidden" name="type" value={ACTION_TYPES.LEAVE_ROOM} />
            <input type="hidden" name="conn_id" value={conn_id || ''} />
            <Button variant="outline" color="red" size="sm" type="submit" leftSection={<IconLogout />}>
              <Text size="sm">Leave</Text>
            </Button>
          </leaveRoomFetcher.Form>
        </Group>
      </Flex>
      <Divider />
      <ScrollArea flex="1" viewportRef={viewportRef} px="lg">
        {conversations.map((conversation: Conversation) => {
          const sentByUser = conversation.user_id === user.id;

          return (
            <Flex
              direction="column"
              w="100%"
              align={sentByUser ? 'flex-end' : 'flex-start'}
              key={conversation.id}
              p="sm"
            >
              <div>
                <Text component="span" c="gray.8" lh="20px" fz="sm">
                  {userHash[conversation.user_id].username}
                </Text>
                <Text ml="0.3rem" component="span" c="gray" lh="20px" fz="xs">
                  {formatDate(conversation.created_at)}
                </Text>
              </div>
              <Paper
                mt="xs"
                className={clsx(sentByUser ? 'bg-gradient-to-r from-sky-200 to-cyan-200' : 'bg-white')}
                radius="md"
                p="xs"
                shadow="md"
              >
                <Text c="gray.9" fz="sm">
                  {conversation.message}
                </Text>
              </Paper>
            </Flex>
          );
        })}
      </ScrollArea>

      <Group w="100%" align="flex-start" p="md">
        <input type="hidden" name="type" value={ACTION_TYPES.SEND_MESSAGE} />
        <input type="hidden" name="room_id" value={id} />
        <Textarea
          onKeyDown={e => {
            if (e.key === 'Enter' && !e.shiftKey) {
              e.preventDefault();
              sendMessage();
            }
          }}
          flex="1"
          autosize
          name="message"
          value={message}
          onChange={e => setMessage(e.target.value)}
        />
        <Button onClick={sendMessage}>Send</Button>
      </Group>
    </Stack>
  );
}
