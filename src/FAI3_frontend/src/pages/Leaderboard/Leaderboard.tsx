import LeaderboardTable from "./LeaderboardTable";
import { useEffect, useState } from "react";

export default function Leaderboard() {
  const [modelsWithDetails, setModelsWithDetails] = useState([{
        "modelURI": "data:application/json;base64,ewogICJuYW1lIjogIkNyZWRpdCBTY29yaW5nIFhnYm9vc3QgTW9kZWwiLAogICJkZXNjcmlwdGlvbiI6ICJBbiBYZ2Jvb3N0LWJhc2VkIG1hY2hpbmUgbGVhcm5pbmcgbW9kZWwgZm9yIGNyZWRpdCBzY29yaW5nIGFwcGxpY2F0aW9ucy4iLAogICJpbWFnZVVSTCI6ICJodHRwczovL2V4YW1wbGUuY29tL2NyZWRpdF9zY29yaW5nX3hnYm9vc3QucG5nIiwKICAiZnJhbWV3b3JrIjogIlhnYm9vc3QiLAogICJ2ZXJzaW9uIjogIjEuMCIsCiAgImh5cGVycGFyYW1ldGVycyI6IHsKICAgICJtYXhfZGVwdGgiOiA1LAogICAgImxlYXJuaW5nX3JhdGUiOiAwLjA1LAogICAgIm5fZXN0aW1hdG9ycyI6IDIwMCwKICAgICJvYmplY3RpdmUiOiAiYmluYXJ5OmxvZ2lzdGljIgogIH0sCiAgInRyYWluZWRfb24iOiAiaHR0cHM6Ly9hcmNoaXZlLmljcy51Y2kuZWR1L2RhdGFzZXQvMTQ0L3N0YXRsb2crZ2VybWFuK2NyZWRpdCtkYXRhIiwKICAiZGVwbG95ZWRfd2l0aCI6ICJLdWJlcm5ldGVzIGNsdXN0ZXIiLAogICJjcmVhdGVkX2J5IjogIkZpbmFuY2VNTENvIiwKICAiZGF0ZV9jcmVhdGVkIjogIjIwMjMtMTAtMTUiCn0K",
        "metrics": [
          -0.128571428571428572,
          0.735294117647058822,
          -0.111515151515151515,
          -0.066666666666666667
        ],
        "numberOfInferences": 49,
        "owner": "0x89d3efe04c3ba4d0d06e7ab7c08ff9e0a6cc914a",
        "verifier": "0xbcc74b3cb8f05f3c58f1efa884151822ec4beb4a",
        "data": {
          "name": "Credit Scoring Xgboost Model",
          "description": "An Xgboost-based machine learning model for credit scoring applications.",
          "imageURL": "https://example.com/credit_scoring_xgboost.png",
          "framework": "Xgboost",
          "version": "1.0",
          "hyperparameters": {
            "max_depth": 5,
            "learning_rate": 0.05,
            "n_estimators": 200,
            "objective": "binary:logistic"
          },
          "trained_on": "https://archive.ics.uci.edu/dataset/144/statlog+german+credit+data",
          "deployed_with": "Kubernetes cluster",
          "created_by": "FinanceMLCo",
          "date_created": "2023-10-15"
        }
      },
    ]);


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