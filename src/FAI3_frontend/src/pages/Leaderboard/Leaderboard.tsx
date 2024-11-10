import LeaderboardTable from "./LeaderboardTable";
import { useEffect, useState } from "react";
import { FAI3_backend } from "../../../../declarations/FAI3_backend"
import { Model } from "../../../../declarations/FAI3_backend/FAI3_backend.did";

export default function Leaderboard() {
  const [modelsWithDetails, setModelsWithDetails] = useState<Model[]>([]);

  useEffect(() => {
    const fetchModels = async () => {
      const models = await FAI3_backend.get_all_models();
      setModelsWithDetails(models);
      console.log(models);
    };
    fetchModels();
  }, [])

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
      {modelsWithDetails.length === 0 ? (
        <div className="w-full text-center">Loading...</div>
      ) : (
        <LeaderboardTable models={modelsWithDetails} />
      )}
    </div>
  );
}