import { Title } from '@mantine/core';

export default function ChatIndex() {
  // a full page of  background linear gradient
  return (
    <div className="p-10 flex items-center justify-center h-full bg-gradient-to-br from-lime-200 to-sky-400">
      <Title order={2} c="white">
        Join a room or create a new room, then start talking happily!
      </Title>
    </div>
  );
}
