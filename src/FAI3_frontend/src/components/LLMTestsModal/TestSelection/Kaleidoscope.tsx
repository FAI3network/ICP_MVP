import { useAuthClient } from "@/utils";
import { Button, Select, Input } from "@/components/ui";
import { Checkbox } from "@/components/ui/checkbox";
import { useParams } from "react-router-dom";
import { z } from "zod";
import { UseFormReturn } from "react-hook-form";
import { Form, FormControl, FormDescription, FormField, FormItem, FormLabel, FormMessage } from "@/components/ui/form";
import { useEffect, useState } from "react";

interface formSchemaType {
    languages: string[],
    max_queries: number,
    seed: number,
}

const availableLanuages = ["ar", "bn", "de", "en", "es", "fa", "fr", "hi", "hr", "hu", "lt", "nl", "pt", "ru", "sr", "uk"];

export default function Kaleidoscope({ form }: { form: UseFormReturn<formSchemaType, any, formSchemaType> }) {
    const { webapp } = useAuthClient();
    const [languages, setLanguages] = useState<string[]>([]);

    return (
        <Form {...form}>
            <div className="space-y-4 flex flex-col text-left w-full">
                <h2 className="text-2xl font-semibold">Kaleidoscope Test (Language Evaluation)</h2>

                <FormField
                    control={form.control}
                    name="languages"
                    render={({ field }) => {
                        useEffect(() => {
                            console.log("field", field);
                        }, [field]);

                        return (
                            <FormItem className="flex flex-col">
                                <FormLabel>Languages</FormLabel>
                                <FormControl>
                                    <div className="flex flex-row items-center gap-2">
                                        <Select
                                            options={availableLanuages}
                                            multiple
                                            selection={form.getValues().languages.join(", ")}
                                            setSelection={(e: string) => field.onChange(e.split(", ").length === 1 && e.split(", ")[0] === "" ? [] : e.split(", "))}
                                            placeholder="Select a language..."
                                        />
                                        {field.value.length > 0 && (
                                            <div className="flex flex-row items-center gap-2">
                                                <p>
                                                    (
                                                    {field.value.map((language: string, index: number) => {
                                                        return (
                                                            <span key={language} className="text-sm text-muted-foreground">
                                                                {language + (index < field.value.length - 1 ? ", " : "")}
                                                            </span>
                                                        );
                                                    })}
                                                    )
                                                </p>
                                            </div>
                                        )}
                                    </div>
                                </FormControl>
                                <FormDescription>Select languages to test model on</FormDescription>
                                <FormMessage />
                            </FormItem>
                        )
                    }}
                />
            </div>

            <form className="space-y-4 flex flex-col text-left w-full">
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
