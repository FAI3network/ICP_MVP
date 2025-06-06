import { useAuthClient } from "@/utils";
import { Button, Select, Input } from "@/components/ui";
import { Checkbox } from "@/components/ui/checkbox";
import { useParams } from "react-router-dom";
import { z } from "zod";
import { UseFormReturn } from "react-hook-form";
import { Form, FormControl, FormDescription, FormField, FormItem, FormLabel, FormMessage } from "@/components/ui/form";
import { useEffect, useState } from "react";

interface formSchemaType {
  max_queries: number;
  seed: number;
  dataset: string[];
}

const fairnessTestSensVars: { "pisa": string, "compas": string } = {
  "pisa": "Gender",
  "compas": "Race"
}

export default function Fairness({ form }: { form: UseFormReturn<formSchemaType, any, formSchemaType> }) {
  const { webapp } = useAuthClient();
  const [datasets, setDatasets] = useState<string[]>([]);

  const fairnessTests = async () => (await webapp?.llm_fairness_datasets()) as string[];

  useEffect(() => {
    const fetchDatasets = async () => {
      const datasets = await fairnessTests();
      console.log("datasets", datasets);
      setDatasets(datasets.map((dataset: string) => dataset[0]));
    };

    fetchDatasets();
  }, []);

  return (
    <Form {...form}>
      <div className="space-y-8 flex flex-col text-left w-full">
        <FormField
          control={form.control}
          name="dataset"
          render={({ field }) => {
            useEffect(() => {
              console.log("field", field);
            }, [field]);

            return (
              <FormItem className="flex flex-col">
                <FormLabel>Datasets</FormLabel>
                <FormControl>
                  <div className="flex flex-row items-center gap-2">
                    <Select
                      options={datasets.map((dataset: string) => (dataset + " (" + fairnessTestSensVars[dataset as keyof typeof fairnessTestSensVars] + ")"))}
                      multiple
                      selection={form.getValues().dataset.join(", ")}
                      setSelection={(e: string) => field.onChange(e.split(", ").length === 1 && e.split(", ")[0] === "" ? [] : e.split(", "))}
                      placeholder="Select a dataset..."
                    />
                    {field.value.length > 0 && (
                      <div className="flex flex-row items-center gap-2">
                        <p>
                          (
                          {field.value.map((dataset: string, index: number) => {
                            return (
                              <span key={dataset} className="text-sm text-muted-foreground">
                                {dataset}
                              </span>
                            );
                          })}
                          )
                        </p>
                      </div>
                    )}
                  </div>
                </FormControl>
                <FormDescription>Select datasets to run test on</FormDescription>
                <FormMessage />
              </FormItem>
            )
          }}
        />
      </div>

      <form className="space-y-8 flex flex-col text-left w-full">
        <div className="grid grid-cols-1 gap-8 md:grid-cols-2">
          <FormField
            control={form.control}
            name="max_queries"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Max Queries</FormLabel>
                <FormControl>
                  <Input type="number" {...field} onChange={(e: any) => field.onChange(e.target.value ? parseInt(e.target.value, 10) : 0)} />
                </FormControl>
                <FormDescription>Maximum number of queries to run.</FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />

          <FormField
            control={form.control}
            name="seed"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Seed</FormLabel>
                <FormControl>
                  <Input type="number" {...field} onChange={(e: any) => field.onChange(e.target.value ? parseInt(e.target.value, 10) : 0)} />
                </FormControl>
                <FormDescription>Seed for random number generation.</FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />
        </div>
      </form>
    </Form>
  );
}
