import FileUpload from "../FileUpload";
import { ModalContent, ModalHeader, ModalTitle, ModalBody, ModalFooter, Button, closeModal } from "../ui";
import { useContext, useState } from "react";
import { DataUploadContext } from "./utils";
import * as Switch from "@radix-ui/react-switch";

export default function UploadDataFile() {
  const [isCompleteDataset, setIsCompleteDataset] = useState(true);

  const { file, setFile, setCurrentStep }: {
    file: File | null,
    setFile: (file: File) => void,
    setCurrentStep: (step: number) => void
  } = useContext(DataUploadContext);

  const handleNextStep = () => {
    if (isCompleteDataset) setCurrentStep(3);

    if (file?.type.includes("csv")) setCurrentStep(1);
  }

  return (
    <ModalContent>
      <ModalHeader>
        <ModalTitle>Upload Data</ModalTitle>
      </ModalHeader>
      <ModalBody>
        <p>Upload your data to retrain the model.</p>
        <div className="flex items-center my-2">
          <label
            className="pr-[15px] text-[15px] leading-none"
            htmlFor="airplane-mode"
          >
            Complete Dataset
          </label>
          <Switch.Root
            className="relative h-[20px] w-[38px] cursor-default rounded-full bg-gray-300 data-[state=checked]:bg-black"
            id="airplane-mode"
            defaultChecked
            onCheckedChange={(e) => setIsCompleteDataset(e)}
          >
            <Switch.Thumb
              className="block size-[15px] translate-x-0.5 rounded-full bg-white transition-transform duration-100 will-change-transform data-[state=checked]:translate-x-[19px]"
            />
          </Switch.Root>
        </div>
        <FileUpload onFileChange={setFile} accept=".csv, image/*" multiple />
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