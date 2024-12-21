import { Link } from "react-router-dom";
import { Actor, HttpAgent } from "@dfinity/agent";
import { canisterId, idlFactory } from "../../../declarations/FAI3_backend";
import { useEffect, useState, useContext } from "react";
import { Button } from "./ui";
import { authClientContext, formatAddress } from "../utils";

export default function Navbar() {
  const [iiUrl, setIiUrl] = useState<string | URL | undefined>();
  const [webapp, setWebapp] = useState<Actor>();
  const authClient = useContext(authClientContext);

  useEffect(() => {
    let url: string;
    if (process.env.DFX_NETWORK === "local") {
      url = `http://${process.env.CANISTER_ID_INTERNET_IDENTITY}.localhost:4943/`;
    } else if (process.env.DFX_NETWORK === "ic") {
      url = `https://${process.env.CANISTER_ID_INTERNET_IDENTITY}.ic0.app`;
    } else {
      url = `https://${process.env.CANISTER_ID_INTERNET_IDENTITY}.dfinity.network`;
    }
    setIiUrl(url);
    console.log(url);
  }, []);

  const connect = async () => {
    if (!authClient) return;

    await new Promise((resolve, reject) => {
      authClient.login({
        identityProvider: iiUrl,
        onSuccess: resolve,
        onError: reject,
      });
    })

    const identity = authClient.getIdentity();
    const agent = HttpAgent.createSync({ identity });

    const webapp = Actor.createActor(idlFactory, {
      agent,
      canisterId: canisterId,
    });

    setWebapp(webapp);
  }

  return (
    <nav className="flex justify-between mx-10 mb-12 mt-[1.5rem] items-center">
      <h1 className="text-2xl">
        <Link to={"/"}>FAI3</Link>
      </h1>
      <ul className="flex gap-12 items-center">
        <li>
          <Link to="/">Leaderboard</Link>
        </li>
        <li>
          <Link to="/">About</Link>
        </li>
        <li className="border border-gray-300 rounded-md">
          <div className="flex items-center">
            {
              webapp && authClient ? (
                <>
                  <p className="text-sm mx-2">
                    {formatAddress(authClient.getIdentity().getPrincipal().toText())}
                  </p>
                  <Button onClick={async () => {
                    await authClient.logout();
                    setWebapp(undefined);
                  }}>Logout</Button>
                </>
              ) : (
                <Button onClick={connect}>Connect</Button>
              )
            }
          </div>

        </li>
      </ul>
    </nav>
  );
}