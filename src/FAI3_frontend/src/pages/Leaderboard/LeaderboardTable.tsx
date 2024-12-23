import {
  ColumnDef,
  ColumnFiltersState,
  SortingState,
  VisibilityState,
  flexRender,
  getCoreRowModel,
  getFilteredRowModel,
  getPaginationRowModel,
  getSortedRowModel,
  useReactTable,
} from "@tanstack/react-table";

import {
  ArrowUpDown,
  ChevronDown,
  ExternalLink,
  MoreHorizontal,
} from "lucide-react";

import {
  Button,
  Input,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
  Modal,
  ModalContent,
  ModalTrigger,
  ModalHeader,
  ModalBody,
  ModalTitle,
  ModalFooter,
  openModal,
  closeModal,

} from "../../components/ui";
import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { DropdownMenuCheckboxes } from "../../components";
import { FAI3_backend } from "../../../../declarations/FAI3_backend"
import { useAuthClient } from "../../utils";


export default function LeaderboardTable({ models, fetchModels }: any) {
  const [sorting, setSorting] = useState<SortingState>([]);
  const [columnFilters, setColumnFilters] = useState<ColumnFiltersState>([]);
  const [columnVisibility, setColumnVisibility] = useState({});
  const [rowSelection, setRowSelection] = useState({});
  const [newModel, setNewModel] = useState({ name: "", details: { description: "", framework: "", version: "", objective: "" } });
  const [errorMessage, setErrorMessage] = useState<string>("");
  const { webapp, connect, connected } = useAuthClient();

  const columns = [
    {
      accessorKey: "name",
      header: ({ column }: any) => {
        return (
          <Button
            variant="ghost"
            onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
          >
            Name
            <ArrowUpDown className="ml-2 h-4 w-4" />
          </Button>
        );
      },
      cell: ({ row }: any) => (
        // console.log(row.original),
        (
          <Link
            to={`/model/${row.original.model_id}`}
            className="text-start hover:underline"
          >
            {row.original.model_name}
          </Link>
        )
      ),
    },
    {
      accessorKey: "SPD",
      header: ({ column }: any) => {
        return (
          <Button
            variant="ghost"
            onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
          >
            Statistical Parity Difference
            <ArrowUpDown className="ml-2 h-4 w-4" />
          </Button>
        );
      },
      cell: ({ row }: any) => {
        const metrics = row.original.metrics;

        return (
          <div
            className={`ml-4 w-fit py-0.5 px-2 rounded-[10px]
                                ${metrics[0] < 0.1 && metrics[0] > -0.1
                ? `text-[#007F00] bg-[#CDFFCD80] bg-opacity-50`
                : metrics[0] > 0.4 || metrics[0] < -0.4
                  ? `text-[#D60E0E] bg-[#FFE0E0]`
                  : `text-[#CE8500] bg-[#FFECCC] bg-opacity-50`
              }`}
          >
            {Number(metrics.statistical_parity_difference[0]).toFixed(3)}
          </div>
        );
      },
    },
    {
      accessorKey: "DI",
      header: ({ column }: any) => {
        return (
          <Button
            variant="ghost"
            onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
          >
            Disparate Impact
            <ArrowUpDown className="ml-2 h-4 w-4" />
          </Button>
        );
      },
      cell: ({ row }: any) => {
        const metrics = row.original.metrics;

        return (
          <div
            className={`ml-4 w-fit py-0.5 px-2 rounded-[10px] ${metrics[1] > 0.8 && metrics[1] < 1.25
              ? `text-[#007F00] bg-[#CDFFCD80] bg-opacity-50`
              : metrics[1] < 0.8 || metrics[1] > 1.25
                ? `text-[#D60E0E] bg-[#FFE0E0]`
                : `text-[#CE8500] bg-[#FFECCC] bg-opacity-50`
              }`}
          >
            {Number(metrics.disparate_impact[0]).toFixed(3)}
          </div>
        );
      },
    },
    {
      accessorKey: "AOD",
      header: ({ column }: any) => {
        return (
          <Button
            variant="ghost"
            onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
          >
            Average Odds Difference
            <ArrowUpDown className="ml-2 h-4 w-4" />
          </Button>
        );
      },
      cell: ({ row }: any) => {
        const metrics = row.original.metrics;

        return (
          <div
            className={`ml-4 w-fit py-0.5 px-2 rounded-[10px] ${metrics[2] < 0.1 && metrics[2] > -0.1
              ? `text-[#007F00] bg-[#CDFFCD80] bg-opacity-50`
              : metrics[2] > 0.2 || metrics[2] < -0.2
                ? `text-[#D60E0E] bg-[#FFE0E0]`
                : `text-[#CE8500] bg-[#FFECCC] bg-opacity-50`
              }`}
          >
            {Number(metrics.average_odds_difference[0]).toFixed(3)}
          </div>
        );
      },
    },
    {
      accessorKey: "EOD",
      header: ({ column }: any) => {
        return (
          <Button
            variant="ghost"
            onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
          >
            Equal Opportunity Difference
            <ArrowUpDown className="ml-2 h-4 w-4" />
          </Button>
        );
      },
      cell: ({ row }: any) => {
        const metrics = row.original.metrics;

        return (
          <div
            className={`ml-4 w-fit py-0.5 px-2 rounded-[10px] ${metrics[2] < 0.1 && metrics[2] > -0.1
              ? `text-[#007F00] bg-[#CDFFCD80] bg-opacity-50`
              : metrics[2] > 0.2 || metrics[2] < -0.2
                ? `text-[#D60E0E] bg-[#FFE0E0]`
                : `text-[#CE8500] bg-[#FFECCC] bg-opacity-50`
              }`}
          >
            {Number(metrics.equal_opportunity_difference[0]).toFixed(3)}
          </div>
        );
      },
    },
  ];

  const table = useReactTable({
    data: models,
    columns,
    onSortingChange: setSorting,
    onColumnFiltersChange: setColumnFilters,
    getCoreRowModel: getCoreRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    onColumnVisibilityChange: setColumnVisibility,
    onRowSelectionChange: setRowSelection,
    state: {
      sorting,
      columnFilters,
      columnVisibility,
      rowSelection,
    },
  });

  const uploadModel = async () => {
    setErrorMessage("");

    if (newModel.name === "") {
      setErrorMessage("Please enter a model name.");
      return;
    }

    // const model = await FAI3_backend.add_model(newModel.name, newModel.details);
    console.log(webapp);
    const model = await webapp?.add_model(newModel.name, newModel.details);
    console.log(model);

    if (model) {
      fetchModels();
      closeModal();
      setNewModel({ name: "", details: { description: "", framework: "", version: "", objective: "" } });
    }
  }

  const clearModelForm = () => {
    setNewModel({ name: "", details: { description: "", framework: "", version: "", objective: "" } });
    closeModal();
  }

  return (
    <div className="w-full">
      {models && (
        <>
          <Modal>
            <ModalContent className="w-1/3">
              <ModalHeader>
                <ModalTitle>
                  Add Model
                </ModalTitle>
              </ModalHeader>
              <ModalBody className="my-4">
                <h3 className="text-lg font-bold mb-4">
                  Model Information
                </h3>
                <div>
                  <h4 className="text-sm font-bold mb-2">
                    Model Name
                  </h4>
                  <Input
                    placeholder="Model Name"
                    className="mb-4"
                    value={newModel.name}
                    onChange={(event: any) => setNewModel({ ...newModel, name: event.target.value })}
                  />
                </div>

                <div>
                  <h4 className="text-sm font-bold mb-2">
                    Model Description
                  </h4>
                  <Input
                    placeholder="description"
                    className="mb-4"
                    value={newModel.details.description}
                    onChange={(event: any) => setNewModel({ ...newModel, details: { ...newModel.details, description: event.target.value } })}
                  />
                </div>

                <div>
                  <h4 className="text-sm font-bold mb-2">
                    Model Framework
                  </h4>
                  <Input
                    placeholder="framework"
                    className="mb-4"
                    value={newModel.details.framework}
                    onChange={(event: any) => setNewModel({ ...newModel, details: { ...newModel.details, framework: event.target.value } })}
                  />
                </div>

                <div>
                  <h4 className="text-sm font-bold mb-2">
                    Model Version
                  </h4>
                  <Input
                    placeholder="version"
                    className="mb-4"
                    value={newModel.details.version}
                    onChange={(event: any) => setNewModel({ ...newModel, details: { ...newModel.details, version: event.target.value } })}
                  />
                </div>

                <div>
                  <h4 className="text-sm font-bold mb-2">
                    Model Objective
                  </h4>
                  <Input
                    placeholder="objective"
                    className="mb-4"
                    value={newModel.details.objective}
                    onChange={(event: any) => setNewModel({ ...newModel, details: { ...newModel.details, objective: event.target.value } })}
                  />
                </div>


              </ModalBody>
              <ModalFooter className="flex-col">
                <div className="text-red-500 text-sm w-full text-center">
                  {errorMessage}
                </div>
                <div className="flex w-full justify-end gap-2">
                  <Button onClick={clearModelForm}>
                    Cancel
                  </Button>
                  <Button onClick={uploadModel}>
                    Add
                  </Button>
                </div>
              </ModalFooter>
            </ModalContent>
          </Modal>

          <div className="flex items-center justify-center py-4 mb-4 gap-3">
            <Button onClick={() => {
              connected ? openModal() : connect();
            }}>
              Add Model
            </Button>
            <Input
              placeholder="Search your favorite model..."
              value={table.getColumn("name")?.getFilterValue() ?? ""}
              onChange={(event: any) => {
                table.getColumn("name")?.setFilterValue(event.target.value);
                console.log(event.target.value);
              }}
              className="max-w-sm"
            />
            <DropdownMenuCheckboxes />
          </div>
          <div className="rounded-md border bg-[#fffaeb] shadow-lg overflow-hidden mb-3">
            <Table>
              <TableHeader>
                {table.getHeaderGroups().map((headerGroup) => (
                  <TableRow
                    key={headerGroup.id}
                    className="hover:bg-[#ECE8EF] hover:bg-opacity-30"
                  >
                    <TableHead>#</TableHead>
                    {headerGroup.headers.map((header) => (
                      <TableHead key={header.id}>
                        {header.isPlaceholder
                          ? null
                          : flexRender(
                            header.column.columnDef.header,
                            header.getContext()
                          )}
                      </TableHead>
                    ))}
                  </TableRow>
                ))}
              </TableHeader>
              <TableBody>
                {table.getRowModel().rows?.length ? (
                  table.getRowModel().rows.map((row) => (
                    <TableRow
                      key={row.id}
                      data-state={row.getIsSelected() && "selected"}
                    >
                      <TableCell>
                        {/* number of row */}
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
          </div>
        </>
      )}
    </div>
  );
}
