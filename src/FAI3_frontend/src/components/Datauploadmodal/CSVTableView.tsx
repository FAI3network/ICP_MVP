import { flexRender, Table as TableType } from "@tanstack/react-table";
import { Table, TableHeader, TableRow, TableHead, TableBody, TableCell, Modal, ModalContent, Button, ModalBody, ModalHeader, ModalTitle } from "../ui";
import { useContext } from "react";
import { DataUploadContext } from "./utils";

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
        <Table className="overflow-scroll">
          <TableHeader>
            {table.getHeaderGroups().map((headerGroup) => {
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
            {table.getRowModel().rows?.length ? (
              table.getRowModel().rows.map((row) => (
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
                  colSpan={columns.length}
                  className="h-24 text-center"
                >
                  No results.
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </ModalBody>
    </ModalContent>
  );
}