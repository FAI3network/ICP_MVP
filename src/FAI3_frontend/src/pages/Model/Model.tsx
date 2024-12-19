import { useEffect, useState } from "react";
import { ModelDetail } from "./ModelDetail";
import { useParams } from "react-router-dom";
import { FAI3_backend } from "../../../../declarations/FAI3_backend"

interface Metric {
  timestamp: string;
  SPD: number;
  DI: number;
  AOD: number;
  EOD: number;
}

export default function Model() {
  const { modelId } = useParams();

  const [modelWithDetails, setModelWithDetails] = useState({
    // "name": "Credit Scoring Xgboost Model",
    // "description": "An Xgboost-based machine learning model for credit scoring applications.",
    // "imageURL": "https://example.com/credit_scoring_xgboost.png",
    // "framework": "Xgboost",
    // "version": "1.0",
    // "hyperparameters": {
    //     "max_depth": 5,
    //     "learning_rate": 0.05,
    //     "n_estimators": 200,
    //     "objective": "binary:logistic"
    // },
    // "trained_on": "https://archive.ics.uci.edu/dataset/144/statlog+german+credit+data",
    // "deployed_with": "Kubernetes cluster",
    // "created_by": "FinanceMLCo",
    // "date_created": "2023-10-15"
  });
  const [metrics, setMetrics] = useState([] as Metric[]);

  useEffect(() => {
    if (Number.isNaN(parseInt(modelId || ""))) {
      console.error("Invalid model ID");
      return;
    }

    let id = BigInt(modelId || "");

    //TODO: exception if id doesnt exist

    const fetchModel = async () => {
      const model = await FAI3_backend.get_model(id);
      setModelWithDetails(model);

      const metricsHistory = model.metrics_history;

      if (!Array.isArray(metricsHistory)) {
        console.error("Invalid metrics response");
        return;
      }

      const metricsList: any[] = [];

      for (let metric of metricsHistory) {
        const timestamp = new Date(Number(metric.timestamp) / 1e6).toISOString().split('T')[0];

        metricsList.push({
          timestamp: timestamp,
          SPD: metric.statistical_parity_difference[0],
          DI: metric.disparate_impact[0],
          AOD: metric.average_odds_difference[0],
          EOD: metric.equal_opportunity_difference[0]
        });
      }

      setMetrics(metricsList);

      // console.log(model);
      // console.log(metricsList);
    };

    fetchModel();
  }, [modelId]);

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