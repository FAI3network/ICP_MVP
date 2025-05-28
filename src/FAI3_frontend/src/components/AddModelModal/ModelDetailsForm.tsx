import { Modal, ModalContent, ModalHeader, ModalTitle, ModalBody, Input, ModalFooter, Button, closeModal, CircularProgress } from "@/components/ui";
import { Toggle } from "@/components/ui/toggle";
import { useEffect, useState } from "react";
import { useAuthClient, useDataContext } from "@/utils";
import { ModelDetails } from "../../../../declarations/FAI3_backend/FAI3_backend.did";
import FormBody from "./FormBody";

export default function ModelDetailsForm({ update, modelId, name, details, is_llm, hf_url, inference_provider, fetchModel }: { update?: boolean, modelId?: number, name?: string | null, details?: ModelDetails | null, is_llm?: boolean, hf_url?: string, inference_provider?: string, fetchModel?: () => Promise<any>, clearModelForm: () => void, closeModal: () => void }) {
    const [newModel, setNewModel] = useState<{ name: string, details: ModelDetails, is_llm?: boolean, hf_url?: string, inference_provider?: string }>({ name: name ?? "", details: details ?? { description: "", framework: "", objective: "", url: "" }, is_llm: is_llm ?? false, hf_url: hf_url ?? "", inference_provider: inference_provider ?? "" });
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
        const model = await (update ?
            webapp?.update_model(modelId, newModel.name, newModel.details, true)
            : newModel.is_llm ?
                webapp?.add_llm_model(modelName, newModel.hf_url, details, [newModel.inference_provider])
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
        setNewModel({ name: "", details: { description: "", framework: "", objective: "", url: "" }, is_llm: false, hf_url: "", inference_provider: "" });
        closeModal();
    }

    return (
        <>
            {
                loading ? (
                    <ModalContent closeButton={false}>
                        <CircularProgress />
                    </ModalContent>
                ) : (
                    <ModalContent className="w-1/3 text-left">
                        <ModalHeader>
                            <ModalTitle>
                                {
                                    update ? (
                                        "Update Model"
                                    ) : (
                                        "Add Model"
                                    )
                                }
                            </ModalTitle>
                        </ModalHeader >
                        <FormBody update={update} newModel={newModel} setNewModel={setNewModel} />
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
                )}
        </>
    )
}

export {
    FormBody
}