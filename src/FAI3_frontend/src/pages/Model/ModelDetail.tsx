import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  CardFooter,
  Button,
  openModal,
} from "../../components/ui";
import {
  LineChartchart,
  TabChart
} from "../../components/charts";
import { DataUploadModal } from "../../components";
import { useState, useEffect, useContext } from "react";
import { FAI3_backend } from "../../../../declarations/FAI3_backend";
import { useParams } from "react-router-dom";
import { useAuthClient } from "../../utils";
import { Principal } from "@dfinity/principal";

export function ModelDetail({ model, metrics, fetchModel }: any) {
  const { modelId } = useParams();
  const [loading, setLoading] = useState(false);
  const [isOwner, setIsOwner] = useState(false)
  const { address, webapp } = useAuthClient();
  const latestVars = metrics[metrics.length-1]?.AOD?.map((v: any) => v.variable_name);

  useEffect(() => {
    if (Object.keys(model).length === 0 || !address) {
      setIsOwner(false)
      return;
    };

    if (Principal.fromUint8Array(model.user_id._arr).toString() == address) {
      setIsOwner(true)

    }
  }, [model, address])

  const chartConfig = {
    SPD: {
      label: "Statistical Parity Difference",
      color: "#2563eb",
      description:
        "The statistical parity difference measures the difference in the positive outcome rates between the unprivileged group and the privileged group.",
      footer: {
        unfair: "SPD significantly different from 0 (e.g., -0.4 or 0.4)",
        fair: "SPD close to 0 (e.g., -0.1 to 0.1)",
      },
      fairRange: [-0.1, 0.1],
      unfairRange: [-0.4, 0.4],
    },
    DI: {
      label: "Disparate Impact",
      color: "#60a5fa",
      description:
        "Disparate impact compares the ratio of the positive outcome rates between the unprivileged group and the privileged group.",
      footer: {
        unfair:
          "DI significantly different from 1 (e.g., less than 0.8 or greater than 1.25)",
        fair: "DI close to 1 (e.g., 0.8 to 1.25)",
      },
      fairRange: [0.8, 1.25],
      unfairRange: [0.8, 1.25],
    },
    AOD: {
      label: "Average Odds Difference",
      color: "#10b981",
      description:
        "The average odds difference measures the difference in false positive rates and true positive rates between the unprivileged group and the privileged group.",
      footer: {
        fair: "AOD close to 0 (e.g., -0.1 to 0.1)",
        unfair: "AOD significantly different from 0 (e.g., -0.2 or 0.2)",
      },
      fairRange: [-0.1, 0.1],
      unfairRange: [-0.2, 0.2],
    },
    EOD: {
      label: "Equal Opportunity Difference",
      color: "#f97316",
      description:
        "The equal opportunity difference measures the difference in true positive rates between the unprivileged group and the privileged group.",
      footer: {
        fair: "EOD close to 0 (e.g., -0.1 to 0.1)",
        unfair: "EOD significantly different from 0 (e.g., -0.2 or 0.2)",
      },
      unfairRange: [-0.2, 0.2],
      fairRange: [-0.1, 0.1],
    },
  };

  const teststat = async () => {
    // const res = await webapp?.add_dataset.inspect();
    // console.log(res)
  }

  return (
    <div className="grid min-h-screen w-full bg-white">
      <button onClick={teststat}>test</button>
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
                  <div className="w-full flex">
                    <Button onClick={openModal}>
                      Upload Data
                    </Button>
                    <DataUploadModal fetchModel={fetchModel} latestVars={latestVars} />
                  </div>
                </>
              )
            }

          </div>
          <div className="grid gap-8 lg:grid-cols-2 lg:h-[500px]">
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
                </div>
              </CardContent>
            </Card>
            {
              metrics.length > 0 && (
                <TabChart chartData={metrics} />
              )
            }
          </div>
          {
            metrics.length > 0 ? (
              <>
                <Card className="bg-[#fffaeb]">
                  <CardHeader>
                    <CardTitle>Model Performance Summary</CardTitle>
                    <CardDescription>
                      Key metrics for the latest model run.
                    </CardDescription>
                  </CardHeader>
                  <CardContent className="grid grid-cols-3 gap-6">
                    <div className="flex flex-col items-center gap-2">
                      <div className="text-4xl font-bold">
                        {
                          model.metrics ? (
                            Number(model.metrics.accuracy).toFixed(2)
                          ) : (
                            "N/A"
                          )
                        }
                      </div>
                      <div className="text-muted-foreground">Accuracy</div>
                    </div>
                    <div className="flex flex-col items-center gap-2">
                      <div className="text-4xl font-bold">
                        {
                          model.metrics ? (
                            Number(model.metrics.precision).toFixed(2)
                          ) : (
                            "N/A"
                          )
                        }
                      </div>
                      <div className="text-muted-foreground">Precision</div>
                    </div>
                    <div className="flex flex-col items-center gap-2">
                      <div className="text-4xl font-bold">
                        {
                          model.metrics ? (
                            Number(model.metrics.recall).toFixed(2)
                          ) : (
                            "N/A"
                          )
                        }
                      </div>
                      <div className="text-muted-foreground">Recall</div>
                    </div>
                  </CardContent>
                </Card>
                <div className="grid gap-8 lg:grid-cols-2">
                  <Card className="bg-[#fffaeb]">
                    <CardHeader>
                      <CardTitle>{chartConfig.SPD.label}</CardTitle>
                      <CardDescription>{chartConfig.SPD.description}</CardDescription>
                    </CardHeader>
                    <CardContent>
                      <LineChartchart
                        dataKey="SPD"
                        label={chartConfig.SPD.label}
                        color={chartConfig.SPD.color}
                        chartData={metrics}
                        unfairRange={chartConfig.SPD.unfairRange}
                        maxVal={metrics.reduce(
                          (max: any, p: any) => (p.average.SPD > max ? p.average.SPD : max),
                          metrics[0]?.average.SPD
                        )}
                        minVal={metrics.reduce(
                          (min: any, p: any) => (p.average.SPD < min ? p.average.SPD : min),
                          metrics[0]?.average.SPD
                        )}
                      />
                    </CardContent>
                    <CardFooter className="flex flex-col text-sm">
                      <p>Unfair outcome: {chartConfig.SPD.footer.unfair}</p>
                      <p>Fair outcome: {chartConfig.SPD.footer.unfair}</p>
                    </CardFooter>
                  </Card>
                  <Card className="bg-[#fffaeb]">
                    <CardHeader>
                      <CardTitle>{chartConfig.DI.label}</CardTitle>
                      <CardDescription>{chartConfig.DI.description}</CardDescription>
                    </CardHeader>
                    <CardContent>
                      <LineChartchart
                        dataKey="DI"
                        label={chartConfig.DI.label}
                        color={chartConfig.DI.color}
                        chartData={metrics}
                        unfairRange={chartConfig.DI.unfairRange}
                        maxVal={metrics.reduce(
                          (max: any, p: any) => (p.average.DI > max ? p.average.DI : max),
                          metrics[0]?.average.DI
                        )}
                        minVal={metrics.reduce(
                          (min: any, p: any) => (p.average.DI < min ? p.average.DI : min),
                          metrics[0]?.average.DI
                        )}
                      />
                    </CardContent>
                    <CardFooter className="flex flex-col text-sm">
                      <p>Unfair outcome: {chartConfig.DI.footer.unfair}</p>
                      <p>Fair outcome: {chartConfig.DI.footer.unfair}</p>
                    </CardFooter>
                  </Card>
                  <Card className="bg-[#fffaeb]">
                    <CardHeader>
                      <CardTitle>{chartConfig.AOD.label}</CardTitle>
                      <CardDescription>{chartConfig.AOD.description}</CardDescription>
                    </CardHeader>
                    <CardContent>
                      <LineChartchart
                        dataKey="AOD"
                        label={chartConfig.AOD.label}
                        color={chartConfig.AOD.color}
                        chartData={metrics}
                        unfairRange={chartConfig.AOD.unfairRange}
                        maxVal={metrics.reduce(
                          (max: any, p: any) => (p.average.AOD > max ? p.average.AOD : max),
                          metrics[0]?.average.AOD
                        )}
                        minVal={metrics.reduce(
                          (min: any, p: any) => (p.average.AOD < min ? p.average.AOD : min),
                          metrics[0]?.average.AOD
                        )}
                      />
                    </CardContent>
                    <CardFooter className="flex flex-col text-sm">
                      <p>Unfair outcome: {chartConfig.AOD.footer.unfair}</p>
                      <p>Fair outcome: {chartConfig.AOD.footer.unfair}</p>
                    </CardFooter>
                  </Card>
                  <Card className="bg-[#fffaeb]">
                    <CardHeader>
                      <CardTitle>{chartConfig.EOD.label}</CardTitle>
                      <CardDescription>{chartConfig.EOD.description}</CardDescription>
                    </CardHeader>
                    <CardContent>
                      <LineChartchart
                        dataKey="EOD"
                        label={chartConfig.EOD.label}
                        color={chartConfig.EOD.color}
                        chartData={metrics}
                        unfairRange={chartConfig.EOD.unfairRange}
                        maxVal={metrics.reduce(
                          (max: any, p: any) => (p.average.EOD > max ? p.average.EOD : max),
                          metrics[0]?.average.EOD
                        )}
                        minVal={metrics.reduce(
                          (min: any, p: any) => (p.average.EOD < min ? p.average.EOD : min),
                          metrics[0]?.average.EOD
                        )}
                      />
                    </CardContent>
                    <CardFooter className="flex flex-col text-sm ">
                      <p>Unfair outcome: {chartConfig.EOD.footer.unfair}</p>
                      <p>Fair outcome: {chartConfig.EOD.footer.unfair}</p>
                    </CardFooter>
                  </Card>
                </div>
              </>
            ) : (
              <div className="w-full text-center">No metrics available</div>
            )
          }

        </section>
      )}
    </div>
  );
}