import { Link } from "react-router-dom";

export default function Navbar() {
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
        <li>
          <Link to="/">Connect</Link>
        </li>
      </ul>
    </nav>
  );
}