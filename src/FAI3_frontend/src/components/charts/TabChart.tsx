import { useEffect, useState } from "react";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle, Table, TableHeader, TableRow, TableHead, TableBody, TableCell, Button, Tabs, TabsContent, TabsList, TabsTrigger, Checkbox, Select } from "../ui";
import BarChartchart from "./BarChartchart";

export default function TabChart({ chartData, chartConfig, title = "Model Metrics", description = "Visualize the performance of your AI model over time." }: any) {
  const allMetricsKeys: string[] = Object.values(chartConfig).map((config: any) => config.key);
  const [selectedMetrics, setSelectedMetrics] = useState<string[]>(allMetricsKeys || []);
  const [selectedMetricsString, setSelectedMetricsString] = useState<string>(allMetricsKeys.join(", "));

  const filteredChartConfig = Object.entries(chartConfig).reduce((acc: any, [key, config]: [string, any]) => {
    if (selectedMetrics.includes(config.key)) {
      acc[key] = config;
    }
    return acc;
  }, {});

  useEffect(() => {
    setSelectedMetrics(selectedMetricsString.split(", "));
    console.log(selectedMetricsString);
  }, [selectedMetricsString]);

  return (
    <Tabs defaultValue="chart" className="">
      <TabsContent value="chart" className="pt-0 mt-0 h-full">
        <Card className="bg-[#fffaeb] h-full">
          <CardHeader className="relative">
            <CardTitle>{title}</CardTitle>
            <CardDescription>{description}</CardDescription>
            <TabsList className="grid w-[200px] grid-cols-2 absolute right-5 top-5">
              <TabsTrigger value="chart">Chart</TabsTrigger>
              <TabsTrigger value="table">Table</TabsTrigger>
            </TabsList>
          </CardHeader>
          <CardContent>
            <div className="mb-4 flex flex-wrap gap-4 items-center">
              <p className="text-sm font-medium ">Filter metrics:</p>

              <Select options={allMetricsKeys} selection={selectedMetricsString} setSelection={setSelectedMetricsString} placeholder="Filter metrics..." multiple />
            </div>
            <BarChartchart chartConfig={filteredChartConfig} chartData={chartData} />
          </CardContent>
        </Card>
      </TabsContent>
      <TabsContent value="table" className="pt-0 mt-0 h-full">
        <Card className="bg-[#fffaeb] h-full">
          <CardHeader className="relative">
            <CardTitle>Model Runs</CardTitle>
            <CardDescription>Detailed information about each model run.</CardDescription>
            <TabsList className="grid w-[200px] grid-cols-2 absolute right-5 top-5">
              <TabsTrigger value="chart">Chart</TabsTrigger>
              <TabsTrigger value="table">Table</TabsTrigger>
            </TabsList>
          </CardHeader>
          <CardContent>
            <div className="mb-4 flex flex-wrap gap-4">
              <Select options={allMetricsKeys} selection={selectedMetricsString} setSelection={setSelectedMetricsString} placeholder="Filter metrics..." multiple />
            </div>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Date</TableHead>
                  <TableHead>Version</TableHead>
                  {Object.values(chartConfig).map((config: any, index: number) => selectedMetrics.includes(config.key) && <TableHead key={index}>{config.label}</TableHead>)}
                </TableRow>
              </TableHeader>
              <TableBody>
                {chartData.map((row: any, index: number) => (
                  <TableRow key={index}>
                    <TableCell>{new Date(Number(row.timestamp) / 1e6).toISOString().split("T")[0]}</TableCell>
                    <TableCell>{index + 1} </TableCell>
                    {allMetricsKeys.map((key: string, idx: number) => {
                      if (!selectedMetrics.includes(key)) return null;

                      if (key.includes(".")) {
                        const parts = key.split(".");
                        let value = row;
                        for (const part of parts) {
                          value = value?.[part];
                          if (value === undefined) break;
                        }
                        return <TableCell key={idx}>{value !== undefined ? value : "-"}</TableCell>;
                      }

                      return <TableCell key={idx}>{row[key] !== undefined ? row[key] : "-"}</TableCell>;
                    })}
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </CardContent>
        </Card>
      </TabsContent>
    </Tabs>
  );
}
