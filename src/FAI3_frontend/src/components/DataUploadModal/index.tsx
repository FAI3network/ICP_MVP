import { Modal } from "../ui";
import { useEffect, useState } from "react";
import Papa from "papaparse";
import { getCoreRowModel, useReactTable } from "@tanstack/react-table";
import { useParams } from "react-router-dom";
import ColumnSelectionSection from "./ColumnSelectionSection";
import CSVTableView from "./CSVTableView";
import UploadDataFile from "./UploadDataFile";
import { DataUploadContext } from "./utils";
import { FormBody } from "../AddModelModal";
import { ModelDetails } from "../../../../declarations/FAI3_backend/FAI3_backend.did";
import UpdateModel from "./UpdateModel";
import ImageUploader from "./ImageUploader";

export default function DataUploadModal({ fetchModel, latestVars, cachedThresholds, cachedSelections, onClose = () => { }, modelInfo, isOpen }: { fetchModel: () => Promise<any>, latestVars: any, cachedThresholds: any, cachedSelections: any, onClose: () => void, modelInfo: { id: number, name: string, details: ModelDetails }, isOpen?: boolean; }) {
  const [file, setFile] = useState<File | null>(null);
  const [data, setData] = useState<any[]>([]);
  const [columns, setColumns] = useState<any[]>([]);
  const [uploadedContent, setUploadedContent] = useState(false);
  const [currentStep, setCurrentStep] = useState(0);
  const [additionalImages, setAdditionalImages] = useState<any[]>([]);
  const [newModelInfo, setNewModelInfo] = useState<{ name: string, details: ModelDetails }>({ name: modelInfo.name, details: modelInfo.details });

  const { modelId } = useParams();

  const closeFile = () => {
    setFile(null);
    setUploadedContent(false);
    setCurrentStep(0);
  };

  useEffect(() => {
    if ((file && !Array.isArray(file) && file.type.includes("csv")) || (file && Array.isArray(file) && file[0].type.includes("csv"))) {
      const readingFile = Array.isArray(file) ? file[0] : file;
      Papa.parse(readingFile as File, {
        header: true,
        complete: (result: Papa.ParseResult<any>) => {
          //Do not accept filelds that are empty strings
          //Remove the empty string field
          console.log(result);
          if (result.data[0][""] !== undefined) {
            result.data.forEach((element: any) => {
              delete element[""];
            });
          }

          result.data = result.data.filter((row: any) => {
            return !Object.values(row).every((value) => value === null || value === "");
          });

          setData(result.data);
          createColumns(result.data);
        },
      });
    }
  }, [file]);

  const createColumns = (receivedData: any[]) => {
    const object = receivedData[0];

    const columnsObject = Object.keys(object).map((key, index) => {
      return {
        id: index.toString(),
        accessorKey: key,
        header: ({ column }: any) => {
          return <div className="flex justify-center items-center">{key}</div>;
        },
        cell: ({ row }: any) => {
          const value = row.original[key];
          const parsed = parseFloat(value);
          return isNaN(parsed) ? <div className="flex justify-center items-center">{value}</div> : <div className="flex justify-center items-center">{Number.isInteger(parsed) ? parsed : parsed.toFixed(2)}</div>;
        },
      };
    });

    setColumns(columnsObject);
  };

  const table = useReactTable({
    data: data || [],
    columns: columns || [],
    getCoreRowModel: getCoreRowModel(),
  });

  const steps = [
    <UpdateModel newModel={newModelInfo} setNewModel={setNewModelInfo} />,
    <UploadDataFile />,
    <CSVTableView />,
    <ColumnSelectionSection fetchModel={fetchModel} latestVars={latestVars} cachedThresholds={cachedThresholds} cachedSelections={cachedSelections} details={newModelInfo} />,
    <ImageUploader />,
  ];

  return (
    <DataUploadContext.Provider value={{ modelId, file, setFile, currentStep, setCurrentStep, table, columns, data, closeFile, additionalImages, setAdditionalImages }}>
      <Modal
        onClose={() => {
          onClose();
          closeFile();
        }}
        isOpen={isOpen}
      >
        {steps[currentStep]}
      </Modal>
    </DataUploadContext.Provider>
  );
}