import { Card, CardHeader, CardTitle, CardDescription, CardContent, CardFooter } from "../ui";
import LineChartchart from "./LineChartchart";
import { useState } from "react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "../ui/tabs";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../ui/table";

export default function KaleidoscopeChart({ metrics }: { metrics: any }) {
    const [selectedTab, setSelectedTab] = useState("overview");

    // chart config for language metrics
    const languageMetricsConfig = {
        overall_accuracy: {
            label: "Overall Accuracy",
            description: "Percentage of correct answers across all queries",
            color: "#00a86b",
            unfairRange: [0, 0.5],
        },
        accuracy_on_valid_responses: {
            label: "Accuracy on Valid Responses",
            description: "Percentage of correct answers among valid responses",
            color: "#4169e1",
            unfairRange: [0, 0.5],
        },
        format_error_rate: {
            label: "Format Error Rate",
            description: "Percentage of responses with format errors",
            color: "#e74c3c",
            unfairRange: [0.2, 1],
        },
        valid_response_rate: {
            label: "Valid Response Rate",
            description: "Percentage of responses that were valid",
            color: "#f39c12",
            unfairRange: [0, 0.8],
        },
    };

    const allLanguages = metrics
        .flatMap((metric: any) => metric.metrics_per_language?.map((lang: any) => lang[0]) || [])
        .filter((v: any, i: any, a: any) => a.indexOf(v) === i);

    return (
        <div className="space-y-6">
            <Tabs defaultValue="overview" onValueChange={setSelectedTab}>
                <TabsList className="grid grid-cols-2 md:grid-cols-3 mb-4">
                    <TabsTrigger value="overview">Overview</TabsTrigger>
                    <TabsTrigger value="languages">Languages</TabsTrigger>
                    <TabsTrigger value="details">Raw Data</TabsTrigger>
                </TabsList>

                <TabsContent value="overview" className="space-y-6">
                    <div className="grid gap-8 lg:grid-cols-2">
                        <Card className="bg-[#fffaeb]">
                            <CardHeader>
                                <CardTitle>{languageMetricsConfig.overall_accuracy.label}</CardTitle>
                                <CardDescription>{languageMetricsConfig.overall_accuracy.description}</CardDescription>
                            </CardHeader>
                            <CardContent>
                                <LineChartchart
                                    dataKey="metrics.overall_accuracy"
                                    label={languageMetricsConfig.overall_accuracy.label}
                                    color={languageMetricsConfig.overall_accuracy.color}
                                    chartData={metrics}
                                    unfairRange={languageMetricsConfig.overall_accuracy.unfairRange}
                                    maxVal={1}
                                    minVal={0}
                                    map={false}
                                />
                            </CardContent>
                        </Card>
                        <Card className="bg-[#fffaeb]">
                            <CardHeader>
                                <CardTitle>{languageMetricsConfig.accuracy_on_valid_responses.label}</CardTitle>
                                <CardDescription>{languageMetricsConfig.accuracy_on_valid_responses.description}</CardDescription>
                            </CardHeader>
                            <CardContent>
                                <LineChartchart
                                    dataKey="metrics.accuracy_on_valid_responses"
                                    label={languageMetricsConfig.accuracy_on_valid_responses.label}
                                    color={languageMetricsConfig.accuracy_on_valid_responses.color}
                                    chartData={metrics}
                                    unfairRange={languageMetricsConfig.accuracy_on_valid_responses.unfairRange}
                                    maxVal={1}
                                    minVal={0}
                                    map={false}
                                />
                            </CardContent>
                        </Card>
                        <Card className="bg-[#fffaeb]">
                            <CardHeader>
                                <CardTitle>Response Types</CardTitle>
                                <CardDescription>Distribution of response types across evaluations</CardDescription>
                            </CardHeader>
                            <CardContent>
                                <div>
                                    <div className="flex flex-col h-full justify-center">
                                        {metrics.length > 0 && (
                                            <div className="space-y-4">
                                                {metrics.map((evaluation: any, idx: number) => (
                                                    <div key={idx} className="space-y-2">
                                                        <p className="text-sm text-muted-foreground">Evaluation {idx + 1}</p>
                                                        <div className="flex w-full h-8">
                                                            <div
                                                                className="bg-green-500 h-full"
                                                                style={{
                                                                    width: `${(evaluation.metrics.correct_responses / evaluation.metrics.n) * 100}%`,
                                                                    minWidth: '1px',
                                                                }}
                                                                title={`${evaluation.metrics.correct_responses} correct responses`}
                                                            ></div>
                                                            <div
                                                                className="bg-red-500 h-full"
                                                                style={{
                                                                    width: `${(evaluation.metrics.incorrect_responses / evaluation.metrics.n) * 100}%`,
                                                                    minWidth: '1px',
                                                                }}
                                                                title={`${evaluation.metrics.incorrect_responses} incorrect responses`}
                                                            ></div>
                                                            <div
                                                                className="bg-yellow-500 h-full"
                                                                style={{
                                                                    width: `${(evaluation.metrics.invalid_responses / evaluation.metrics.n) * 100}%`,
                                                                    minWidth: '1px',
                                                                }}
                                                                title={`${evaluation.metrics.invalid_responses} invalid responses`}
                                                            ></div>
                                                            <div
                                                                className="bg-gray-500 h-full"
                                                                style={{
                                                                    width: `${(evaluation.metrics.error_count / evaluation.metrics.n) * 100}%`,
                                                                    minWidth: '1px',
                                                                }}
                                                                title={`${evaluation.metrics.error_count} errors`}
                                                            ></div>
                                                        </div>
                                                        <div className="flex text-xs justify-between">
                                                            <span>Total: {evaluation.metrics.n} queries</span>
                                                            <span>Languages: {evaluation.languages.join(", ")}</span>
                                                            <span>{new Date(Number(evaluation.timestamp) / 1e6).toLocaleDateString()}</span>
                                                        </div>
                                                    </div>
                                                ))}
                                            </div>
                                        )}
                                    </div>
                                </div>
                            </CardContent>
                        </Card>

                        <Card className="bg-[#fffaeb]">
                            <CardHeader>
                                <CardTitle>Query Distribution</CardTitle>
                                <CardDescription>Number of queries per evaluation</CardDescription>
                            </CardHeader>
                            <CardContent>
                                <div>
                                    <LineChartchart
                                        dataKey="metrics.n"
                                        label="Total Queries"
                                        color="#9c27b0"
                                        chartData={metrics}
                                        maxVal={Math.max(...metrics.map((m: any) => m.metrics.n))}
                                        minVal={0}
                                        unfairRange={[0, Math.max(...metrics.map((m: any) => m.metrics.n))]}
                                        map={false}
                                        iterable={false}
                                    />
                                </div>
                            </CardContent>
                        </Card>
                    </div>
                </TabsContent>

                <TabsContent value="languages" className="space-y-6">
                    <div className="grid gap-8 lg:grid-cols-2">
                        {allLanguages.map((language: string) => (
                            <Card key={language} className="bg-[#fffaeb]">
                                <CardHeader>
                                    <CardTitle>Language: {language}</CardTitle>
                                    <CardDescription>Performance metrics for {language} language</CardDescription>
                                </CardHeader>
                                <CardContent>
                                    {/* <LineChartchart
                                        dataKey={`metrics_per_language.${language}.overall_accuracy`}
                                        label={`${language} Accuracy`}
                                        color="#1e88e5"
                                        chartData={metrics.map((evaluation: any) => {
                                            const langMetrics = evaluation.metrics_per_language?.find((l: any) => l[0] === language);
                                            return {
                                                ...evaluation,
                                                metrics_per_language: {
                                                    [language]: langMetrics ? langMetrics[1] : null
                                                }
                                            };
                                        })}
                                        maxVal={1}
                                        minVal={0}
                                    /> */}
                                </CardContent>
                                <CardFooter>
                                    <div className="text-sm text-muted-foreground">
                                        {metrics.length > 0 &&
                                            `Latest: ${metrics[metrics.length - 1].metrics_per_language?.find((l: any) => l[0] === language)?.[1]?.overall_accuracy ?? 'N/A'}`}
                                    </div>
                                </CardFooter>
                            </Card>
                        ))}
                    </div>
                </TabsContent>

                <TabsContent value="details">
                    <Card className="bg-[#fffaeb]">
                        <CardHeader>
                            <CardTitle>Raw Evaluation Data</CardTitle>
                            <CardDescription>Detailed metrics for all language evaluations</CardDescription>
                        </CardHeader>
                        <CardContent>
                            <Table>
                                <TableHeader>
                                    <TableRow>
                                        <TableHead>Date</TableHead>
                                        <TableHead>Languages</TableHead>
                                        <TableHead>Queries</TableHead>
                                        <TableHead>Overall Accuracy</TableHead>
                                        <TableHead>Valid Response Accuracy</TableHead>
                                        <TableHead>Correct</TableHead>
                                        <TableHead>Incorrect</TableHead>
                                        <TableHead>Invalid</TableHead>
                                        <TableHead>Errors</TableHead>
                                    </TableRow>
                                </TableHeader>
                                <TableBody>
                                    {metrics.map((evaluation: any, idx: number) => (
                                        <TableRow key={idx}>
                                            <TableCell>{new Date(Number(evaluation.timestamp) / 1e6).toLocaleDateString()}</TableCell>
                                            <TableCell>{evaluation.languages.join(", ")}</TableCell>
                                            <TableCell>{evaluation.metrics.n}</TableCell>
                                            <TableCell>{evaluation.metrics.overall_accuracy ?? 'N/A'}</TableCell>
                                            <TableCell>{evaluation.metrics.accuracy_on_valid_responses ?? 'N/A'}</TableCell>
                                            <TableCell>{evaluation.metrics.correct_responses}</TableCell>
                                            <TableCell>{evaluation.metrics.incorrect_responses}</TableCell>
                                            <TableCell>{evaluation.metrics.invalid_responses}</TableCell>
                                            <TableCell>{evaluation.metrics.error_count}</TableCell>
                                        </TableRow>
                                    ))}
                                </TableBody>
                            </Table>
                        </CardContent>
                    </Card>
                </TabsContent>
            </Tabs>
        </div>
    );
}