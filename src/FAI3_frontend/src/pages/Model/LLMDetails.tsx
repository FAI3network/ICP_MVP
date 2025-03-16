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
import { FAI3_backend } from "../../../../declarations/FAI3_backend";
import { useParams } from "react-router-dom";
import { useAuthClient } from "../../utils";
import { Principal } from "@dfinity/principal";
import { ContextAssociationTestMetricsBag, ContextAssociationTestMetrics } from "../../../../declarations/FAI3_backend/FAI3_backend.did";

export default function LLMDetails({ model, metrics, fetchModel }: any) {
  const { modelId } = useParams();
  const [loading, setLoading] = useState(false);
  const [isOwner, setIsOwner] = useState(false)
  const { address, webapp } = useAuthClient();
  const [editOrUpload, setEditOrUpload] = useState<string | null>(null);
  const latestVars = metrics[metrics.length - 1]?.AOD?.map((v: any) => v.variable_name);

  useEffect(() => {
    if (Object.keys(model).length === 0 || !address) {
      setIsOwner(false)
      return;
    };

    setIsOwner(model.owners.map((o: any) => Principal.fromUint8Array(o._arr).toString()).includes(address))

  }, [model, address])

  useEffect(() => {
    editOrUpload != null && openModal()
  }, [editOrUpload])

  const runCAT = async () => {
    setLoading(true);
    const res = await webapp?.context_association_test(BigInt(modelId!), 20, 1, false);
    console.log(res);
    setLoading(false);
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

                    {
                      editOrUpload === "edit" ? (
                        <AddModelModal onClose={() => setEditOrUpload(null)} modelId={parseInt(modelId!)} name={model.model_name} details={model.details} update fetchModel={fetchModel} />
                      ) : editOrUpload == "upload" ? (
                        <DataUploadModal fetchModel={fetchModel} latestVars={latestVars} onClose={() => setEditOrUpload(null)} />
                      ) : null
                    }


                    <Button onClick={() => { setEditOrUpload("edit") }}>
                      Edit Model
                    </Button>
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
                  <p>
                    <strong>Hugging Face URL:</strong>{" "}
                    {model.model_type?.LLM?.hugging_face_url}
                  </p>
                </div>
              </CardContent>
            </Card>
            {
              metrics.length > 0 && (
                <Card className="bg-[#fffaeb] h-full">
                  <CardHeader className="relative">
                    <CardTitle>Model Runs</CardTitle>
                    <CardDescription>
                      Detailed information about each model run.
                    </CardDescription>
                  </CardHeader>
                  <CardContent>
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
              )
            }
          </div>
          {
            metrics.length > 0 ? (
              <>
                {/* <div className="grid gap-8 lg:grid-cols-2">
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
                </div> */}
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