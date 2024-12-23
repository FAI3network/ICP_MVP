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

export const authClientContext = createContext<AuthClientContext | undefined>(undefined);

export const useAuthClient = () => {
  const context: AuthClientContext | undefined = useContext(authClientContext);
  if (context === undefined) {
    throw new Error("useAuthClient must be used within an AuthClientProvider");
  }
  return context;
};