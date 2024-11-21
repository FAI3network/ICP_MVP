import { Bar, BarChart, CartesianGrid, XAxis, YAxis } from "recharts";
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from "../ui";

export default function BarChartchart({ chartData, chartConfig }: any) {
  return (
    <ChartContainer config={chartConfig} className="min-h-[200px] w-full">
      <BarChart accessibilityLayer data={chartData}>
        <CartesianGrid vertical={false} />
        <XAxis
          dataKey="timestamp"
          tickLine={false}
          tickMargin={10}
          axisLine={false}
          tickFormatter={(value) => value.slice(5, 10)} // Formats to show only MM-DD
        />
        <YAxis />
        <ChartTooltip content={<ChartTooltipContent />} />
        <Bar dataKey="SPD" fill={chartConfig.SPD.color} radius={4} />
        <Bar dataKey="DI" fill={chartConfig.DI.color} radius={4} />
        <Bar dataKey="AOD" fill={chartConfig.AOD.color} radius={4} />
        <Bar dataKey="EOD" fill={chartConfig.EOD.color} radius={4} />
      </BarChart>
    </ChartContainer>
  );
}
