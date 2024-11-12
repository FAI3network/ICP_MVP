import { useEffect, useState } from "react";
import { ModelDetail } from "./ModelDetail";
import { useParams } from "react-router-dom";
import { FAI3_backend } from "../../../../declarations/FAI3_backend"

export default function Model() {
  const { modelId } = useParams();

  useEffect(() => {
    if (Number.isNaN(parseInt(modelId || ""))) {
      console.error("Invalid model ID");
      return;
    }

    let id = BigInt(modelId || "");

    //TODO: exception if id doesnt exist

    console.log(id);

    const fetchModel = async () => {
      const model = await FAI3_backend.get_model(id);
      // setModelWithDetails(model);
      console.log(model);
    };

    const fetchMetrics = async () => {
      const metrics = await FAI3_backend.get_model_metrics(id);
      // setMetrics(metrics);
      console.log(metrics);
    };

    fetchModel();
    fetchMetrics();
  }, [modelId]);

  const [modelWithDetails, setModelWithDetails] = useState({
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
  });
  const [metrics, setMetrics] = useState([
    {
        "timestamp": "2024-07-22",
        "SPD": -0.333,
        "DI": 0.375,
        "AOD": -0.417,
        "EOD": -0.667
    },
    {
        "timestamp": "2024-07-26",
        "SPD": -0.097,
        "DI": 0.799,
        "AOD": -0.098,
        "EOD": -0.095
    },
    {
        "timestamp": "2024-07-26",
        "SPD": -0.129,
        "DI": 0.735,
        "AOD": -0.112,
        "EOD": -0.067
    }
  ]);

  return (
    <div>
      {modelWithDetails && metrics.length ? (
        <div>
          {/* {console.log(modelWithDetails.data.name, metrics)} */}
          <ModelDetail model={modelWithDetails} metrics={metrics} />
        </div>
      ) : (
        <div className="w-full text-center">Loading...</div>
      )}
    </div>
  );
}