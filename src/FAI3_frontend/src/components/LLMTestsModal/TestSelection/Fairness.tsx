import { useAuthClient } from "@/utils";
import { Button, Select, Input } from "@/components/ui";
import { Checkbox } from "@/components/ui/checkbox";
import { useParams } from "react-router-dom";
import { z } from "zod";
import { UseFormReturn } from "react-hook-form";
import { Form, FormControl, FormDescription, FormField, FormItem, FormLabel, FormMessage } from "@/components/ui/form";

interface formSchemaType {
  max_queries: number;
  seed: number;
}

export default function Fairness({ form, formSchema }: { form: UseFormReturn<formSchemaType, any, formSchemaType>; formSchema: z.ZodObject<any> }) {
  const { webapp } = useAuthClient();
  const { modelId } = useParams();

  const fairnessTests = async () => await webapp?.llm_fairness_datasets();

  return (
    <Form {...form}>
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
