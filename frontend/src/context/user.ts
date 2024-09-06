import { createContext, useContext } from 'react';
import { User } from '@types';

const UserContext = createContext<User | null>(null);

export function useUser() {
  return useContext(UserContext);
}

export const UserProvider = UserContext.Provider;
