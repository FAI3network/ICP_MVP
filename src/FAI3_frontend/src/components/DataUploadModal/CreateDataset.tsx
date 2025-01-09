import { Button, closeModal, ModalBody, ModalContent, ModalFooter, ModalHeader, ModalTitle } from "../ui";
import { useContext, useState } from "react";
import { DataUploadContext } from "./utils";
import { Table } from "@tanstack/react-table";
import DataTable from "./DataTable";

export default function CreateDataset() {
  const { file, setCurrentStep, table, columns }: {
    file: File | null,
    setCurrentStep: (step: number) => void,
    table: Table<any>,
    columns: any[]
  } = useContext(DataUploadContext);
  
  

  return (
    <ModalContent>
      <ModalHeader>
        <ModalTitle>Create a Dataset</ModalTitle>
      </ModalHeader>
      <ModalBody>
        <p>Upload your data to retrain the model.</p>


      </ModalBody>
      <ModalFooter>
        <Button variant="secondary" onClick={closeModal}>Cancel</Button>
      </ModalFooter>
    </ModalContent>
  )
}