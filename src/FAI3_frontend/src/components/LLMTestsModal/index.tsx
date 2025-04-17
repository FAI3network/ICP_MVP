import { Modal, CircularProgress, ModalContent, ModalBody } from "../ui";
import { useEffect, useState } from "react";
import Papa from "papaparse";
import { getCoreRowModel, useReactTable } from "@tanstack/react-table";
import { useParams } from "react-router-dom";
import { LLMTestsContext } from "./utils";
import TestSelection from "./TestSelection";

export default function LLMTestsModal({ isOpen, onClose }: { isOpen?: boolean; onClose?: () => void }) {
  const [currentStep, setCurrentStep] = useState(0);
  const { modelId } = useParams();
  const [loading, setLoading] = useState(false);

  const steps = [<TestSelection setLoading={setLoading} />];

  return (
    <LLMTestsContext.Provider value={{ modelId, currentStep, setCurrentStep }}>
      <Modal isOpen={isOpen} onClose={onClose} className="w-full h-full">
        {loading ? (
          <ModalContent className="w-1/2 h-1/2">
            <ModalBody className="flex flex-col items-center justify-center h-full">
              <div className="flex flex-col items-center">
                <p className="text-lg font-semibold">Loading...</p>
                <CircularProgress />
              </div>
            </ModalBody>
          </ModalContent>
        ) : (
          steps[currentStep]
        )}
      </Modal>
    </LLMTestsContext.Provider>
  );
}
