import { Modal, ModalContent, ModalHeader, ModalTitle, ModalBody, Input, ModalFooter, Button, closeModal, CircularProgress, Select } from "@/components/ui";
import { Toggle } from "@/components/ui/toggle";
import { ModelDetails } from "../../../../declarations/FAI3_backend/FAI3_backend.did";
import { useState } from "react";

export default function FormBody({ update = false, newModel, setNewModel }: { update?: boolean, newModel: { name: string, details: ModelDetails, is_llm?: boolean, hf_url?: string, inference_provider?: string }, setNewModel: (model: { name: string, details: ModelDetails, is_llm?: boolean, hf_url?: string, inference_provider?: string }) => void }) {
    const [modelType, setModelType] = useState<string>(update ? newModel.is_llm ? "LLM" : "Classifier" : "");

    const handleModelTypeChange = (value: string) => {
        setModelType(value);
        setNewModel({ ...newModel, is_llm: value === "LLM" });
    };

    return (
        <ModalBody className="my-4">
            <h3 className="text-lg font-bold mb-4">
                Model Information
            </h3>

            {
                // !update && (
                //     <Toggle variant="outline" size="default" className="mb-4" onPressedChange={() => setNewModel({ ...newModel, is_llm: !newModel.is_llm })}>
                //         Is LLM
                //     </Toggle>)
                !update && (
                    <div className="mb-4">
                        <Select options={["LLM", "Classifier"]} selection={modelType} setSelection={(value: string) => handleModelTypeChange(value)} />
                    </div>
                )
            }

            {
                modelType && (
                    <>
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
                                <>
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
                                    <div>
                                        <h4 className="text-sm font-bold mb-2">
                                            Inference Provider
                                        </h4>
                                        <Input
                                            placeholder="inference_provider"
                                            className="mb-4"
                                            value={newModel.inference_provider || ""}
                                            onChange={(event: any) => setNewModel({ ...newModel, inference_provider: event.target.value })}
                                        />
                                    </div>
                                </>
                            )
                        }</>
                )}
        </ModalBody>
    )
}