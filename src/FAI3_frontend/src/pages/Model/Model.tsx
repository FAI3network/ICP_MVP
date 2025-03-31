import { useEffect, useState } from "react";
import { ModelDetail } from "./ModelDetail";
import LLMDetails from "./LLMDetails";
import { useParams } from "react-router-dom";
import { useAuthClient } from "../../utils";
import { Model as ModelAsType, ClassifierModelData, LLMModelData, ContextAssociationTestMetricsBag } from "../../../../declarations/FAI3_backend/FAI3_backend.did";
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

    const isClassifier = 'Classifier' in model?.model_type ? true : false;

    if (isClassifier) {
      const classifierData = (model?.model_type as { Classifier: ClassifierModelData }).Classifier;
      const metricsHistory = classifierData?.metrics_history;

      if (!Array.isArray(metricsHistory)) {
        console.error("Invalid metrics response");
        return;
      }

      const metricsList: any[] = [];

      for (let metric of metricsHistory) {
        metricsList.push({
          timestamp: metric.timestamp,
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

      console.log(metricsList);

      setMetrics(metricsList);
    } else {
      const llmData = (model?.model_type as { LLM: LLMModelData }).LLM;


      setMetrics(llmData.cat_metrics_history);
    }
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
    <>
      {modelWithDetails && Object.keys(modelWithDetails).length > 0 ? (
        'Classifier' in modelWithDetails?.model_type ? (
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
