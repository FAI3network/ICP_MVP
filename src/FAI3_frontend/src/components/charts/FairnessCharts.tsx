import { Card, CardHeader, CardTitle, CardDescription, CardContent, CardFooter } from "../ui";
import LineChartchart from "./LineChartchart";
import { fairnessConfig, prefixedFairnessConfig } from "@/configs";

export default function FairnessCharts({ metrics, key }: { metrics: any; key?: string }) {
  console.log(metrics);

  const chartConfig = key ? prefixedFairnessConfig(key) : fairnessConfig;

  return (
    <div className="grid gap-8 lg:grid-cols-2">
      <Card className="bg-[#fffaeb]">
        <CardHeader>
          <CardTitle>{chartConfig.SPD.label}</CardTitle>
          <CardDescription>{chartConfig.SPD.description}</CardDescription>
        </CardHeader>
        <CardContent>
          <LineChartchart
            dataKey="SPD"
            label={chartConfig.SPD.label}
            color={chartConfig.SPD.color}
            chartData={metrics}
            unfairRange={chartConfig.SPD.unfairRange}
            maxVal={metrics.reduce((max: any, p: any) => (p.average.SPD > max ? p.average.SPD : max), metrics[0]?.average.SPD)}
            minVal={metrics.reduce((min: any, p: any) => (p.average.SPD < min ? p.average.SPD : min), metrics[0]?.average.SPD)}
          />
        </CardContent>
        <CardFooter className="flex flex-col text-sm">
          <p>Unfair outcome: {chartConfig.SPD.footer.unfair}</p>
          <p>Fair outcome: {chartConfig.SPD.footer.unfair}</p>
        </CardFooter>
      </Card>
      <Card className="bg-[#fffaeb]">
        <CardHeader>
          <CardTitle>{chartConfig.DI.label}</CardTitle>
          <CardDescription>{chartConfig.DI.description}</CardDescription>
        </CardHeader>
        <CardContent>
          <LineChartchart
            dataKey="DI"
            label={chartConfig.DI.label}
            color={chartConfig.DI.color}
            chartData={metrics}
            unfairRange={chartConfig.DI.unfairRange}
            maxVal={metrics.reduce((max: any, p: any) => (p.average.DI > max ? p.average.DI : max), metrics[0]?.average.DI)}
            minVal={metrics.reduce((min: any, p: any) => (p.average.DI < min ? p.average.DI : min), metrics[0]?.average.DI)}
          />
        </CardContent>
        <CardFooter className="flex flex-col text-sm">
          <p>Unfair outcome: {chartConfig.DI.footer.unfair}</p>
          <p>Fair outcome: {chartConfig.DI.footer.unfair}</p>
        </CardFooter>
      </Card>
      <Card className="bg-[#fffaeb]">
        <CardHeader>
          <CardTitle>{chartConfig.AOD.label}</CardTitle>
          <CardDescription>{chartConfig.AOD.description}</CardDescription>
        </CardHeader>
        <CardContent>
          <LineChartchart
            dataKey="AOD"
            label={chartConfig.AOD.label}
            color={chartConfig.AOD.color}
            chartData={metrics}
            unfairRange={chartConfig.AOD.unfairRange}
            maxVal={metrics.reduce((max: any, p: any) => (p.average.AOD > max ? p.average.AOD : max), metrics[0]?.average.AOD)}
            minVal={metrics.reduce((min: any, p: any) => (p.average.AOD < min ? p.average.AOD : min), metrics[0]?.average.AOD)}
          />
        </CardContent>
        <CardFooter className="flex flex-col text-sm">
          <p>Unfair outcome: {chartConfig.AOD.footer.unfair}</p>
          <p>Fair outcome: {chartConfig.AOD.footer.unfair}</p>
        </CardFooter>
      </Card>
      <Card className="bg-[#fffaeb]">
        <CardHeader>
          <CardTitle>{chartConfig.EOD.label}</CardTitle>
          <CardDescription>{chartConfig.EOD.description}</CardDescription>
        </CardHeader>
        <CardContent>
          <LineChartchart
            dataKey="EOD"
            label={chartConfig.EOD.label}
            color={chartConfig.EOD.color}
            chartData={metrics}
            unfairRange={chartConfig.EOD.unfairRange}
            maxVal={metrics.reduce((max: any, p: any) => (p.average.EOD > max ? p.average.EOD : max), metrics[0]?.average.EOD)}
            minVal={metrics.reduce((min: any, p: any) => (p.average.EOD < min ? p.average.EOD : min), metrics[0]?.average.EOD)}
          />
        </CardContent>
        <CardFooter className="flex flex-col text-sm ">
          <p>Unfair outcome: {chartConfig.EOD.footer.unfair}</p>
          <p>Fair outcome: {chartConfig.EOD.footer.unfair}</p>
        </CardFooter>
      </Card>
    </div>
  );
}
