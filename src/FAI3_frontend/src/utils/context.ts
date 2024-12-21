import { createContext } from "react";
import { AuthClient } from "@dfinity/auth-client";

// interface AuthClientContext {
//   client: AuthClient;
// }

export const authClientContext = createContext<AuthClient | null>(null);