import { Outlet, redirect, useLoaderData } from 'react-router-dom';
import { UserProvider } from '../../context/user';
import { User } from '@types';

export async function loader() {
  const res = await fetch('/api/auth/user');
  if (res.status === 401) {
    return redirect('/auth');
  }
  const user = await res.json();

  return user;
}

export default function Home() {
  const data = useLoaderData() as User;

  return (
    <UserProvider value={data}>
      <Outlet />
    </UserProvider>
  );
}
