import { z } from "zod";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm, UseFormReturn } from "react-hook-form";
import { Button } from "@/components/ui/button";
import { Form, FormControl, FormDescription, FormField, FormItem, FormLabel, FormMessage } from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { Checkbox } from "@/components/ui/checkbox";
import { useEffect } from "react";

interface formSchemaType {
  max_queries: number;
  seed: number;
  shuffle: boolean;
}
export default function ContextAssociation({ form }: { form: UseFormReturn<formSchemaType, any, formSchemaType> }) {
  return (
    <Form {...form}>
      <form className="space-y-4 flex flex-col text-left w-full mx-0">
        <h2 className="text-2xl font-semibold">Context Association Test</h2>
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

        <FormField
          control={form.control}
          name="shuffle"
          render={({ field }) => (
            <FormItem className="flex flex-row items-start space-x-3 space-y-0 rounded-md border p-4">
              <FormControl>
                <Checkbox checked={field.value} onCheckedChange={field.onChange} />
              </FormControl>
              <div className="space-y-1 leading-none">
                <FormLabel>Shuffle Queries</FormLabel>
                <FormDescription>Shuffle the queries for randomized testing</FormDescription>
                <FormMessage />
              </div>
            </FormItem>
          )}
        />
      </form>
    </Form>
  );
}
