import { createContext, useContext } from "react";
import { AuthClient } from "@dfinity/auth-client";
import { ActorSubclass, ActorMethod } from "@dfinity/agent";

interface AuthClientContext {
  authClient: AuthClient | undefined;
  address: string;
  connect: () => void;
  disconnect: () => void;
  webapp: ActorSubclass<Record<string, ActorMethod<unknown[], unknown>>> | undefined;
  connected: boolean;
}

export const AuthClientContext = createContext<AuthClientContext | undefined>(undefined);

export const useAuthClient = () => {
  const context: AuthClientContext | undefined = useContext(AuthClientContext);
  if (context === undefined) {
    throw new Error("useAuthClient must be used within an AuthClientProvider");
  }
  return context;
};

export const DataContext = createContext<any | undefined>(undefined);

export const useDataContext = () => {
  const context: any = useContext(DataContext);
  if (context === undefined) {
    throw new Error("useDataContext must be used within a DataContextProvider");
  }
  return context;
}