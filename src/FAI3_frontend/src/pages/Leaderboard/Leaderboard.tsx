import LeaderboardTable from "./LeaderboardTable";
import { useEffect, useState } from "react";
import { FAI3_backend } from "../../../../declarations/FAI3_backend"
import { Model } from "../../../../declarations/FAI3_backend/FAI3_backend.did";
import { Button } from "../../components/ui";
import { Principal } from "@dfinity/principal";
import { useAuthClient, useDataContext } from "../../utils";

export default function Leaderboard() {
  const { webapp, connected } = useAuthClient();
  const { setModels, Models } = useDataContext();
  const [loading, setLoading] = useState(Models.length === 0);

  useEffect(() => {
    if (Models.length > 0) return;

    fetchModels();
  }, [])

  const fetchModels = async () => {
    console.log("fetching")
    setLoading(true);
    // const models = await FAI3_backend.get_all_models();
    console.log(connected);
    const models: Model[] = connected ?
      await (webapp?.get_all_models() as Promise<Model[]>)
      :
      await FAI3_backend.get_all_models().catch((err) => {
        console.error(err);
        return [];
      });

    setModels(models);
    console.log(models);
    setLoading(false);
  };

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
        <LeaderboardTable fetchModels={fetchModels} />
      )}
    </div>
  );
}