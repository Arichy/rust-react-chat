import './App.css';
import { createBrowserRouter, RouterProvider } from 'react-router-dom';
import Home, { loader as rootLoader } from './routes/Home/Home';
import ChatRoom, { loader as chatRoomLoader, action as chatRoomAction } from './routes/ChatRoom/ChatRoom';
import Auth, { action as authAction } from './routes/Auth/Auth';
import { MantineProvider } from '@mantine/core';
import '@mantine/core/styles.css';
import Chat, {
  loader as ChatLoader,
  action as ChatAction,
  ErrorBoundary as ChatErrorBoundary,
} from './routes/Chat/Chat';
import ChatIndex from './routes/ChatIndex/ChatIndex';
import { ModalsProvider } from '@mantine/modals';
import { Notifications } from '@mantine/notifications';

const router = createBrowserRouter([
  {
    id: 'root',
    path: '/',
    Component: Home,
    loader: rootLoader,
    children: [
      {
        id: 'chat_room',
        path: '/',
        Component: ChatRoom,
        loader: chatRoomLoader,
        action: chatRoomAction,
        children: [
          {
            id: 'chat_index',
            path: '/',
            Component: ChatIndex,
            action: chatRoomAction,
          },
          {
            id: 'chat',
            path: '/room/:id',
            Component: Chat,
            ErrorBoundary: ChatErrorBoundary,
            loader: ChatLoader,
            action: ChatAction,
          },
        ],
      },
    ],
  },
  {
    id: 'auth',
    path: '/auth',
    Component: Auth,
    action: authAction,
  },
]);

export default function App() {
  return (
    <MantineProvider>
      <ModalsProvider>
        <Notifications />
          <RouterProvider router={router} />
      </ModalsProvider>
    </MantineProvider>
  );
}
