import { Trash2 } from "lucide-react";
import { Input, Button, ModalContent, ModalBody, ModalHeader, ModalTitle } from "../ui";
import { useState, useContext } from "react";
import { DataUploadContext } from "./utils";

export default function ImageUploader() {
  const { file, closeFile } = useContext(DataUploadContext);
  const [imageData, setImageData] = useState<any[]>([{ label: "" }, {
    key: "",
    value: ""
  }]);
  const [errorMessage, setErrorMessage] = useState("");

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

  return (
    <ModalContent>
      <ModalHeader>
        <ModalTitle>{file?.name}</ModalTitle>
      </ModalHeader>
      <ModalBody>
        <div className="flex w-full gap-2">
          <Button onClick={uploadImageData}>
            Upload
          </Button>
          <Button variant="secondary" onClick={closeFile}>
            Use another file
          </Button>
          <div className="flex w-full pt-2 text-red-700">
            {errorMessage}
          </div>
        </div>
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
      </ModalBody>
    </ModalContent>
  )
}