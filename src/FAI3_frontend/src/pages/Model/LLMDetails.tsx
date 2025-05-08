import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  CardFooter,
  Button,
  openModal,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow
} from "../../components/ui";
import {
  LineChartchart,
  TabChart
} from "../../components/charts";
import { DataUploadModal, AddModelModal } from "../../components";
import { useState, useEffect, useContext } from "react";
import { useParams } from "react-router-dom";
import { useAuthClient, useDataContext, toasts } from "../../utils";
import { Principal } from "@dfinity/principal";
import { ContextAssociationTestMetricsBag, ContextAssociationTestMetrics, GenericError } from "../../../../declarations/FAI3_backend/FAI3_backend.did";

export default function LLMDetails({ model, metrics, fetchModel }: any) {
  const { modelId } = useParams();
  const [loading, setLoading] = useState(false);
  const [isOwner, setIsOwner] = useState(false)
  const { address, webapp } = useAuthClient();
  const { fetchModels } = useDataContext();


  useEffect(() => {
    if (Object.keys(model).length === 0 || !address) {
      setIsOwner(false)
      return;
    };

    setIsOwner(model.owners.map((o: any) => Principal.fromUint8Array(o._arr).toString()).includes(address))

  }, [model, address])

  const runCAT = async () => {
    setLoading(true);
    const res = await webapp?.context_association_test(BigInt(modelId!), 5, 1, false, 0);

    if (res && typeof res === 'object' && res !== null && 'Err' in res) {
      console.error("Failed to run context association test:", res.Err);
      const err = res.Err as GenericError;
      toasts.genericErrorToast(err);
    } else {
      console.log(res);
    }
    fetchModel();
    fetchModels();
    setLoading(false);
  }

  const generalChartConfig = {
    general_lms: {
      label: "General Lms",
      color: "#2563eb",
      description:
        "general_lms",
      key: "general_lms"
    },
    general_n: {
      label: "General N",
      color: "#60a5fa",
      description:
        "general_n",
      key: "general_n"
    },
    general_ss: {
      label: "General SS",
      color: "#10b981",
      description:
        "general_ss",
      key: "general_ss"
    }
  };

  const icatChartConfig = {
    icat_score_gender: {
      label: "ICAT Score Gender",
      color: "#2563eb",
      description: "icat_score_gender",
      key: "icat_score_gender"
    },
    icat_score_general: {
      label: "ICAT Score General",
      color: "#60a5fa",
      description: "icat_score_general",
      key: "icat_score_general"
    },
    icat_score_inter: {
      label: "ICAT Score Intersentence",
      color: "#10b981",
      description: "icat_score_inter",
      key: "icat_score_inter"
    },
    icat_score_intra: {
      label: "ICAT Score Intrasentence",
      color: "#f97316",
      description: "icat_score_intra",
      key: "icat_score_intra"
    },
    icat_score_profession: {
      label: "ICAT Score Profession",
      color: "#f59e0b",
      description: "icat_score_profession",
      key: "icat_score_profession"
    },
    icat_score_race: {
      label: "ICAT Score Race",
      color: "#bbf50b",
      description: "icat_score_race",
      key: "icat_score_race"
    },
    icat_score_religion: {
      label: "ICAT Score Religion",
      color: "#f5e20b",
      description: "icat_score_religion",
      key: "icat_score_religion"
    }
  }

  return (
    <div className="grid min-h-screen w-full bg-white">
      {
        loading && (
          <div className="w-full text-center">Loading...</div>
        )
      }
      {model && metrics && !loading && (
        <section className="grid gap-8 p-6 md:p-10">
          <div className="text-center relative w-full">
            <h1 className="text-4xl font-bold pb-3">{model.model_name}</h1>
            <h3>
              Get a detailed overview of the model&apos;s architecture and
              performance.
            </h3>

            {
              isOwner && (
                <>
                  <div className="w-full flex justify-between">
                    <Button onClick={runCAT}>
                      Run test
                    </Button>

                    <AddModelModal modelId={parseInt(modelId!)} name={model.model_name} details={model.details} update fetchModel={fetchModel} is_llm hf_url={model.model_type.LLM.hugging_face_url} />

                    <Button onClick={openModal}>
                      Edit Model
                    </Button>
                  </div>
                </>
              )
            }

          </div>
          <div className="grid gap-8 lg:grid-cols-2">
            <Card className="bg-[#fffaeb]">
              <CardHeader className="">
                <CardTitle>Model Details</CardTitle>
                <CardDescription className="text-md">
                  {model.details?.description}
                </CardDescription>
              </CardHeader>
              <CardContent className="grid grid-cols-1 gap-8 max-h-96">
                <div className="grid gap-4 h-fit text-lg">
                  <p>
                    <strong>Framework:</strong> {model.details?.framework}
                  </p>
                  <p>
                    <strong>Version:</strong> {model.details?.version}
                  </p>
                  <p>
                    <strong>Size:</strong> {model.details?.size}
                  </p>
                  <p>
                    <strong>Accuracy:</strong> {model.details?.accuracy}
                  </p>
                  <p>
                    <strong>Objective:</strong>{" "}
                    {model.details?.objective}
                  </p>
                  <p>
                    <strong>URL:</strong>{" "}
                    {model.details?.url}
                  </p>
                  <p>
                    <strong>Hugging Face URL:</strong>{" "}
                    {model.model_type?.LLM?.hugging_face_url}
                  </p>
                </div>
              </CardContent>
            </Card>
            {
              metrics.length > 0 ? (
                <>
                  <Card className="bg-[#fffaeb] h-full">
                    <CardHeader className="relative">
                      <CardTitle>Model Runs</CardTitle>
                      <CardDescription>
                        Detailed information about each model run.
                      </CardDescription>
                    </CardHeader>
                    <CardContent className="overflow-auto h-[350px]">
                      <Table>
                        <TableHeader>
                          <TableRow>
                            <TableHead>Name</TableHead>
                            <TableHead>Stereotype</TableHead>
                            <TableHead>Anti Stereotype</TableHead>
                            <TableHead>Neutral</TableHead>
                            <TableHead>Other</TableHead>
                            <TableHead>Date</TableHead>
                          </TableRow>
                        </TableHeader>
                        <TableBody>
                          {/* {chartData.map((row: any, index: number) => (
                          <TableRow key={index}>
                            <TableCell>{row.timestamp}</TableCell>
                            <TableCell>{row.average.SPD}</TableCell>
                            <TableCell>{row.average.DI}</TableCell>
                            <TableCell>{row.average.AOD}</TableCell>
                            <TableCell>{row.average.EOD}</TableCell>
                          </TableRow>
                        ))} */}
                          {
                            metrics.map((testResults: ContextAssociationTestMetricsBag, index: number) => {
                              let rows: any[] = [];


                              for (let i of ["gender", "general", "intersentence", "intrasentence", "profession", "race", "religion"]) {
                                const testResult: ContextAssociationTestMetrics = testResults[i as keyof Omit<ContextAssociationTestMetricsBag, 'timestamp'>] as ContextAssociationTestMetrics;
                                const row = (
                                  <TableRow key={index + i}>
                                    <TableCell>{i}</TableCell>
                                    <TableCell>{testResult.stereotype}</TableCell>
                                    <TableCell>{testResult.anti_stereotype}</TableCell>
                                    <TableCell>{testResult.neutral}</TableCell>
                                    <TableCell>{testResult.other}</TableCell>
                                    <TableCell>{new Date(Number(testResults.timestamp / 1000000n)).toLocaleDateString()}</TableCell>
                                  </TableRow>
                                )

                                rows.push(row)
                              }

                              return rows;
                            })
                          }
                        </TableBody>
                      </Table>
                    </CardContent>
                  </Card>

                  <TabChart chartConfig={generalChartConfig} chartData={metrics} />

                  <TabChart chartConfig={icatChartConfig} chartData={metrics} />
                </>
              ) : (
                <div className="w-full text-center">No metrics available</div>
              )
            }
          </div>
        </section>
      )}
    </div>
  );
}
