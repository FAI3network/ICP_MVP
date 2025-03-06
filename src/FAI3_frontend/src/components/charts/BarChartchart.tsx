import { Bar, BarChart, CartesianGrid, XAxis, YAxis } from "recharts";
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from "../ui";

export default function BarChartchart({ chartData, chartConfig }: any) {
  console.log(chartData);

  return (
    <ChartContainer config={chartConfig} className="min-h-[200px] w-full">
      <BarChart accessibilityLayer data={chartData}>
        <CartesianGrid vertical={false} />
        <XAxis
          dataKey="timestamp"
          tickLine={false}
          tickMargin={10}
          axisLine={false}
          tickFormatter={(_, index) => `v${index + 1}`}
        />
        <YAxis />
        <ChartTooltip content={<ChartTooltipContent />} />
        <Bar dataKey="average.SPD" fill={chartConfig.SPD.color} radius={4} />
        <Bar dataKey="average.DI" fill={chartConfig.DI.color} radius={4} />
        <Bar dataKey="average.AOD" fill={chartConfig.AOD.color} radius={4} />
        <Bar dataKey="average.EOD" fill={chartConfig.EOD.color} radius={4} />
      </BarChart>
    </ChartContainer>
  );
}
