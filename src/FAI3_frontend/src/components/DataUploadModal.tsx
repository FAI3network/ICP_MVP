import {
  Button,
  Modal,
  ModalContent,
  ModalHeader,
  ModalBody,
  ModalTitle,
  ModalFooter,
  closeModal,
  Table,
  TableHeader,
  TableRow,
  TableHead,
  TableCell,
  TableBody,
  Input
} from "./ui";

import FileUpload from "./FileUpload";

import { useEffect, useState, forwardRef } from "react";

import Papa from "papaparse";

import {
  flexRender,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table";

import {
  Trash2
} from "lucide-react";

import { FAI3_backend } from "../../../declarations/FAI3_backend";

import { useParams } from "react-router-dom";

import * as Select from "@radix-ui/react-select";
import { cn } from "../utils";
import { Check, ChevronDown } from "lucide-react";

function DataUploadModal() {
  const [currentSection, setCurrentSection] = useState(0);
  const [file, setFile] = useState<File | null>(null);
  const [uploadedData, setUploadedData] = useState<any[]>([]);
  const [columns, setColumns] = useState<any[]>([]);
  const [showUploadedContent, setShowUploadedContent] = useState(false);
  const [imageData, setImageData] = useState<any[]>([{ label: "" }, {
    key: "",
    value: ""
  }]);
  const [errorMessage, setErrorMessage] = useState("");
  const [formattedData, setFormattedData] = useState<any>({
    labels: "",
    predictions: "",
    privledged: ""
  })

  const { modelId } = useParams();

  const handleFileUpload = () => {
    if (file?.type.includes("csv")) {
      Papa.parse(file as File, {
        header: true,
        complete: (result: Papa.ParseResult<any>) => {
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

    const columnsObject = Object.keys(object).map((key, index) => {
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

    setColumns(columnsObject);
  }

  const uploadedTable = useReactTable({
    data: uploadedData || [],
    columns: columns || [],
    getCoreRowModel: getCoreRowModel(),
  });

  useEffect(() => {
    if (columns.length) {
      setShowUploadedContent(true);
    }
  }, [columns]);

  const closeFile = () => {
    setFile(null);
    setUploadedData([]);
    setColumns([]);
    setShowUploadedContent(false);
    setCurrentSection(0);

    setImageData([{ label: "" }, {
      key: "",
      value: ""
    }]);
  }

  const uploadData = () => {
    setErrorMessage("");

    if (file?.type.includes("csv")) {
      confirmDataUpload();
    } else {
      uploadImageData();
    }

    closeModal();
  }

  const uploadImageData = () => {
    if (!imageData[0].label) {
      setErrorMessage("Label is required");
      return;
    }

    for (let i = 1; i < imageData.length; i++) {
      if (!imageData[i].key && imageData[i].value || imageData[i].key && !imageData[i].value) {
        setErrorMessage("Incomplete data");
        return;
      }
    }


  }

  const confirmDataUpload = async () => {
    let labels: boolean[] = [];
    let predictions: boolean[] = [];
    const privledgedIndexs: bigint[] = []; //index of columns that are privledged
    let features: number[][] = [];

    for (let i = 0; i < columns.length; i++) {
      if (columns[i].accessorKey === formattedData.labels) {
        labels = uploadedTable.getRowModel().rows.map((row) => (row.original[formattedData.labels] == 1 ? true : false));
      } else if (columns[i].accessorKey === formattedData.predictions) {
        predictions = uploadedTable.getRowModel().rows.map((row) => (row.original[formattedData.predictions] == 1 ? true : false));
      } else if (columns[i].accessorKey === formattedData.privledged) {
        privledgedIndexs.push(BigInt(i));
      } else {
        features.push(uploadedTable.getRowModel().rows.map((row) => parseFloat(row.original[columns[i].accessorKey])));
      }
    }

    await FAI3_backend.add_dataset(BigInt(modelId!), features, labels, predictions, privledgedIndexs);
  }

  const DataPreviewSection = () => (
    <ModalContent>
      <ModalHeader>
        <ModalTitle>{file?.name}</ModalTitle>
      </ModalHeader>
      <ModalBody>
        <div className="flex w-full gap-2">
          <Button onClick={() => setCurrentSection(1)}>
            Upload
          </Button>
          <Button variant="secondary" onClick={closeFile}>
            Use another file
          </Button>
        </div>
        <div className="flex w-full pt-2 text-red-700">
          {errorMessage}
        </div>
        {
          file?.type.includes("image") ? (
            <div>
              <img className="mb-4 mt-2" src={URL.createObjectURL(file)} alt="Uploaded" />
              <div className="flex flex-col space-y-2 items-start">
                <h3 className="text-lg text-gray-600 font-semibold">Data:</h3>
                <div className="flex w-full items-center">
                  <p className="text-sm text-left w-1/4">Label</p>
                  <strong className="text-xl mx-1">:</strong>
                  <Input
                    type="text"
                    placeholder="Label"
                    value={imageData[0].label}
                    onChange={(e: any) => {
                      const value = e.target.value;
                      setImageData(
                        imageData.map((d, i) => {
                          if (i === 0) {
                            return {
                              ...d,
                              label: value
                            }
                          }
                          return d;
                        })
                      )
                    }}
                    className="w-3/4 h-fit p-1 text-gray-600 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
                <div className="flex items-center flex-col gap-1">
                  {
                    imageData.map((data, index) => {
                      if (index == 0) return null;

                      return (
                        <div key={index} className="flex w-full items-center">
                          <Input
                            type="text"
                            placeholder="Key"
                            value={data.key}
                            onChange={(e: any) => {
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
                            className="w-1/4 h-fit p-1 text-gray-600 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                          />
                          <strong className="text-xl mx-1">:</strong>
                          <Input
                            type="text"
                            placeholder="Value"
                            value={data.value}
                            onChange={(e: any) => {
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
                            className="w-3/4 h-fit p-1 text-gray-600 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                          />
                          {
                            imageData.length > 2 && (
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
                      )
                    })
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
                      colSpan={columns.length}
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
  )

  const ColumnSelectionSection = () => (
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
            <SelectComp
              options={columns.map((col) => col.accessorKey)}
              selection={formattedData.labels}
              setSelection={(selection: any) => setFormattedData({
                ...formattedData,
                labels: selection
              })}
            />
          </div>
          <div className="flex flex-row gap-2">
            <h3>
              Predictions:
            </h3>
            <SelectComp
              options={columns.map((col) => col.accessorKey)}
              selection={formattedData.predictions}
              setSelection={(selection: any) => setFormattedData({
                ...formattedData,
                predictions: selection
              })}
            />
          </div>
          <div className="flex flex-row gap-2">
            <h3>
              Privledged:
            </h3>
            <SelectComp
              options={columns.map((col) => col.accessorKey)}
              selection={formattedData.privledged}
              setSelection={(selection: any) => setFormattedData({
                ...formattedData,
                privledged: selection
              })}
            />
          </div>
        </div>
      </ModalBody>
      <ModalFooter className="flex flex-row justify-between">
        <Button variant="secondary" onClick={() => setCurrentSection(0)}>Back</Button>
        <div className="flex gap-4">
          <Button variant="secondary" onClick={closeModal}>Cancel</Button>
          <Button onClick={uploadData}>Confirm and Upload</Button>
        </div>
      </ModalFooter>
    </ModalContent>
  )

  const SelectComp = ({ options, selection, setSelection }: any) => (
    <Select.Root value={selection} onValueChange={(value: string) => setSelection(value)}>
      <Select.Trigger className="w-fit inline-flex items-center justify-center rounded px-4 py-2 text-sm leading-none h-9 gap-1 bg-white shadow-md hover:bg-mauve-100 focus:outline-none focus:ring-2 focus:ring-black" aria-label="Food">
        <Select.Value placeholder="Select a fruit…" />
        <Select.Icon>
          <ChevronDown />
        </Select.Icon>
      </Select.Trigger>
      <Select.Portal>
        <Select.Content className="overflow-hidden bg-white rounded-lg shadow-lg z-50">
          <Select.ScrollUpButton className="flex items-center justify-center h-6 bg-white cursor-default">
            <ChevronDown />
          </Select.ScrollUpButton>
          <Select.Viewport className="p-1">
            <Select.Group>
              {
                options.map((option: any) => (
                  <SelectItem key={option} value={option}>{option}</SelectItem>
                ))
              }
            </Select.Group>
          </Select.Viewport>
          <Select.ScrollDownButton className="flex items-center justify-center h-6 bg-white cursor-default">
            <ChevronDown />
          </Select.ScrollDownButton>
        </Select.Content>
      </Select.Portal>
    </Select.Root>
  )

  const SelectItem = forwardRef(
    ({ children, className, ...props }: any, forwardedRef) => {

      return (
        <Select.Item
          className={cn("text-sm leading-none rounded flex items-center h-6 px-6 py-1 relative select-none", className)}
          {...props}
          ref={forwardedRef}
        >
          <Select.ItemText>{children}</Select.ItemText>
          <Select.ItemIndicator className="absolute left-0 w-6 flex items-center justify-center">
            <Check />
          </Select.ItemIndicator>
        </Select.Item>
      );
    },
  );

  const Sections = [
    DataPreviewSection,
    ColumnSelectionSection
  ]

  return (
    <Modal onClose={closeFile}>
      {
        showUploadedContent ? (
          Sections[currentSection]()
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
  )
}