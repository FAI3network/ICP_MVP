import { Modal, ModalContent, ModalHeader, ModalTitle, ModalBody, Input, ModalFooter, Button, closeModal, CircularProgress } from "../ui";
import { useState } from "react";
import { useAuthClient, useDataContext } from "../../utils";

export default function AddModelModal() {
  const [newModel, setNewModel] = useState({ name: "", details: { description: "", framework: "", version: "", objective: "", url: "" } });
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

    // const model = await FAI3_backend.add_model(newModel.name, newModel.details);
    console.log(webapp);
    console.log(newModel);
    const model = await webapp?.add_model(newModel.name, newModel.details);
    console.log(model);

    if (model) {
      fetchModels();
      clearModelForm();
    }

    setInterval(() => {
      setLoading(false);
    }, 1000);
  }

  const clearModelForm = () => {
    setNewModel({ name: "", details: { description: "", framework: "", version: "", objective: "", url: "" } });
    closeModal();
  }

  return (
    <Modal>
      {
        loading ? (
          <ModalContent closeButton={false}>
            <CircularProgress />
          </ModalContent>
        ) : (
          <ModalContent className="w-1/3">
            <ModalHeader>
              <ModalTitle>
                Add Model
              </ModalTitle>
            </ModalHeader >
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
          </ModalContent >
        )
      }

    </Modal >
  )
}