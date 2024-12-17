import { flexRender, Table as TableType } from "@tanstack/react-table";
import { Table, TableHeader, TableRow, TableHead, TableBody, TableCell, Modal, ModalContent, Button, ModalBody, ModalHeader, ModalTitle } from "../ui";
import { useContext, useEffect, useState } from "react";
import { DataUploadContext } from "./utils";
import DataTable from "./DataTable";
import FileUpload from "../FileUpload";
import { X } from "lucide-react";

export default function CSVTableView() {
  const { file, closeFile, table, columns, currentStep, setCurrentStep, additionalImages }: {
    file: File | File[] | null,
    closeFile: () => void,
    table: TableType<any>,
    columns: any,
    currentStep: number,
    setCurrentStep: (step: number) => void,
    additionalImages: any[]
  } = useContext(DataUploadContext);


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
          <Button className="bg-slate-700" onClick={() => setCurrentStep(3)}>
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
          additionalImages.length > 0 && (
            <div className="w-full text-left my-2">
              Additional Images ({additionalImages.length})
            </div>
          )
        }


        <DataTable table={table} columns={columns} />
      </ModalBody>
    </ModalContent>
  );
}