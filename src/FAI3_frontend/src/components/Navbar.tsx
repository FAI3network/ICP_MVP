import { Link } from "react-router-dom";
import { Actor, HttpAgent } from "@dfinity/agent";
import { canisterId, idlFactory } from "../../../declarations/FAI3_backend";
import { useEffect, useState, useContext } from "react";
import { Button, CircularProgress } from "./ui";
import { useAuthClient, formatAddress } from "../utils";

export default function Navbar() {
  const { authClient, address, webapp, connect, disconnect, connecting } = useAuthClient();

  const indicateCopied = () => {
    const tooltip = document.getElementById("tooltip");
    if (tooltip) {
      tooltip.style.opacity = "1";
      setTimeout(() => {
        tooltip.style.opacity = "0";
      }, 1000);
    }
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
        {
          connecting ? (
            <li>
              <CircularProgress />
            </li>
          ) : (
            <li className="border border-gray-300 rounded-md">
              <div className="flex items-center">
                {
                  webapp && authClient ? (
                    <>
                      <div className="relative group">
                        <p className="text-sm mx-2 cursor-pointer" onClick={() => {
                          navigator.clipboard.writeText(address);
                          indicateCopied();
                        }}>
                          {formatAddress(address)}
                        </p>
                        <span id="tooltip" className="absolute left-1/2 transform -translate-x-1/2 mb-2 px-2 py-1 text-xs text-white bg-black rounded opacity-0 transition-opacity duration-300">
                          Copied!
                        </span>
                      </div>
                      <Button onClick={disconnect}>Logout</Button>
                    </>
                  ) : (
                    <Button onClick={connect}>Connect</Button>
                  )
                }
              </div>
            </li>
          )
        }
      </ul>
    </nav>
  );
}