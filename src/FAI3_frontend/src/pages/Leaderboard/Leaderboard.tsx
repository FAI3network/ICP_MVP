import LeaderboardTable from "./LeaderboardTable";
import { use, useEffect, useState } from "react";
import { FAI3_backend } from "../../../../declarations/FAI3_backend"
import { Model } from "../../../../declarations/FAI3_backend/FAI3_backend.did";
import { Button } from "../../components/ui";
import { Principal } from "@dfinity/principal";
import { useAuthClient, useDataContext } from "../../utils";


export default function Leaderboard() {
  const { webapp, connected } = useAuthClient();
  const { setModels, Models, fetchModels, LLMModels } = useDataContext();
  const [loading, setLoading] = useState(Models.length === 0);

  useEffect(() => {
    if (Models.length > 0) return;

    (async () => {
      setLoading(true);
      await fetchModels();
      setLoading(false);
    })();
  }, [])

  useEffect(() => {
    console.log(LLMModels);
  }, [LLMModels])

  return (
    <div className="mx-20">
      <div className="flex flex-col items-center justify-center mb-4">
        <h1 className="text-4xl font-bold text-center">
          Machine Learning Model Leaderboard
        </h1>

        <p className="mt-4 text-lg text-center text-gray-500">
          Compare the performance of different machine learning models.
        </p>
      </div>
      {loading ? (
        <div className="w-full text-center">Loading...</div>
      ) : (
        <LeaderboardTable />
      )}
    </div>
  );
}