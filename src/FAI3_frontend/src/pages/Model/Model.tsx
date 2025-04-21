import { useEffect, useState } from "react";
import { ModelDetail } from "./ModelDetail";
import LLMDetails from "./LLMDetails";
import { useParams } from "react-router-dom";
import { useAuthClient } from "../../utils";
import { Model as ModelAsType, ClassifierModelData, LLMModelData, ContextAssociationTestMetricsBag, Metrics, ModelEvaluationResult } from "../../../../declarations/FAI3_backend/FAI3_backend.did";
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

  const [modelWithDetails, setModelWithDetails] = useState<ModelAsType>();
  const [metrics, setMetrics] = useState<Metric[] | ContextAssociationTestMetricsBag[]>([]);

  const fetchModel = async () => {
    let id = BigInt(modelId || "");
    // const model = await FAI3_backend.get_model(id);
    const model: ModelAsType = (connected ? await webapp?.get_model(id) : await FAI3_backend.get_model(id)) as ModelAsType;

    console.log(model);

    setModelWithDetails(model);

    const isClassifier = "Classifier" in model?.model_type ? true : false;

    const classifierData = (model?.model_type as { Classifier: ClassifierModelData }).Classifier;
    const metricsHistory = isClassifier ? (model?.model_type as { Classifier: ClassifierModelData }).Classifier?.metrics_history : (model?.model_type as { LLM: LLMModelData }).LLM?.evaluations;

    if (!Array.isArray(metricsHistory)) {
      console.error("Invalid metrics response");
      return;
    }

    const metricsList: any[] = [];

    for (let metric of metricsHistory) {
      if (!isClassifier) metric = (metric as ModelEvaluationResult).metrics as Metrics;

      // Ensure metric is treated as Metrics type
      const metricsData = metric as Metrics;

      metricsList.push({
        timestamp: metricsData.timestamp,
        SPD: metricsData.statistical_parity_difference[0],
        DI: metricsData.disparate_impact[0],
        AOD: metricsData.average_odds_difference[0],
        EOD: metricsData.equal_opportunity_difference[0],
        average: {
          SPD: metricsData.average_metrics.statistical_parity_difference[0],
          DI: metricsData.average_metrics.disparate_impact[0],
          AOD: metricsData.average_metrics.average_odds_difference[0],
          EOD: metricsData.average_metrics.equal_opportunity_difference[0],
        },
      });
    }

    setMetrics(metricsList);
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
    <>
      {modelWithDetails && Object.keys(modelWithDetails).length > 0 ? (
        "Classifier" in modelWithDetails?.model_type ? (
          <ModelDetail model={modelWithDetails} metrics={metrics} fetchModel={fetchModel} />
        ) : (
          <LLMDetails model={modelWithDetails} metrics={metrics} fetchModel={fetchModel} />
        )
      ) : (
        <div className="w-full text-center">Loading...</div>
      )}
    </>
  );
}
