import { ModalContent, ModalHeader, ModalTitle, ModalBody, ModalFooter, Button, closeModal, Select } from "../ui";
import { useState, useContext, useEffect } from "react";
import { FAI3_backend } from "../../../../declarations/FAI3_backend";
import { Table } from "@tanstack/react-table";
import { DataUploadContext } from "./utils";
import { useAuthClient, useDataContext } from "../../utils";
import { toast } from "sonner";
import { features } from "process";

export default function ColumnSelectionSection({ fetchModel, latestVars, cachedThresholds }: { fetchModel: () => Promise<any>, latestVars: any, cachedThresholds: any }) {
  const { modelId, table, columns, currentStep, setCurrentStep }: {
    modelId: string | undefined,
    table: Table<any>,
    columns: any[],
    currentStep: number,
    setCurrentStep: (step: number) => void
  } = useContext(DataUploadContext);
  const { webapp } = useAuthClient();
  const { fetchModels } = useDataContext();

  const [columnLabels, setColumnLabels] = useState<any>({
    labels: "",
    predictions: "",
    privledged: ""
  })
  const [loading, setLoading] = useState(false);
  const [openThresholdField, setOpenThresholdField] = useState(false);
  const [thresholds, setThresholds] = useState<{ varName: string, comparator: string, amount: number | null }[]>([]);

  useEffect(() => {
    console.log(columnLabels.privledged);
    if (columnLabels.privledged.length == 0) {
      setOpenThresholdField(false);
    }
  }, [columnLabels.privledged]);

  useEffect(() => {
    const tableColumns = table.getRowModel().rows.length
      ? Object.keys(table.getRowModel().rows[0].original)
      : [];

    const labelFilter = tableColumns.filter((col) => col.toLowerCase().includes("label"));
    const predictionFilter = tableColumns.filter((col) => col.toLowerCase().includes("prediction"));

    setColumnLabels({
      labels: labelFilter.length > 0 ? labelFilter[0] : "",
      predictions: predictionFilter.length > 0 ? predictionFilter[0] : "",
      privledged: latestVars && latestVars.length > 0 ? latestVars.join(", ") : ""
    });
  }, [table]);

  useEffect(() => {
    if (cachedThresholds && cachedThresholds.length > 0) {
      setThresholds(cachedThresholds[0].thresholds[0].map((thresholdDetails: any) => ({ varName: thresholdDetails[0], comparator: thresholdDetails[1][1] ? "greater" : "lower", amount: thresholdDetails[1][0] })));
    }
  }, [cachedThresholds]);

  useEffect(() => {
    console.log(thresholds);
  }, [thresholds]);

  useEffect(() => {
    if (columnLabels.privledged.length > 0 && cachedThresholds.length == 0) {
      setThresholds(columnLabels.privledged.split(", ").map((varName: string) => ({ varName, comparator: "greater", amount: null })));
    }
  }, [columnLabels.privledged]);

  const uploadData = async () => {
    setLoading(true);

    let labels: boolean[] = [];
    let predictions: boolean[] = [];
    let features: number[][] = [];

    const privledgedLabels = columnLabels.privledged.split(", ");

    const privilegedVariables: { key: string, value: bigint }[] = [];

    for (let i = 0; i < columns.length; i++) {
      if (columns[i].accessorKey === columnLabels.labels) {
        labels = table.getRowModel().rows.map((row) => (row.original[columnLabels.labels] == 1 ? true : false));
      } else if (columns[i].accessorKey === columnLabels.predictions) {
        predictions = table.getRowModel().rows.map((row) => (row.original[columnLabels.predictions] == 1 ? true : false));
      } else {
        features.push(table.getRowModel().rows.map((row) => parseFloat(row.original[columns[i].accessorKey])));
        if (privledgedLabels.includes(columns[i].accessorKey)) {
          privilegedVariables.push({ key: columns[i].accessorKey, value: BigInt(i) });
        }
      }
    }

    let valid = false;

    const thresholdValues = thresholds.map((threshold) => ([threshold.varName, [threshold.amount ?? calculateMedian(features[Number(privilegedVariables.find((priv) => priv.key === threshold.varName)?.value)]), threshold.comparator == "greater" ? true : false]]));

    for (const priv of privilegedVariables) {
      const threshold = thresholdValues.find((threshold) => threshold[0] === priv.key);
      const amount = threshold?.[1][0] as number | null;
      const comparator = threshold?.[1][1];

      const feats = features[Number(priv.value)];

      if (amount === null) {
        toast.error(`Something went wrong, please try again`);
      }

      if (amount! < Math.min(...feats) || amount! > Math.max(...feats)) {
        console.log("threshold out of range");
      } else if (comparator) {
        valid = feats.some((value) => value > amount!);
      } else {
        valid = feats.some((value) => value < amount!);
      }

      if (!valid) {
        toast.error(`Privileged variable ${priv.key} does not meet the threshold`);
      }

    }

    if (valid) {
      await webapp?.add_dataset(BigInt(modelId!), features, labels, predictions, privilegedVariables);
      await webapp?.calculate_all_metrics(BigInt(modelId!), [thresholdValues]);
      await fetchModel();
      await fetchModels();
      closeModal();
    }

    setLoading(false);
  }

  const calculateMedian = (features: number[]) => {
    const max = Math.max(...features);
    const min = Math.min(...features);

    return (max + min) / 2;
  }

  if (loading) {
    return (
      <ModalContent closeButton={false} className="h-1/4 w-1/4 flex justify-center items-center">
        <ModalBody>
          <h1 className="text-2xl font-semibold text-gray-800">Uploading Data...</h1>
        </ModalBody>
      </ModalContent>
    )
  }

  return (
    <ModalContent>
      <ModalHeader>
        <ModalTitle>Column Selection</ModalTitle>
      </ModalHeader>
      <ModalBody className="flex flex-col justify-start">
        <p>Select which columns of your dataset are predictions, labels, and privlidged.</p>
        <div className="flex flex-col gap-2 my-2">
          <div className="flex flex-row gap-2">
            <h3>
              Labels:
            </h3>
            <Select
              options={columns.map((col) => col.accessorKey)}
              selection={columnLabels.labels}
              setSelection={(selection: any) => setColumnLabels({
                ...columnLabels,
                labels: selection
              })}
            />
          </div>
          <div className="flex flex-row gap-2">
            <h3>
              Predictions:
            </h3>
            <Select
              options={columns.map((col) => col.accessorKey)}
              selection={columnLabels.predictions}
              setSelection={(selection: any) => setColumnLabels({
                ...columnLabels,
                predictions: selection
              })}
            />
          </div>
          <div className="flex flex-row gap-2">
            <h3>
              Privledged:
            </h3>
            <Select
              options={columns.map((col) => col.accessorKey)}
              selection={columnLabels.privledged}
              setSelection={(selection: any) => setColumnLabels({
                ...columnLabels,
                privledged: selection
              })}
              multiple
            />
          </div>
          {
            columnLabels.privledged.length > 0 && (
              <div className="flex items-center hover:text-gray-900 hover:cursor-pointer" onClick={() => setOpenThresholdField(!openThresholdField)}>
                <p className="text-sm text-gray-500 mr-2">Set privileged threshold</p>
                <div className="flex-grow border-t border-gray-300"></div>
                <p className="ml-2 text-xl font-bold text-gray-500">+</p>
              </div>
            )
          }
          {
            openThresholdField && (
              <div className="flex flex-col gap-2">
                <p className="text-xs text-gray-500 break-words wrap text-left">
                  The number you set will be used as the threshold. <br /> Any datapoint value larger than this number will be considered privileged.
                </p>
                {
                  columnLabels.privledged.split(", ").map((label: string, index: number) => (
                    <div className="flex flex-row gap-2 items-center" key={index}>
                      <h3>{label} Threshold:</h3>
                      <Select options={["greater", "lower"]} selection={thresholds[index].comparator} 

                        setSelection={(selection: any) => {
                          const newThresholds = [...thresholds];
                          newThresholds[index].comparator = selection;
                          setThresholds(newThresholds);
                        }} 
                      />
                      <input type="number" className="border border-gray-300 rounded-md p-1"
                        value={thresholds[index].amount ?? ""}
                        onChange={(e) => {
                          const newThresholds = [...thresholds];
                          newThresholds[index].amount = parseFloat(e.target.value);
                          setThresholds(newThresholds);
                        }} 
                      />
                    </div>
                  ))
                }
              </div>
            )
          }

        </div>
      </ModalBody>
      <ModalFooter className="flex flex-row justify-between">
        <Button variant="secondary" onClick={() => setCurrentStep(currentStep - 1)}>Back</Button>
        <div className="flex gap-4">
          <Button variant="secondary" onClick={closeModal}>Cancel</Button>
          <Button onClick={uploadData}>Confirm and Upload</Button>
        </div>
      </ModalFooter>
    </ModalContent>
  );
}