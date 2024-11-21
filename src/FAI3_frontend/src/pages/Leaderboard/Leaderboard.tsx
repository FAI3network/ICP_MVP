import LeaderboardTable from "./LeaderboardTable";
import { useEffect, useState } from "react";
import { FAI3_backend } from "../../../../declarations/FAI3_backend"
import { Model } from "../../../../declarations/FAI3_backend/FAI3_backend.did";

export default function Leaderboard() {
  const [modelsWithDetails, setModelsWithDetails] = useState<
  // Model[]
  any
  >([{
    "metrics": {
      "equal_opportunity_difference": [-0.128571428571428572],
      "statistical_parity_difference": [0.735294117647058822],
      "disparate_impact": [-0.111515151515151515],
      "average_odds_difference": [-0.066666666666666667]
    },
    "model_name": "Credit Scoring Xgboost Model",
    "numberOfInferences": 49,
    "user_id": "0x89d3efe04c3ba4d0d06e7ab7c08ff9e0a6cc914a",
    "model_id": 1,
  },
]);

  // useEffect(() => {
  //   const fetchModels = async () => {
  //     const models = await FAI3_backend.get_all_models();
  //     setModelsWithDetails(models);
  //     console.log(models);
  //   };
  //   fetchModels();
  // }, [])

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