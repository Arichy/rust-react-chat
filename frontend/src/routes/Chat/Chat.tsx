import { ActionIcon } from '@mantine/core';
import { IconCircleX } from '@tabler/icons-react';
import { useNavigate, useParams } from 'react-router-dom';

export default function Chat() {
  const roomId = useParams<{ id: string }>().id;

  const navigate = useNavigate();

  return (
    <div>
      <div>
        <ActionIcon
          onClick={() => {
            navigate('../');
          }}
        >
          <IconCircleX />
        </ActionIcon>
      </div>
    </div>
  );
}
