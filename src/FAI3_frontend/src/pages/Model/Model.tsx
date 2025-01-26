import { useEffect, useState } from "react";
import { ModelDetail } from "./ModelDetail";
import { useParams } from "react-router-dom";
import { useAuthClient } from "../../utils";
import { Model as ModelType } from "../../../../declarations/FAI3_backend/FAI3_backend.did";
import { FAI3_backend } from "../../../../declarations/FAI3_backend";

interface Metric {
  timestamp: string;
  SPD: number;
  DI: number;
  AOD: number;
  EOD: number;
}

export default function Model() {
  const { modelId } = useParams();
  const { webapp, connected } = useAuthClient();

  const [modelWithDetails, setModelWithDetails] = useState({});
  const [metrics, setMetrics] = useState([] as Metric[]);

  const fetchModel = async () => {
    let id = BigInt(modelId || "");
    // const model = await FAI3_backend.get_model(id);
    const model: ModelType = connected ? await (webapp?.get_model(id) as Promise<ModelType>) : await FAI3_backend.get_model(id);

    setModelWithDetails(model);

    const metricsHistory = model?.metrics_history;

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
        EOD: metric.equal_opportunity_difference[0],
        average: {
          SPD: metric.average_metrics.statistical_parity_difference[0],
          DI: metric.average_metrics.disparate_impact[0],
          AOD: metric.average_metrics.average_odds_difference[0],
          EOD: metric.average_metrics.equal_opportunity_difference[0]
        }
      });
    }

    setMetrics(metricsList);
    // console.log(metricsList);
  };

  useEffect(() => {
    if (Number.isNaN(parseInt(modelId || ""))) {
      console.error("Invalid model ID");
      return;
    }
    //TODO: exception if id doesnt exist
    // (async () => {
    //   const { model, metricsList } = await fetchModel(BigInt(modelId || ""));
    //   setModelWithDetails(model);
    //   setMetrics(metricsList);
    // })()

    fetchModel();
  }, [modelId]);


  return (
    <div>
      {modelWithDetails && Object.keys(modelWithDetails).length > 0 ? (
        <div>
          {/* {console.log(modelWithDetails.data.name, metrics)} */}
          <ModelDetail model={modelWithDetails} metrics={metrics} fetchModel={fetchModel} />
        </div>
      ) : (
        <div className="w-full text-center">Loading...</div>
      )}
    </div>
  );
}