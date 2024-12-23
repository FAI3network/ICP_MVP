import { StrictMode, useState, useEffect } from 'react';
import ReactDOM from 'react-dom/client';
import './index.scss';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { Leaderboard, Model } from './pages';
import { Layout } from './components';
import { IDL as IDLType } from "@dfinity/candid";
import { authClientContext } from './utils';
import { AuthClient } from "@dfinity/auth-client";
import { HttpAgent, Actor, ActorSubclass, ActorMethod } from '@dfinity/agent';
import { idlFactory, canisterId } from '../../declarations/FAI3_backend';

function App() {
  const [webapp, setWebApp] = useState<ActorSubclass<Record<string, ActorMethod<unknown[], unknown>>> | undefined>();
  const [address, setAddress] = useState<string>("");
  const [authClient, setAuthClient] = useState<AuthClient | undefined>(undefined);
  const [connected, setConnected] = useState(false);

  useEffect(() => {
    (async () => {
      setAuthClient(await AuthClient.create());
    })();
  }, [])

  let iiUrl: string;
  if (process.env.DFX_NETWORK === "local") {
    iiUrl = `http://${process.env.CANISTER_ID_INTERNET_IDENTITY}.localhost:4943/`;
  } else if (process.env.DFX_NETWORK === "ic") {
    iiUrl = `https://${process.env.CANISTER_ID_INTERNET_IDENTITY}.ic0.app`;
  } else {
    iiUrl = `https://${process.env.CANISTER_ID_INTERNET_IDENTITY}.dfinity.network`;
  }

  const connect = async () => {
    if (!authClient) return;

    await new Promise((resolve, reject) => {
      authClient.login({
        identityProvider: iiUrl,
        onSuccess: resolve,
        onError: reject,
      });
    }).then(() => {
      console.log("Logged in!");
      setAddress(authClient.getIdentity().getPrincipal().toText());
    })

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

    setWebApp(webapp);
    setConnected(true);
  }

  const disconnect = () => {
    if (!authClient) return;

    authClient.logout();
    setAddress("");
    setWebApp(undefined);
    setConnected(false);
  }

  return (<StrictMode>
    <authClientContext.Provider value={{ authClient, address, connect, disconnect, webapp, connected }}>
      <Router>
        <Routes>
          <Route path="/" element={<Layout />} >
            <Route index element={<Leaderboard />} />
            <Route path="model/:modelId" element={<Model />} />
          </Route>
        </Routes>
      </Router>
    </authClientContext.Provider>
  </StrictMode>
  )
}

const rootElement = document.getElementById('root');
if (rootElement) {
  ReactDOM.createRoot(rootElement).render(
    <App />
  );
}