import { flexRender } from "@tanstack/react-table";

import {
  ColumnFiltersState,
  SortingState,
  getCoreRowModel,
  getFilteredRowModel,
  getPaginationRowModel,
  getSortedRowModel,
  useReactTable,
} from "@tanstack/react-table";

import { ArrowUpDown } from "lucide-react";

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
import { DropdownMenuCheckboxes, AddModelModal } from "../../components";
import { FAI3_backend } from "../../../../declarations/FAI3_backend"
import { Model as ModelAsType, ClassifierModelData, Metrics } from "../../../../declarations/FAI3_backend/FAI3_backend.did";
import { useAuthClient, useDataContext } from "../../utils";

export default function ClassifierTable() {
    const { ClassifierModels } = useDataContext();
    const [sorting, setSorting] = useState<SortingState>([]);
    const [columnFilters, setColumnFilters] = useState<ColumnFiltersState>([]);
    const [columnVisibility, setColumnVisibility] = useState({});
    const [rowSelection, setRowSelection] = useState({});
  

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
        const model_data = row.original.model_type['Classifier'] as ClassifierModelData; 
        const metrics: Metrics = model_data.metrics;
        const cellValue = Number(metrics.average_metrics.statistical_parity_difference[0]);

        return isNaN(cellValue) ? null : (
            <div
              className={`ml-4 w-fit py-0.5 px-2 rounded-[10px]
                                ${cellValue < 0.1 && cellValue > -0.1
                  ? `text-[#007F00] bg-[#CDFFCD80] bg-opacity-50`
                  : cellValue > 0.4 || cellValue < -0.4
                    ? `text-[#D60E0E] bg-[#FFE0E0]`
                    : `text-[#CE8500] bg-[#FFECCC] bg-opacity-50`
                }`}
            >
              {cellValue.toFixed(3)}
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
        const model_data = row.original.model_type['Classifier'] as ClassifierModelData; 
        const metrics: Metrics = model_data.metrics;
        const cellValue = Number(metrics.average_metrics.disparate_impact[0]);

        return isNaN(cellValue) ? null : (
          <div
            className={`ml-4 w-fit py-0.5 px-2 rounded-[10px] ${cellValue > 0.8 && cellValue < 1.25
              ? `text-[#007F00] bg-[#CDFFCD80] bg-opacity-50`
              : cellValue < 0.8 || cellValue > 1.25
                ? `text-[#D60E0E] bg-[#FFE0E0]`
                : `text-[#CE8500] bg-[#FFECCC] bg-opacity-50`
              }`}
          >
            {cellValue.toFixed(3)}
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
        const model_data = row.original.model_type['Classifier'] as ClassifierModelData; 
        const metrics: Metrics = model_data.metrics;
        const cellValue = Number(metrics.average_metrics.average_odds_difference[0]);
        
        return isNaN(cellValue) ? null : (
          <div
            className={`ml-4 w-fit py-0.5 px-2 rounded-[10px] ${cellValue < 0.1 && cellValue > -0.1
              ? `text-[#007F00] bg-[#CDFFCD80] bg-opacity-50`
              : cellValue > 0.2 || cellValue < -0.2
                ? `text-[#D60E0E] bg-[#FFE0E0]`
                : `text-[#CE8500] bg-[#FFECCC] bg-opacity-50`
              }`}
          >
            {cellValue.toFixed(3)}
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
        const model_data = row.original.model_type['Classifier'] as ClassifierModelData; 
        const metrics: Metrics = model_data.metrics;
        const cellValue = Number(metrics.average_metrics.equal_opportunity_difference[0]);

        return isNaN(cellValue) ? null : (
          <div
            className={`ml-4 w-fit py-0.5 px-2 rounded-[10px] ${cellValue < 0.1 && cellValue > -0.1
              ? `text-[#007F00] bg-[#CDFFCD80] bg-opacity-50`
              : cellValue > 0.2 || cellValue < -0.2
                ? `text-[#D60E0E] bg-[#FFE0E0]`
                : `text-[#CE8500] bg-[#FFECCC] bg-opacity-50`
              }`}
          >
            {cellValue.toFixed(3)}
          </div>
        );
      },
    },
  ];

  const table = useReactTable({
    data: ClassifierModels,
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

  return (
    <div className="rounded-md border bg-[#fffaeb] shadow-lg overflow-hidden mb-3">
      <Table>
        <TableHeader>
          {table.getHeaderGroups().map((headerGroup: any) => (
            <TableRow
              key={headerGroup.id}
              className="hover:bg-[#ECE8EF] hover:bg-opacity-30"
            >
              <TableHead>#</TableHead>
              {headerGroup.headers.map((header: any) => (
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
            table.getRowModel().rows.map((row: any) => (
              <TableRow
                key={row.id}
                data-state={row.getIsSelected() && "selected"}
              >
                <TableCell>
                  {/* number of row */}
                  {row.index + 1}
                </TableCell>
                {row.getVisibleCells().map((cell: any) => (
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
  );
}
