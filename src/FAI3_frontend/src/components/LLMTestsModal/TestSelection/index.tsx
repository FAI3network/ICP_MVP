import { ModalContent, ModalHeader, ModalTitle, ModalBody, ModalFooter, Button, closeModal, Select, CircularProgress } from "../../ui";
import { useContext, useEffect, useState } from "react";
import { LLMTestsContext } from "../utils";
import ContextAssociation from "./ContextAssociation";
import Fairness from "./Fairness";
import { z } from "zod";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";

const catFormSchema = z.object({
  max_queries: z.number().min(1, "Max queries must be between 1 and 1000").max(1000, "Max queries must be between 1 and 1000"),
  seed: z.number().min(0, "Seed must be between 0 and 1000").max(1000, "Seed must be between 0 and 1000"),
  shuffle: z.boolean(),
});

const fairnessFormSchema = z.object({
  max_queries: z.number().min(1, "Max queries must be between 1 and 1000").max(1000, "Max queries must be between 1 and 1000"),
  seed: z.number().min(0, "Seed must be between 0 and 1000").max(1000, "Seed must be between 0 and 1000"),
  dataset: z.string().min(1, "Dataset must be selected"),
});

export default function TestSelection({ setLoading }: { setLoading: (loading: boolean) => void }) {
  const { currentStep, setCurrentStep } = useContext(LLMTestsContext);
  const [selectedTest, setSelectedTest] = useState<string>("");

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
      dataset: "",
    },
  });

  const handleNextStep = () => {
    // setCurrentStep(currentStep + 1);

    console.log(catForm.getValues());
  };

  return (
    <ModalContent className="w-1/2">
      <ModalHeader>
        <ModalTitle>Which test would you like to run?</ModalTitle>
      </ModalHeader>
      <ModalBody className="flex flex-col gap-4">
        <Select options={["Context Association", "Fairness"]} multiple selection={selectedTest} setSelection={(selection: string) => setSelectedTest(selection)} />

        {selectedTest.includes("Context Association") && <ContextAssociation form={catForm} formSchema={catFormSchema} />}
        {selectedTest.includes("Fairness") && <Fairness form={fairnessForm} formSchema={fairnessFormSchema} />}
      </ModalBody>
      <ModalFooter>
        <Button variant="secondary" onClick={closeModal}>
          Cancel
        </Button>
        <Button onClick={handleNextStep}>Next</Button>
      </ModalFooter>
    </ModalContent>
  );
}
