import { Modal, ModalContent, ModalHeader, ModalTitle, ModalBody, Input, ModalFooter, Button, closeModal, CircularProgress } from "@/components/ui";
import { useEffect, useState } from "react";
import { useAuthClient, useDataContext } from "@/utils";
import { Toggle } from "@/components/ui/toggle";

interface ModelDetails {
  description: string;
  framework: string;
  version: string;
  objective: string;
  url: string;
}

export default function AddModelModal({ onClose = () => { }, name = null, details = null, update = false, modelId, fetchModel, is_llm, hf_url }: { onClose?: () => void, name?: string | null, details?: ModelDetails | null, update?: boolean, modelId?: number, fetchModel?: () => Promise<any>, is_llm?: boolean, hf_url?: string }) {
  const [newModel, setNewModel] = useState<{ name: string, details: ModelDetails, is_llm: boolean, hf_url: string }>({ name: name ?? "", details: details ?? { description: "", framework: "", version: "", objective: "", url: "" }, is_llm: is_llm ?? false, hf_url: hf_url ?? "" });
  const [errorMessage, setErrorMessage] = useState<string>("");
  const [loading, setLoading] = useState(false);
  const { webapp } = useAuthClient();
  const { fetchModels } = useDataContext();

  const uploadModel = async () => {
    setErrorMessage("");

    if (newModel.name === "") {
      setErrorMessage("Please enter a model name.");
      return;
    }

    setLoading(true);

    const modelName = newModel.name;
    const details = newModel.details;

    // const model = await FAI3_backend.add_model(newModel.name, newModel.details);
    const model = await (update? 
        webapp?.update_model(modelId, newModel.name, newModel.details)
      : newModel.is_llm ? 
        webapp?.add_llm_model(modelName, newModel.hf_url, details)
      : webapp?.add_classifier_model(modelName, details));

    if (model) {
      console.log("fetching and clearing");
      fetchModels();
      clearModelForm();

      if (fetchModel) {
        console.log("refetching model ");
        fetchModel();
      }
    }

    setInterval(() => {
      setLoading(false);
    }, 1000);
  }

  useEffect(() => {
    console.log(newModel);
  }, [newModel]);

  const clearModelForm = () => {
    setNewModel({ name: "", details: { description: "", framework: "", version: "", objective: "", url: "" }, is_llm: false, hf_url: "" });
    closeModal();
  }

  return (
    <Modal onClose={onClose}>
      {
        loading ? (
          <ModalContent closeButton={false}>
            <CircularProgress />
          </ModalContent>
        ) : (
          <ModalContent className="w-1/3 text-left">
            <ModalHeader>
              <ModalTitle>
                Add Model
              </ModalTitle>
            </ModalHeader >
            <ModalBody className="my-4">
              <h3 className="text-lg font-bold mb-4">
                Model Information
              </h3>

              {
                !update && (
                  <Toggle variant="outline" size="default" className="mb-4" onPressedChange={() => setNewModel({ ...newModel, is_llm: !newModel.is_llm })}>
                  Is LLM
                </Toggle>)
              }

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

              <div>
                <h4 className="text-sm font-bold mb-2">
                  Model URL
                </h4>
                <Input
                  placeholder="url"
                  className="mb-4"
                  value={newModel.details.url}
                  onChange={(event: any) => setNewModel({ ...newModel, details: { ...newModel.details, url: event.target.value } })}
                />
              </div>

              {
                newModel.is_llm && (
                  <div>
                    <h4 className="text-sm font-bold mb-2">
                      Hugging Face URL
                    </h4>

                    <Input
                      placeholder="hf_url"
                      className="mb-4"
                      value={newModel.hf_url}
                      onChange={(event: any) => setNewModel({ ...newModel, hf_url: event.target.value })}
                    />
                  </div>
                ) 
              }
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
                  {
                    update ? "Update" : "Add"
                  }
                </Button>
              </div>
            </ModalFooter>
          </ModalContent >
        )
      }

    </Modal >
  )
}
