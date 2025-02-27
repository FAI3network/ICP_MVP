import { AuthClientContext, DataContext } from '../utils';
import { AuthClient } from "@dfinity/auth-client";
import { HttpAgent, Actor, ActorSubclass, ActorMethod } from '@dfinity/agent';
import { idlFactory, canisterId } from '../../../declarations/FAI3_backend';
import { useEffect, useState } from 'react';
import { LLMModel, Model } from '../../../declarations/FAI3_backend/FAI3_backend.did';
import { FAI3_backend } from '../../../declarations/FAI3_backend';

export default function Providers({ children }: { children: React.ReactNode }) {
  const [webapp, setWebApp] = useState<ActorSubclass<Record<string, ActorMethod<unknown[], unknown>>> | undefined>();
  const [address, setAddress] = useState<string>("");
  const [authClient, setAuthClient] = useState<AuthClient | undefined>(undefined);
  const [connected, setConnected] = useState(false);
  const [connecting, setConnecting] = useState(true);
  const [Models, setModels] = useState<Model[]>([]);
  const [LLMModels, setLLMModels] = useState<LLMModel[]>([]);
  const [isAdmin, setIsAdmin] = useState(false);

  useEffect(() => {
    (async () => {
      if (!authClient) {
        setAuthClient(await AuthClient.create());
      }
    })();
  }, [])

  useEffect(() => {
    if (authClient) {
      (async () => {
        if (await authClient.isAuthenticated()) {
          connect({ alreadyConnected: true });
          return;
        }
        setConnecting(false);
      })();
    }
  }, [authClient])

  let iiUrl: string;
  if (process.env.DFX_NETWORK === "local") {
    iiUrl = `http://${process.env.CANISTER_ID_INTERNET_IDENTITY}.localhost:4943/`;
  } else if (process.env.DFX_NETWORK === "ic") {
    iiUrl = `https://${process.env.CANISTER_ID_INTERNET_IDENTITY}.ic0.app`;
  } else {
    iiUrl = `https://${process.env.CANISTER_ID_INTERNET_IDENTITY}.dfinity.network`;
  }

  const connect = async ({ alreadyConnected = false }) => {
    if (!authClient) return;
    setConnecting(true);

    if (!alreadyConnected) {
      await new Promise((resolve, reject) => {
        authClient.login({
          identityProvider: iiUrl,
          onSuccess: resolve,
          onError: reject,
          maxTimeToLive: BigInt(7 * 24 * 60 * 60 * 1000 * 1000 * 1000)
        });
      }).catch((err) => {
        console.error(err);
        setConnecting(false);
        return;
      });
    }

    setAddress(authClient.getIdentity().getPrincipal().toText());
    const identity = authClient.getIdentity();
    // const agent = await HttpAgent.create({
    //   identity,
    // });

    const agent = new HttpAgent({ identity });

    agent.fetchRootKey().catch((err) => {
      console.log("Unable to fetch root key. Is the replica running?");
      console.error(err);
    });

    const webapp = Actor.createActor(idlFactory, {
      agent,
      canisterId: canisterId,
    });

    const is_admin: boolean | undefined = await (webapp?.is_admin() as Promise<boolean>).catch((err) => {
      console.error(err);
      return undefined;
    });

    if (is_admin === undefined) {
      disconnect();
      return;
    }

    setIsAdmin(is_admin);

    setWebApp(webapp);
    setConnected(true);
    setConnecting(false);
  }

  const disconnect = () => {
    if (!authClient) return;

    authClient.logout();
    setAddress("");
    setWebApp(undefined);
    setConnected(false);
  }

  const fetchModels = async () => {
    const models: Model[] = connected ?
      await (webapp?.get_all_models() as Promise<Model[]>)
      :
      await FAI3_backend.get_all_models().catch((err) => {
        console.error(err);
        return [];
      });

    setModels(models);

    const llmmodels: LLMModel[] = connected ?
      await (webapp?.get_all_llm_models() as Promise<LLMModel[]>)
      :
      await FAI3_backend.get_all_llm_models().catch((err) => {
        console.error(err);
        return [];
      }
    );

    setLLMModels(llmmodels);
  };

  return (
    <DataContext.Provider value={{ Models, setModels, fetchModels, LLMModels, setLLMModels }}>
      <AuthClientContext.Provider value={{ authClient, address, connect, disconnect, webapp, connected, isAdmin, connecting }}>
        {children}
      </AuthClientContext.Provider>
    </DataContext.Provider>
  );
}