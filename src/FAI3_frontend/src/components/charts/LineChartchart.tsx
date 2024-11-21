import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  ReferenceArea,
  ReferenceLine,
} from "recharts";

import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from "../ui";

export default function LineChartchart({
  dataKey,
  label,
  color,
  chartData,
  unfairRange,
  maxVal,
  minVal,
}: any) {
  //   console.log(label, ": ", Math.min(unfairRange[0], minVal));
  return (
    <ChartContainer
      config={{ [dataKey]: { label, color } }}
      className="min-h-[200px] w-full mb-8 "
    >
      <AreaChart
        width={500}
        height={400}
        data={chartData}
        margin={{
          top: 30,
          right: 30,
          left: 0,
          bottom: 0,
        }}
      >
        <CartesianGrid strokeDasharray="3 3" />
        <XAxis
          dataKey="timestamp"
          tickLine={false}
          tickMargin={10}
          axisLine={false}
          tickFormatter={(value) => value.slice(5, 10)} // Formats to show only MM-DD
        />
        <YAxis
          domain={[
            Math.min(unfairRange[0], minVal),
            Math.max(unfairRange[1], maxVal),
          ]}
        />
        <ChartTooltip content={<ChartTooltipContent />} />
        <Area type="monotone" dataKey={dataKey} stroke={color} fill={color} />
        <ReferenceLine
          y={unfairRange[0]}
          stroke="red"
          strokeDasharray="3 3"
          label={{
            value: `Unfairness Limit (${unfairRange[0]})`,
            position: "insideBottomRight",
          }}
        />
        <ReferenceLine
          y={unfairRange[1]}
          stroke="red"
          strokeDasharray="3 3"
          label={{
            value: `Unfairness Limit (${unfairRange[1]})`,
            position: "insideBottomRight",
          }}
        />
      </AreaChart>
    </ChartContainer>
  );
}
