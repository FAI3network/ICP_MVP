import { FormBody } from "../AddModelModal"
import { ModalContent, ModalHeader, ModalTitle, ModalBody, ModalFooter, Button, closeModal } from "../ui";
import { DataUploadContext } from "./utils";
import { useContext } from "react";

export default function UpdateModel({ newModel, setNewModel }: { newModel: any, setNewModel: (model: any) => void }) {
    const { setCurrentStep, currentStep }: {
        setCurrentStep: (step: number) => void,
        currentStep: number
      } = useContext(DataUploadContext);
    
      const handleNextStep = () => {
        setCurrentStep(currentStep + 1);
      }
    

    return (
        <ModalContent className="w-1/3 text-left">
            <ModalHeader>
                <ModalTitle>Update Model Details</ModalTitle>
            </ModalHeader>
            <FormBody update newModel={newModel} setNewModel={setNewModel} />
            <ModalFooter>
                <Button variant="secondary" onClick={closeModal}>Cancel</Button>
                <Button
                    onClick={handleNextStep}>Next</Button>
            </ModalFooter>
        </ModalContent>
    )
}