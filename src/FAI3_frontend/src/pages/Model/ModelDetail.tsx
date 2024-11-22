import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  CardFooter,
  Button,
  Modal,
  ModalContent,
  ModalTrigger,
  ModalHeader,
  ModalBody,
  ModalTitle,
  ModalFooter,
  openModal,
  closeModal,
  Table,
  TableHeader,
  TableRow,
  TableHead,
  TableCell,
  TableBody
} from "../../components/ui";

import {
  LineChartchart,
  TabChart
} from "../../components/charts";

import { FileUpload } from "../../components";

import { useEffect, useMemo, useState } from "react";

import Papa from "papaparse";

import {
  flexRender,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table";

import {
  Trash2
} from "lucide-react";

import { FAI3_backend } from "../../../../declarations/FAI3_backend";

import { useParams } from "react-router-dom";

export function ModelDetail({ model, metrics }: any) {
  const [file, setFile] = useState<File | null>(null);
  const [uploadedData, setUploadedData] = useState<any[]>([]);
  const [uploadedColumns, setUploadedColumns] = useState<any[]>([]);
  const [showUploadedContent, setShowUploadedContent] = useState(false);
  const [imageData, setImageData] = useState<any[]>([{
    key: "",
    value: ""
  }]);

  const { modelId } = useParams();

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

  const handleFileUpload = () => {
    console.log("file type", file?.type);

    if (file?.type.includes("csv")) {
      Papa.parse(file as File, {
        header: true,
        complete: (result: Papa.ParseResult<any>) => {
          console.log(result);
          setUploadedData(result.data);
          createColumns(result.data);
        },
      });
    } else if (file?.type.includes("image")) {
      setShowUploadedContent(true);
    }
  }

  const createColumns = (data: any) => {
    const object = data[0];

    const columns = Object.keys(object).map((key, index) => {
      return {
        id: index.toString(),
        accessorKey: key,
        header: ({ column }: any) => {
          return (
            <div className="flex justify-center items-center">
              {key}
            </div>
          );
        },
        cell: ({ row }: any) => {
          const value = row.original[key];
          const parsed = parseFloat(value);
          return isNaN(parsed) ? (
            <div className="flex justify-center items-center">
              {value}
            </div>
          ) : (
            <div className="flex justify-center items-center">
              {Number.isInteger(parsed) ? parsed : parsed.toFixed(2)}
            </div>
          )
        },
      };
    });

    setUploadedColumns(columns);
  }

  const uploadedTable = useReactTable({
    data: uploadedData || [],
    columns: uploadedColumns || [],
    getCoreRowModel: getCoreRowModel(),
  });

  useEffect(() => {
    console.log(uploadedColumns);
    if (uploadedColumns.length) {
      console.log(uploadedData);

      setShowUploadedContent(true);
    }
  }, [uploadedColumns]);

  const closeFile = () => {
    setFile(null);
    setUploadedData([]);
    setUploadedColumns([]);
    setShowUploadedContent(false);
  }

  const uploadData = () => {
    //TODO: Implement storing data to smart contract

    if (file?.type.includes("csv")) {
      uploadDataSet();
    }

    // closeFile();
    // closeModal();
  }

  const uploadDataSet = () => {
    //For now only the test upload csv file works
    let dataByAtr: any= {};
    uploadedData.forEach((d) => {
      for (let key in d) {
        const parsed = parseFloat(d[key]);
        if (!isNaN(parsed)) {
          if (!dataByAtr[key]) {
            dataByAtr[key] = [];
          }
          dataByAtr[key].push(parsed);
        } 
      }
    });

    console.log(dataByAtr);

    const arg1 = [];

    for (let key in dataByAtr) {
      if (key == "Labels" || key == "Gender" || key == "Predictions") continue;
      arg1.push(dataByAtr[key]);
    }

    const arg2 = dataByAtr["Gender"].map((d : number) => d == 1 ? "Male" : "Female");

    const arg3 = dataByAtr["Labels"].map((d : number) => d == 1 ? true : false);

    const arg4 = dataByAtr["Predictions"].map((d : number) => d == 1 ? true : false);

    FAI3_backend.add_dataset(BigInt(modelId!), arg1, arg2, arg3, arg4);
  }

  return (
    <div className="grid min-h-screen w-full bg-white">
      {model && metrics && (
        <section className="grid gap-8 p-6 md:p-10">
          <div className="text-center relative">
            <h1 className="text-4xl font-bold pb-3">{model.name}</h1>
            <h3>
              Get a detailed overview of the model&apos;s architecture and
              performance.
            </h3>

            <div className="absolute top-1/2 right-0">
              <Modal onClose={closeFile}>
                <ModalTrigger>
                  Upload Data
                </ModalTrigger>
                {
                  showUploadedContent ? (
                    <ModalContent>
                      <ModalHeader>
                        <ModalTitle>{file?.name}</ModalTitle>
                      </ModalHeader>
                      <ModalBody>
                        <div className="flex w-full gap-2">
                          <Button onClick={uploadData}>
                            Upload
                          </Button>
                          <Button variant="secondary" onClick={closeFile}>
                            Use another file
                          </Button>
                        </div>
                        {
                          file?.type.includes("image") ? (
                            <div>
                              <img className="my-4" src={URL.createObjectURL(file)} alt="Uploaded" />
                              <div className="flex flex-col space-y-2 items-start">
                                <label className="text-sm font-medium">Data:</label>
                                <div className="flex items-center flex-col">
                                  {
                                    imageData.map((data, index) => (
                                      <div key={index} className="flex w-full my-2 items-center">
                                        <input
                                          type="text"
                                          placeholder="Key"
                                          value={data.key}
                                          onChange={(e) => {
                                            const value = e.target.value;
                                            setImageData(
                                              imageData.map((d, i) => {
                                                if (i === index) {
                                                  return {
                                                    ...d,
                                                    key: value
                                                  }
                                                }
                                                return d;
                                              })
                                            )
                                          }}
                                          className="w-1/4 p-1 text-gray-600 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                                        />
                                        <span className="text-lg mx-2">:</span>
                                        <input
                                          type="text"
                                          placeholder="Value"
                                          value={data.value}
                                          onChange={(e) => {
                                            const value = e.target.value;
                                            setImageData(
                                              imageData.map((d, i) => {
                                                if (i === index) {
                                                  return {
                                                    ...d,
                                                    value
                                                  }
                                                }
                                                return d;
                                              })
                                            )
                                          }}
                                          className="w-3/4 p-1 text-gray-600 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                                        />
                                        {
                                          imageData.length > 1 && (
                                            <Button
                                              variant="destructive"
                                              className="ml-2"
                                              onClick={() => setImageData(
                                                imageData.filter((_, i) => i !== index)
                                              )}
                                            >
                                              <Trash2 size={16} />
                                            </Button>
                                          )
                                        }

                                      </div>
                                    ))
                                  }
                                  <div className="flex w-full my-2">
                                    <Button
                                      variant="outline"
                                      onClick={() => setImageData([
                                        ...imageData,
                                        {
                                          key: "",
                                          value: ""
                                        }
                                      ])}
                                    >
                                      Add field
                                    </Button>
                                  </div>
                                </div>
                              </div>
                            </div>
                          ) : (
                            <Table className="overflow-scroll">
                              <TableHeader>
                                {uploadedTable.getHeaderGroups().map((headerGroup) => {
                                  console.log(headerGroup);
                                  return (
                                    <TableRow key={headerGroup.id}>
                                      <TableHead>#</TableHead>
                                      {headerGroup.headers.map((header) => (
                                        <TableHead key={header.id}>
                                          {header.isPlaceholder
                                            ? null
                                            : flexRender(
                                              header.column.columnDef.header,
                                              header.getContext()
                                            )}</TableHead>
                                      ))}
                                    </TableRow>
                                  )
                                })}
                              </TableHeader>
                              <TableBody>
                                {uploadedTable.getRowModel().rows?.length ? (
                                  uploadedTable.getRowModel().rows.map((row) => (
                                    <TableRow
                                      key={row.id}
                                      data-state={row.getIsSelected() && "selected"}
                                    >
                                      <TableCell>
                                        {row.index + 1}
                                      </TableCell>
                                      {row.getVisibleCells().map((cell) => (
                                        <TableCell key={cell.id}>
                                          {flexRender(
                                            cell.column.columnDef.cell,
                                            cell.getContext()
                                          )}
                                        </TableCell>
                                      ))}
                                    </TableRow>
                                  ))
                                ) : (
                                  <TableRow>
                                    <TableCell
                                      colSpan={uploadedColumns.length}
                                      className="h-24 text-center"
                                    >
                                      No results.
                                    </TableCell>
                                  </TableRow>
                                )}
                              </TableBody>
                            </Table>
                          )
                        }
                      </ModalBody>
                    </ModalContent>
                  ) : (
                    <ModalContent>
                      <ModalHeader>
                        <ModalTitle>Upload Data</ModalTitle>
                      </ModalHeader>
                      <ModalBody>
                        <p>Upload your data to retrain the model.</p>
                        <FileUpload onFileChange={setFile} accept=".csv, image/*" />
                      </ModalBody>
                      <ModalFooter>
                        <Button variant="secondary" onClick={closeModal}>Cancel</Button>
                        <Button onClick={handleFileUpload}>Next</Button>
                      </ModalFooter>
                    </ModalContent>
                  )
                }

              </Modal>
            </div>
          </div>
          <div className="grid gap-8 lg:grid-cols-2 lg:h-[500px]">
            <Card className="bg-[#fffaeb]">
              <CardHeader className="">
                <CardTitle>Model Details</CardTitle>
                <CardDescription className="text-md">
                  {model.description}
                </CardDescription>
              </CardHeader>
              <CardContent className="grid grid-cols-1 gap-8 max-h-96">
                <div className="grid gap-4 h-fit text-lg">
                  <p>
                    <strong>Framework:</strong> {model.framework}
                  </p>
                  <p>
                    <strong>Version:</strong> {model.version}
                  </p>
                  <p>
                    <strong>Size:</strong> {model.size}
                  </p>
                  <p>
                    <strong>Accuracy:</strong> {model.accuracy}
                  </p>
                  <p>
                    <strong>Objective:</strong>{" "}
                    {model.hyperparameters.objective}
                  </p>
                </div>
              </CardContent>
            </Card>
            <TabChart chartData={metrics} />
          </div>
          <Card className="bg-[#fffaeb]">
            <CardHeader>
              <CardTitle>Model Performance Summary</CardTitle>
              <CardDescription>
                Key metrics for the latest model run.
              </CardDescription>
            </CardHeader>
            <CardContent className="grid grid-cols-3 gap-6">
              <div className="flex flex-col items-center gap-2">
                <div className="text-4xl font-bold">0.92</div>
                <div className="text-muted-foreground">Accuracy</div>
              </div>
              <div className="flex flex-col items-center gap-2">
                <div className="text-4xl font-bold">0.88</div>
                <div className="text-muted-foreground">Precision</div>
              </div>
              <div className="flex flex-col items-center gap-2">
                <div className="text-4xl font-bold">0.94</div>
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
                    (max: any, p: any) => (p.SPD > max ? p.SPD : max),
                    metrics[0].SPD
                  )}
                  minVal={metrics.reduce(
                    (min: any, p: any) => (p.SPD < min ? p.SPD : min),
                    metrics[0].SPD
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
                    (max: any, p: any) => (p.DI > max ? p.DI : max),
                    metrics[0].DI
                  )}
                  minVal={metrics.reduce(
                    (min: any, p: any) => (p.DI < min ? p.DI : min),
                    metrics[0].DI
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
                    (max: any, p: any) => (p.AOD > max ? p.AOD : max),
                    metrics[0].AOD
                  )}
                  minVal={metrics.reduce(
                    (min: any, p: any) => (p.AOD < min ? p.AOD : min),
                    metrics[0].AOD
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
                    (max: any, p: any) => (p.EOD > max ? p.EOD : max),
                    metrics[0].EOD
                  )}
                  minVal={metrics.reduce(
                    (min: any, p: any) => (p.EOD < min ? p.EOD : min),
                    metrics[0].EOD
                  )}
                />
              </CardContent>
              <CardFooter className="flex flex-col text-sm ">
                <p>Unfair outcome: {chartConfig.EOD.footer.unfair}</p>
                <p>Fair outcome: {chartConfig.EOD.footer.unfair}</p>
              </CardFooter>
            </Card>
          </div>
        </section>
      )}
    </div>
  );
}