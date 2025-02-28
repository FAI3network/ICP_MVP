import { useEffect } from "react";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
  Table,
  TableHeader,
  TableRow,
  TableHead,
  TableBody,
  TableCell,
  Button,
  Tabs, 
  TabsContent, 
  TabsList, 
  TabsTrigger
} from "../ui";
import BarChartchart from "./BarChartchart";

export default function TabChart({ chartData }: any) {
  const chartConfig = {
    SPD: {
      label: "Statistical Parity Difference",
      color: "#2563eb",
      description:
        "The statistical parity difference measures the difference in the positive outcome rates between the unprivileged group and the privileged group.",
      footer: {
        unfair: "SPD significantly different from 0 (e.g., -0.4 or 0.4)",
        fair: "SPD close to 0 (e.g., -0.1 to 0.1)",
      },
      fairRange: [-0.1, 0.1],
      unfairRange: [-0.4, 0.4],
    },
    DI: {
      label: "Disparate Impact",
      color: "#60a5fa",
      description:
        "Disparate impact compares the ratio of the positive outcome rates between the unprivileged group and the privileged group.",
      footer: {
        unfair:
          "DI significantly different from 1 (e.g., less than 0.8 or greater than 1.25)",
        fair: "DI close to 1 (e.g., 0.8 to 1.25)",
      },
      fairRange: [0.8, 1.25],
      unfairRange: [0.8, 1.25],
    },
    AOD: {
      label: "Average Odds Difference",
      color: "#10b981",
      description:
        "The average odds difference measures the difference in false positive rates and true positive rates between the unprivileged group and the privileged group.",
      footer: {
        fair: "AOD close to 0 (e.g., -0.1 to 0.1)",
        unfair: "AOD significantly different from 0 (e.g., -0.2 or 0.2)",
      },
      fairRange: [-0.1, 0.1],
      unfairRange: [-0.2, 0.2],
    },
    EOD: {
      label: "Equal Opportunity Difference",
      color: "#f97316",
      description:
        "The equal opportunity difference measures the difference in true positive rates between the unprivileged group and the privileged group.",
      footer: {
        fair: "EOD close to 0 (e.g., -0.1 to 0.1)",
        unfair: "EOD significantly different from 0 (e.g., -0.2 or 0.2)",
      },
      unfairRange: [-0.2, 0.2],
      fairRange: [-0.1, 0.1],
    },
  };

  return (
    <Tabs defaultValue="chart" className="">
      {/* <TabsList className="grid w-full grid-cols-2">
        <TabsTrigger value="chart">Chart</TabsTrigger>
        <TabsTrigger value="table">Table</TabsTrigger>
      </TabsList> */}
      <TabsContent value="chart" className="pt-0 mt-0 h-full">
        <Card className="bg-[#fffaeb] h-full">
          <CardHeader className="relative">
            <CardTitle>Model Metrics</CardTitle>
            <CardDescription>
              Visualize the performance of your AI model over time.
            </CardDescription>
            <TabsList className="grid w-[200px] grid-cols-2 absolute right-5 top-5">
              <TabsTrigger value="chart">Chart</TabsTrigger>
              <TabsTrigger value="table">Table</TabsTrigger>
            </TabsList>
          </CardHeader>
          <CardContent>
            <BarChartchart chartConfig={chartConfig} chartData={chartData} />
          </CardContent>
        </Card>
      </TabsContent>
      <TabsContent value="table" className="pt-0 mt-0 h-full">
        <Card className="bg-[#fffaeb] h-full">
          <CardHeader className="relative">
            <CardTitle>Model Runs</CardTitle>
            <CardDescription>
              Detailed information about each model run.
            </CardDescription>
            <TabsList className="grid w-[200px] grid-cols-2 absolute right-5 top-5">
              <TabsTrigger value="chart">Chart</TabsTrigger>
              <TabsTrigger value="table">Table</TabsTrigger>
            </TabsList>
          </CardHeader>
          <CardContent>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Date</TableHead>
                  <TableHead>Statistical Parity Difference (SPD)</TableHead>
                  <TableHead>Disparate Impact (DI)</TableHead>
                  <TableHead>Average Odds Difference (AOD)</TableHead>
                  <TableHead>Equal Opportunity Difference (EOD)</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {chartData.map((row: any, index: number) => (
                  <TableRow key={index}>
                    {/* <TableCell>{row.timestamp}</TableCell> */}
                    <TableCell>v{index+1}</TableCell>
                    <TableCell>{row.average.SPD}</TableCell>
                    <TableCell>{row.average.DI}</TableCell>
                    <TableCell>{row.average.AOD}</TableCell>
                    <TableCell>{row.average.EOD}</TableCell>
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
