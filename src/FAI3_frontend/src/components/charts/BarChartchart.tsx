import { Bar, BarChart, CartesianGrid, XAxis, YAxis, Legend } from "recharts";
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from "../ui";

export default function BarChartchart({ chartData, chartConfig }: any) {
  return (
    <ChartContainer config={chartConfig} className="min-h-[350px] w-full">
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
        {/* <Bar dataKey="average.SPD" fill={chartConfig.SPD.color} radius={4} />
        <Bar dataKey="average.DI" fill={chartConfig.DI.color} radius={4} />
        <Bar dataKey="average.AOD" fill={chartConfig.AOD.color} radius={4} />
        <Bar dataKey="average.EOD" fill={chartConfig.EOD.color} radius={4} /> */}~
        {Object.values(chartConfig).map((config: any, index: number) => (
          <Bar key={index} dataKey={config.key} fill={config.color} radius={4} />
        ))}
      </BarChart>
    </ChartContainer>
  );
}
