import { flexRender, Table as TableType } from "@tanstack/react-table";
import { Table, TableHeader, TableRow, TableHead, TableBody, TableCell, Modal, ModalContent, Button, ModalBody, ModalHeader, ModalTitle } from "../ui";
import { useContext } from "react";
import { DataUploadContext } from "./utils";
import DataTable from "./DataTable";

export default function CSVTableView() {
  const { file, closeFile, table, columns, currentStep, setCurrentStep }: {
    file: File | null,
    closeFile: () => void,
    table: TableType<any>,
    columns: any,
    currentStep: number,
    setCurrentStep: (step: number) => void
  } = useContext(DataUploadContext);

  return (
    <ModalContent>
      <ModalHeader>
        <ModalTitle>{file?.name}</ModalTitle>
      </ModalHeader>
      <ModalBody>
        <div className="flex w-full gap-2">
          <Button onClick={() => setCurrentStep(currentStep + 1)}>
            Upload
          </Button>
          <Button variant="secondary" onClick={closeFile}>
            Use another file
          </Button>
          {/* <div className="flex w-full pt-2 text-red-700">
            {errorMessage}
          </div> */}
        </div>
        <DataTable table={table} columns={columns} />
      </ModalBody>
    </ModalContent>
  );
}