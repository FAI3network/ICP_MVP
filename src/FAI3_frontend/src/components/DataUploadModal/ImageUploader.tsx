import { Trash2 } from "lucide-react";
import { Input, Button, ModalContent, ModalBody, ModalHeader, ModalTitle, ModalFooter } from "../ui";
import { useState, useContext, useEffect } from "react";
import { DataUploadContext } from "./utils";
import FileUpload from "../FileUpload";
import { X } from "lucide-react";

export default function ImageUploader() {
  const { file, setCurrentStep, setAdditionalImages } = useContext(DataUploadContext);
  const [additionalData, setAdditionalData] = useState<any[]>([]);

  useEffect(() => {
    setAdditionalImages(additionalData);
  }, [additionalData]);

  return (
    <ModalContent className="w-1/2 h-max flex flex-col">
      <ModalHeader>
        <ModalTitle>Upload Images</ModalTitle>
      </ModalHeader>
      <ModalBody className="flex-1 justify-center items-center">
        <div>
          <FileUpload showFileName={false} onFileChange={(newFile) => setAdditionalData([...additionalData, ...(Array.isArray(newFile) ? newFile : [newFile])])} multiple={true} accept={"image/*"} />
        </div>
        <div className="flex-1 gap-2 overflow-y-auto max-h-64 my-6">
          {
            additionalData?.map((file, index) => (
              <div key={index} className="flex justify-between items-center p-2 border-b">
                <span>{file.name}</span>
                <X className="cursor-pointer" onClick={() => setAdditionalData(additionalData.filter((_, i) => i !== index))} />
              </div>
            ))
          }
        </div>
      </ModalBody>
      <ModalFooter className="justify-between">
        <Button onClick={() => setCurrentStep(1)}>
          Done
        </Button>
        <Button variant="secondary" onClick={() => setAdditionalData([])}>
          Clear All
        </Button>
      </ModalFooter>
    </ModalContent>
  )
}