import { useAuthClient, useDataContext } from "@/utils";
import { useState, useCallback } from "react";
import { set } from "zod";
import { toasts } from "@/utils";

type WorkerTypes = {
  CAT: string;
  FAIRNESS: string;
  KALEIDOSCOPE: string;
};

type BaseWorkerData = {
  modelId: string;
  max_queries: number;
  seed: number;
};

type WorkerDataTypes = BaseWorkerData & (
  { shuffle: boolean } |
  { dataset: string[] } |
  { languages: string[] }
);

export function useWorker() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<any>(null);
  const { webapp } = useAuthClient();
  const { workerProcesses, setWorkerProcesses } = useDataContext();

  const runTest = useCallback((data: WorkerDataTypes, workerType: keyof WorkerTypes) => {
    // Check if test for this model is already running
    if (workerProcesses.includes(workerType)) {
      toasts.errrorToast(`${workerType} is already running.`);
      return;
    }

    setLoading(true);
    setError(null);

    // setWorkerProcesses([...workerProcesses, workerType]);

    // Create a new worker
    const worker = new Worker(new URL("./metricTestWorker.ts", import.meta.url), { type: "module" });

    worker.onmessage = async (event) => {
      const { type, payload, requestId } = event.data;

      if (type === "API_REQUEST") {
        try {
          // Execute the requested API call using webapp
          let result;
          switch (payload.method) {
            case "context_association_test":
              result = await webapp?.context_association_test(
                BigInt(payload.modelId),
                payload.max_queries,
                payload.seed,
                payload.shuffle,
                100
              );

              console.log("Result for context_association_test:", result);

              const catJobId = (result as { Ok: string })?.Ok;

              setWorkerProcesses([...workerProcesses, {
                type: workerType,
                jobId: catJobId,
              }]);

              break;
            case "fairness_test":
              const { modelId, max_queries, seed, dataset } = payload;

              console.log("Dataset:", dataset);

              for (const item of dataset) {
                try {
                  result = await webapp?.calculate_llm_metrics(
                    BigInt(modelId),
                    item,
                    max_queries,
                    seed,
                    100
                  );

                  console.log("Result for dataset item:", item, result);

                  const jobId = (result as { Ok: string })?.Ok;

                  setWorkerProcesses([...workerProcesses, {
                    type: workerType,
                    jobId: jobId,
                  }]);

                }
                catch (error) {
                  console.error("Error in calculate_llm_metrics:", error);
                  throw new Error("Failed to calculate LLM metrics.");
                }
              };

              try {
                result = await webapp?.average_llm_metrics(
                  BigInt(modelId),
                  dataset
                );
              }
              catch (error) {
                console.error("Error in average_llm_metrics:", error);
                throw new Error("Failed to calculate average LLM metrics.");
              }

              break;
            case "kaleidoscope_test":
              const { languages } = payload;

              result = await webapp?.llm_evaluate_languages(
                BigInt(payload.modelId),
                languages,
                payload.max_queries,
                payload.seed,

              );

              console.log("Result for context_association_test:", result);

              const jobId = (result as { Ok: string })?.Ok;

              setWorkerProcesses([...workerProcesses, {
                type: workerType,
                jobId: jobId,
              }]);


              break;
          }

          // Send result back to worker
          worker.postMessage({
            type: "API_RESPONSE",
            requestId,
            success: true,
            data: result
          });
        } catch (error: any) {
          console.error("Error in worker:", error);
          worker.postMessage({
            type: "API_RESPONSE",
            requestId,
            success: false,
            error: error.message
          });
        }
      } else if (type === "COMPLETE") {
        // Handle worker completion
        setLoading(false);
        if (payload.success) {
          setResult(payload.data);
          // toasts.successToast(`${workerType} completed successfully.`);
        } else {
          setError(payload.error);
          toasts.errrorToast(`Error in ${workerType}: ${payload.error}`);
        }
        worker.terminate();
        // Remove the completed worker process from the list
        setWorkerProcesses((prev: any[]) => prev.filter((item: any) => item.type !== workerType));
      }
    };

    // Send initial data to worker
    worker.postMessage({
      type: 'INIT',
      data: data,
      test: workerType,
    });

    toasts.successToast(`${workerType} is running.`);

    // Return a function to cancel the operation if needed
    return () => {
      worker.terminate();
      setLoading(false);
    };
  }, [workerProcesses, setWorkerProcesses]);

  return { runTest, loading, error, result, workerProcesses };
}