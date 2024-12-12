import { flexRender, Table as TableType } from "@tanstack/react-table";
import { Table, TableHeader, TableRow, TableHead, TableBody, TableCell, Modal, ModalContent, Button, ModalBody, ModalHeader, ModalTitle } from "../ui";
import { useContext, useEffect, useState } from "react";
import { DataUploadContext } from "./utils";
import DataTable from "./DataTable";
import FileUpload from "../FileUpload";
import { X } from "lucide-react";

export default function CSVTableView() {
  const { file, closeFile, table, columns, currentStep, setCurrentStep }: {
    file: File | File[] | null,
    closeFile: () => void,
    table: TableType<any>,
    columns: any,
    currentStep: number,
    setCurrentStep: (step: number) => void
  } = useContext(DataUploadContext);

  const [additionalData, setAdditionalData] = useState<any[]>([]);
  const [moreData, setMoreData] = useState<boolean>(false);

  useEffect(() => {
    console.log(additionalData);
  }, [additionalData]);

  return (
    <ModalContent>
      <ModalHeader>
        <ModalTitle>{Array.isArray(file) ? file[0]?.name : file?.name}</ModalTitle>
      </ModalHeader>
      <ModalBody>
        <div className="flex w-full gap-2">
          <Button variant="secondary" onClick={closeFile}>
            Use another file
          </Button>
          <Button className="bg-slate-700" onClick={() => setMoreData(true)}>
            Add images
          </Button>
          <Button onClick={() => setCurrentStep(currentStep + 1)}>
            Next
          </Button>
          {/* <div className="flex w-full pt-2 text-red-700">
            {errorMessage}
          </div> */}
        </div>

        {
          moreData &&
          <div className="flex flex-col gap-2 mb-6">
            <FileUpload showFileName={false} onFileChange={(newFile) => setAdditionalData([...additionalData, ...(Array.isArray(newFile) ? newFile : [newFile])])} multiple={true} accept={"image/*"} />
            <div className="flex gap-2">
              {
                additionalData?.map((file, index) => (
                  <div key={index} className="flex relative w-fit h-fit px-4 py-8 border-2 rounded-md">
                    <X className="absolute top-2 right-2 cursor-pointer" onClick={() => setAdditionalData(additionalData.filter((_, i) => i !== index))} />
                    <span>{file.name}</span>
                  </div>
                ))
              }
            </div>

            <Button onClick={() => setMoreData(false)}>
              Done
            </Button>
          </div>
        }

        <DataTable table={table} columns={columns} />
      </ModalBody>
    </ModalContent>
  );
}