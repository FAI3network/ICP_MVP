import { useAuthClient } from "../utils";

export default function Whoami() {
  const { address } = useAuthClient();

  return (
    <div>
      <h1>Your address is: {address}</h1>
    </div>
  )
}