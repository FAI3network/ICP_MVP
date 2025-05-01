import { Card, CardHeader, CardTitle, CardDescription, CardContent, CardFooter, Button, openModal } from "../../components/ui";
import { LineChartchart, TabChart, FairnessCharts } from "../../components/charts";
import { DataUploadModal, AddModelModal } from "../../components";
import { useState, useEffect, useContext } from "react";
import { FAI3_backend } from "../../../../declarations/FAI3_backend";
import { useParams } from "react-router-dom";
import { useAuthClient } from "../../utils";
import { Principal } from "@dfinity/principal";
import { fairnessConfig, prefixedFairnessConfig } from "@/configs";

export function ModelDetail({ model, metrics, fetchModel }: any) {
  const { modelId } = useParams();
  const [loading, setLoading] = useState(false);
  const [isOwner, setIsOwner] = useState(false);
  const { address, webapp } = useAuthClient();
  const [editOrUpload, setEditOrUpload] = useState<string | null>(null);
  const latestVars = metrics[metrics.length - 1]?.AOD?.map((v: any) => v.variable_name);
  const [openEvalModal, setOpenEvalModal] = useState(false);
  const [openEditModal, setOpenEditModal] = useState(false);

  useEffect(() => {
    if (Object.keys(model).length === 0 || !address) {
      setIsOwner(false);
      return;
    }

    setIsOwner(model.owners.map((o: any) => Principal.fromUint8Array(o._arr).toString()).includes(address));
  }, [model, address]);

  const teststat = async () => {
    // const res = await webapp?.add_dataset.inspect();
    // console.log(res)
  };

  useEffect(() => {
    editOrUpload != null && openModal();
  }, [editOrUpload]);

  return (
    <div className="grid min-h-screen w-full bg-white">
      {loading && <div className="w-full text-center">Loading...</div>}
      {model && metrics && !loading && (
        <section className="grid gap-8 p-6 md:p-10">
          <div className="text-center relative w-full">
            <h1 className="text-4xl font-bold pb-3">{model.model_name}</h1>
            <h3>Get a detailed overview of the model&apos;s architecture and performance.</h3>

            {isOwner && (
              <>
                <div className="w-full flex justify-between">
                  <Button
                    onClick={() => {
                      setOpenEvalModal(true);
                    }}
                  >
                    Evaluate
                  </Button>

                  <DataUploadModal fetchModel={fetchModel} latestVars={latestVars} cachedThresholds={model.cached_thresholds} cachedSelections={model.cached_selections} isOpen={openEvalModal} onClose={() => setOpenEvalModal(false)} />

                  <AddModelModal onClose={() => setOpenEditModal(false)} isOpen={openEditModal} modelId={parseInt(modelId!)} name={model.model_name} details={model.details} update fetchModel={fetchModel} modelInfo={{id: parseInt(modelId!), name:model.model_name, details: model.details}} />

                  <Button
                    onClick={() => {
                      setOpenEditModal(true);
                    }}
                  >
                    Edit Model
                  </Button>
                </div>
              </>
            )}
          </div>
          <div className="grid gap-8 lg:grid-cols-2 lg:h-[500px]">
            <Card className="bg-[#fffaeb]">
              <CardHeader className="">
                <CardTitle>Model Details</CardTitle>
                <CardDescription className="text-md">{model.details?.description}</CardDescription>
              </CardHeader>
              <CardContent className="grid grid-cols-1 gap-8 max-h-96">
                <div className="grid gap-4 h-fit text-lg">
                  <p>
                    <strong>Framework:</strong> {model.details?.framework}
                  </p>
                  <p>
                    <strong>Version:</strong> {model?.version}
                  </p>
                  <p>
                    <strong>Objective:</strong> {model.details?.objective}
                  </p>
                  <p>
                    <strong>URL:</strong> {model.details?.url}
                  </p>
                </div>
              </CardContent>
            </Card>
            {metrics.length > 0 && <TabChart chartData={metrics} chartConfig={fairnessConfig} />}
          </div>
          {metrics.length > 0 ? (
            <>
              <Card className="bg-[#fffaeb]">
                <CardHeader>
                  <CardTitle>Model Performance Summary</CardTitle>
                  <CardDescription>Key metrics for the latest model run.</CardDescription>
                </CardHeader>
                <CardContent className="grid grid-cols-3 gap-6">
                  <div className="flex flex-col items-center gap-2">
                    <div className="text-4xl font-bold">{model.model_type?.Classifier ? Number(model.model_type?.Classifier?.metrics.accuracy[0]).toFixed(2) : "N/A"}</div>
                    <div className="text-muted-foreground">Accuracy</div>
                  </div>
                  <div className="flex flex-col items-center gap-2">
                    <div className="text-4xl font-bold">{model.model_type?.Classifier ? Number(model.model_type?.Classifier?.metrics.precision[0]).toFixed(2) : "N/A"}</div>
                    <div className="text-muted-foreground">Precision</div>
                  </div>
                  <div className="flex flex-col items-center gap-2">
                    <div className="text-4xl font-bold">{model.model_type?.Classifier ? Number(model.model_type?.Classifier?.metrics.recall[0]).toFixed(2) : "N/A"}</div>
                    <div className="text-muted-foreground">Recall</div>
                  </div>
                </CardContent>
              </Card>
              <FairnessCharts metrics={metrics} />
            </>
          ) : (
            <div className="w-full text-center">No metrics available</div>
          )}
        </section>
      )}
    </div>
  );
}
