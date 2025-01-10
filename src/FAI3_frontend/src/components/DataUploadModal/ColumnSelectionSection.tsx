import { ModalContent, ModalHeader, ModalTitle, ModalBody, ModalFooter, Button, closeModal, Select } from "../ui";
import { useState, useContext } from "react";
import { FAI3_backend } from "../../../../declarations/FAI3_backend";
import { Table } from "@tanstack/react-table";
import { DataUploadContext } from "./utils";
import { useAuthClient } from "../../utils";

export default function ColumnSelectionSection({ fetchModel }: { fetchModel: () => Promise<any> }) {
  const { modelId, table, columns, currentStep, setCurrentStep }: {
    modelId: string | undefined,
    table: Table<any>,
    columns: any[],
    currentStep: number,
    setCurrentStep: (step: number) => void
  } = useContext(DataUploadContext);
  const { webapp } = useAuthClient();

  const [columnLabels, setColumnLabels] = useState<any>({
    labels: "",
    predictions: "",
    privledged: ""
  })
  const [loading, setLoading] = useState(false);

  const uploadData = async () => {
    setLoading(true);

    let labels: boolean[] = [];
    let predictions: boolean[] = [];
    const privledgedIndexs: bigint[] = []; //index of columns that are privledged
    let features: number[][] = [];

    const privledgedLabels = columnLabels.privledged.split(", ");

    for (let i = 0; i < columns.length; i++) {
      if (columns[i].accessorKey === columnLabels.labels) {
        labels = table.getRowModel().rows.map((row) => (row.original[columnLabels.labels] == 1 ? true : false));
      } else if (columns[i].accessorKey === columnLabels.predictions) {
        predictions = table.getRowModel().rows.map((row) => (row.original[columnLabels.predictions] == 1 ? true : false));
      } else if (privledgedLabels.includes(columns[i].accessorKey)) {
        privledgedIndexs.push(BigInt(i));
      } else {
        features.push(table.getRowModel().rows.map((row) => parseFloat(row.original[columns[i].accessorKey])));
      }
    }

    await webapp?.add_dataset(BigInt(modelId!), features, labels, predictions, privledgedIndexs);
    console.log("using webapp")
    await webapp?.calculate_all_metrics(BigInt(modelId!));
    await fetchModel();
    setLoading(false);
    closeModal();
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