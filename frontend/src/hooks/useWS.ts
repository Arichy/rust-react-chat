import { useEffect, useRef } from 'react';

export default function useWS() {
  const ws = useRef<WebSocket | null>(null);
  if (!ws.current) {
    ws.current = new WebSocket('/ws');
    ws.current.onopen = () => {
      console.log('Connected to WS');
    };
    ws.current.onclose = () => {
      console.log('Disconnected from WS');
    };
    ws.current.onerror = e => {
      console.error('WS error:', e);
    };
  }

  useEffect(() => {
    return () => {
      ws.current?.close();
    };
  }, []);

  return ws.current;
}
