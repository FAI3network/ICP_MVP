import { Link } from "react-router-dom";
import { Actor, HttpAgent } from "@dfinity/agent";
import { canisterId, idlFactory } from "../../../declarations/FAI3_backend";
import { useEffect, useState, useContext } from "react";
import { Button } from "./ui";
import { useAuthClient, formatAddress } from "../utils";

export default function Navbar() {
  const { authClient, address, webapp, connect, disconnect } = useAuthClient();

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
                    {formatAddress(address)}
                  </p>
                  <Button onClick={disconnect}>Logout</Button>
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