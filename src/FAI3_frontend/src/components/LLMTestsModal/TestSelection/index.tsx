import { ModalContent, ModalHeader, ModalTitle, ModalBody, ModalFooter, Button, closeModal, Select, CircularProgress } from "../../ui";
import { useContext, useEffect, useState } from "react";
import { LLMTestsContext } from "../utils";
import ContextAssociation from "./ContextAssociation";
import Fairness from "./Fairness";
import { z } from "zod";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { useParams } from "react-router-dom";
import { useAuthClient, toasts, useDataContext } from "@/utils";
import { GenericError } from "../../../../../declarations/FAI3_backend/FAI3_backend.did";

const catFormSchema = z.object({
  max_queries: z.number().min(1, "Max queries must be between 1 and 1000").max(1000, "Max queries must be between 1 and 1000"),
  seed: z.number().min(0, "Seed must be between 0 and 1000").max(1000, "Seed must be between 0 and 1000"),
  shuffle: z.boolean(),
});

const fairnessFormSchema = z.object({
  max_queries: z.number().min(1, "Max queries must be between 1 and 1000").max(1000, "Max queries must be between 1 and 1000"),
  seed: z.number().min(0, "Seed must be between 0 and 1000").max(1000, "Seed must be between 0 and 1000"),
  dataset: z.array(z.string()).min(1, "At least one dataset must be selected"),
});

export default function TestSelection({ setLoading, fetchModel }: { setLoading: (loading: boolean) => void; fetchModel: () => void }) {
  const { currentStep, setCurrentStep } = useContext(LLMTestsContext);
  const [selectedTest, setSelectedTest] = useState<string>("");
  const { modelId } = useParams();
  const { webapp } = useAuthClient();
  const { fetchModels } = useDataContext();

  const catForm = useForm<z.infer<typeof catFormSchema>>({
    resolver: zodResolver(catFormSchema),
    defaultValues: {
      max_queries: 10,
      seed: 0,
      shuffle: false,
    },
  });

  const fairnessForm = useForm<z.infer<typeof fairnessFormSchema>>({
    resolver: zodResolver(fairnessFormSchema),
    defaultValues: {
      max_queries: 10,
      seed: 0,
      dataset: [],
    },
  });

  const evaluate = async () => {
    setLoading(true);
    if (selectedTest.includes("Context Association")) {
      await catForm.handleSubmit(async (data) => {
        console.log("Context Association Data", data);

        const res = await webapp?.context_association_test(BigInt(modelId!), data.max_queries, data.seed, data.shuffle);
        if (res && typeof res === "object" && res !== null && "Err" in res) {
          console.error("Failed to run context association test:", res.Err);
          const err = res.Err as GenericError;
          toasts.genericErrorToast(err);
        } else {
          console.log(res);
        }
      })();
    }
    if (selectedTest.includes("Fairness")) {
      await fairnessForm.handleSubmit(async (data) => {
        console.log("Fairness Data", data);

        data.dataset.forEach(async (dataset) => {
          console.log("Dataset", dataset);
          const res = await webapp?.calculate_llm_metrics(BigInt(modelId!), dataset, data.max_queries, data.seed);

          console.log("Fairness Result", res);
        });

        const res = await webapp?.average_llm_metrics(BigInt(modelId!), data.dataset);
        if (res && typeof res === "object" && res !== null && "Err" in res) {
          console.error("Failed to run context association test:", res.Err);
          const err = res.Err as GenericError;
          toasts.genericErrorToast(err);
        } else {
          console.log(res);
        }
      })();
    }

    fetchModel();
    fetchModels();

    setLoading(false);
    closeModal();
  };

  return (
    <ModalContent className="w-1/2">
      <ModalHeader>
        <ModalTitle>Which test would you like to run?</ModalTitle>
      </ModalHeader>
      <ModalBody className="flex flex-col gap-4">
        <Select options={["Context Association", "Fairness"]} multiple selection={selectedTest} setSelection={(selection: string) => setSelectedTest(selection)} />

        {selectedTest.includes("Context Association") && <ContextAssociation form={catForm} />}
        {selectedTest.includes("Fairness") && <Fairness form={fairnessForm} />}
      </ModalBody>
      <ModalFooter>
        <Button variant="secondary" onClick={closeModal}>
          Cancel
        </Button>
        <Button onClick={evaluate}>Evaluate</Button>
      </ModalFooter>
    </ModalContent>
  );
}
