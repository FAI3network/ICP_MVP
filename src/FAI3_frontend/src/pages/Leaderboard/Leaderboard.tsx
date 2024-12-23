import LeaderboardTable from "./LeaderboardTable";
import { useEffect, useState } from "react";
import { FAI3_backend } from "../../../../declarations/FAI3_backend"
import { Model } from "../../../../declarations/FAI3_backend/FAI3_backend.did";
import { Button } from "../../components/ui";
import { Principal } from "@dfinity/principal";
import { useAuthClient } from "../../utils";

export default function Leaderboard() {
  const [modelsWithDetails, setModelsWithDetails] = useState<Model[]>([]);
  const [loading, setLoading] = useState(true);
  const { webapp, connected } = useAuthClient();

  useEffect(() => {
    fetchModels();
  }, [])

  const fetchModels = async () => {
    setLoading(true);
    // const models = await FAI3_backend.get_all_models();
    const models: Model[] = connected ? await (webapp?.get_all_models() as Promise<Model[]>) : await FAI3_backend.get_all_models();
    
    setModelsWithDetails(models);
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
        <LeaderboardTable models={modelsWithDetails} fetchModels={fetchModels} />
      )}
    </div>
  );
}