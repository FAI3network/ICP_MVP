import FileUpload from "../FileUpload";
import { ModalContent, ModalHeader, ModalTitle, ModalBody, ModalFooter, Button, closeModal } from "../ui";
import { useContext, useState } from "react";
import { DataUploadContext } from "./utils";
import * as Switch from "@radix-ui/react-switch";

export default function UploadDataFile() {
  const { file, setFile, setCurrentStep }: {
    file: File | File[] | null,
    setFile: (file: File | File[]) => void,
    setCurrentStep: (step: number) => void
  } = useContext(DataUploadContext);

  const handleNextStep = () => {
    if (Array.isArray(file)) {
      if (file.length > 0 && file[0].type.includes("csv")) setCurrentStep(1);
    } else if (file?.type.includes("csv")) {
      setCurrentStep(1);
    }
  }


  return (
    <ModalContent>
      <ModalHeader>
        <ModalTitle>Evaluate</ModalTitle>
      </ModalHeader>
      <ModalBody>
        <p>Upload your data to retrain the model.</p>
        <FileUpload onFileChange={setFile} accept=".csv" multiple />
      </ModalBody>
      <ModalFooter>
        <Button variant="secondary" onClick={closeModal}>Cancel</Button>
        <Button
          disabled={!file}
          onClick={handleNextStep}>Next</Button>
      </ModalFooter>
    </ModalContent>
  )
}